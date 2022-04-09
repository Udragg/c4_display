use crate::{display::Display, display::Instruction, LedState, Sync, SyncType};
use std::{
    sync::mpsc::{Receiver, TryRecvError},
    thread,
    time::Instant,
};

use super::animation::Animation;

pub(super) struct DisplayManager<const W: usize, const H: usize> {
    disp: Display<W, H>,
    rx: Receiver<Instruction>,
    animations: Vec<Animation>,
}

impl<const W: usize, const H: usize> DisplayManager<W, H> {
    /// Create a new `DisplayManager` with the given `Display` and `Receiver`.
    pub(super) fn new(disp: Display<W, H>, rx: Receiver<Instruction>) -> Self {
        Self {
            disp,
            rx,
            animations: Vec::new(),
        }
    }

    /// Start the display.
    pub(super) fn start(&mut self) {
        loop {
            let start_time = std::time::Instant::now();
            // get new sync instructions
            match self.rx.try_recv() {
                Ok(msg) => match msg {
                    Instruction::Pause => {
                        thread::park();
                        continue;
                    }
                    Instruction::Stop => break,
                    Instruction::Sync(sync_type) => self.disp.sync(sync_type),
                    Instruction::AddAnimation(animation) => self.animations.push(animation),
                    Instruction::ClearAnimations => self.animations.clear(),
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    log::error!("Display interface disconnected. Stopping thread...");
                    break;
                }
            }

            // update display with animations
            // newer animations will override older ones if they affect the same leds
            // TODO refactor into methods, this is unreadable
            // TODO remove flicker at end of restarting animations that occurs because last frame is cleared and next frame only gets loaded on cycle later
            for animation in &mut self.animations {
                let prev_frame = if animation.activeframe > 0 {
                    Some(animation.frames[animation.activeframe - 1].clone())
                } else {
                    None
                };

                match animation.frames.get_mut(animation.activeframe) {
                    Some(frame) => {
                        // the first time the frame is run a start time is assigned
                        // the frame is written to the display
                        if let None = frame.start_time {
                            frame.start_time = Some(Instant::now());

                            if let Some(frame) = prev_frame {
                                if frame.rst_after {
                                    for (x, y, _) in &frame.leds {
                                        self.disp.sync(SyncType::Single(Sync {
                                            x: *x,
                                            y: *y,
                                            state: LedState::default(),
                                        }));
                                    }
                                }
                            }

                            for (x, y, state) in &frame.leds {
                                self.disp.sync(SyncType::Single(Sync {
                                    x: *x,
                                    y: *y,
                                    state: *state,
                                }));
                            }
                        }

                        match frame.finished() {
                            // if the frame has finished, move on to the next frame
                            // a frame is finished when start_time + frame_duration > current_time
                            Ok(finished) if finished => {
                                // set leds affected by the frame to Off if reset_frame is set to true
                                // if frame.rst_after {
                                //     for (x, y, _) in &frame.leds {
                                //         self.disp.sync(SyncType::Single(Sync {
                                //             x: *x,
                                //             y: *y,
                                //             state: LedState::default(),
                                //         }));
                                //     }
                                // }
                                animation.next_frame()
                            }
                            // if the frame hasn't finished, do nothing
                            Ok(_) => (),
                            Err(_) => panic!("No start time exists"),
                        }
                    }
                    // if no frame is returned, the animation has finished
                    None => animation.finished = true,
                }

                if animation.finished
                    && animation
                        .frames
                        .last()
                        .expect("No frames in animation")
                        .rst_after
                {
                    for (x, y, _) in &animation.frames.last().unwrap().leds {
                        self.disp.sync(SyncType::Single(Sync {
                            x: *x,
                            y: *y,
                            state: LedState::default(),
                        }));
                    }
                }

                // remove finished flag for repeating animations
                match animation.finished {
                    true if animation.r#loop => animation.rst(),
                    true if animation.repeats > 0 => animation.rst(),

                    _ => (),
                }
            }

            // remove finished animations
            // self.animations.retain(|animation| !animation.finished);
            self.animations.retain(|animation| {
                if animation.finished && animation.keep_last {
                    for (x, y, state) in &animation
                        .frames
                        .last()
                        .expect("No frames in animation")
                        .leds
                    {
                        self.disp.sync(SyncType::Single(Sync {
                            x: *x,
                            y: *y,
                            state: *state,
                        }));
                    }
                }
                !animation.finished
            });

            // run multiplexing
            self.disp.run_once(start_time);
        }
    }
}

impl<const W: usize, const H: usize> Drop for DisplayManager<W, H> {
    fn drop(&mut self) {
        self.disp.clear_row();
    }
}
