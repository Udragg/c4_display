// TODO animation system (single led)
// * vector of led positions, time the position is active, optional blink information
// * option to repeat the animation (forever or x times) (ex. for sidescrolling led at top row)
// * option to keep the last frame permanently (ex. for placement dropdown)
// ? option to reset a led to the state it was before it was affected by an animation

// TODO create animation from text file (macro?)

use std::{
    str::FromStr,
    time::{Duration, Instant},
};

use crate::{BlinkInfo, DisplayResult, Error, LedColor, LedState};

#[derive(Debug)]
pub enum AnimationParseError {
    MissingParam,
    BadFormatting,
    MissingSeperator,
}

/// Struct containing animation info.
#[derive(Debug)]
pub struct Animation {
    pub(super) r#loop: bool,                // enable permanent loop
    pub(super) frames: Vec<AnimationFrame>, // frames of the animation
    pub(super) repeats: usize,              // remaining times to repeat the animation
    pub(super) keep_last: bool,             // keep last frame active
    pub(super) activeframe: usize,
    pub(super) finished: bool,
}

/// A single frame of an animation.
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    pub(super) frame_dur: Duration, // time the frame is active
    pub(super) leds: Vec<(usize, usize, LedState)>, // x, y, led
    pub(super) start_time: Option<Instant>, // frame start time
    pub(super) rst_after: bool,     // clear affected leds after frame ends
}

impl Animation {
    /// Create a new animation.
    pub fn new(r#loop: bool, frames: Vec<AnimationFrame>, repeats: usize, keep_last: bool) -> Self {
        Self {
            r#loop,
            frames,
            repeats,
            keep_last,
            activeframe: 0,
            finished: false,
        }
    }

    /// Create a new animation from an ascii text file.
    // TODO text file layout
    pub fn from_file(file: &str) -> DisplayResult<Self> {
        match std::fs::read_to_string(file) {
            Ok(string) => match Self::from_str(string.as_str()) {
                Ok(animation) => Ok(animation),
                Err(e) => Err(Error::ParseError(e)),
            },
            Err(e) => {
                println!("{}", e);
                Err(Error::FileNotFound)
            }
        }
    }

    /// Increase the active frame by one.
    pub(super) fn next_frame(&mut self) {
        self.activeframe += 1;
    }

    /// Reset the active frame to frame 0.
    fn rst_frame_ctr(&mut self) {
        self.activeframe = 0;
    }

    /// Reset start time of all frames to [None](std::option::Option).
    fn rst_frame_st(&mut self) {
        for frame in &mut self.frames {
            frame.start_time = None;
        }
    }

    /// Reset the animation
    pub(super) fn rst(&mut self) {
        self.rst_frame_ctr();
        self.rst_frame_st();
        self.repeats = self.repeats.saturating_sub(1);
        self.finished = false;
    }
}

impl AnimationFrame {
    /// Create a new animation frame.
    pub fn new(frame_dur: Duration, leds: Vec<(usize, usize, LedState)>, rst_after: bool) -> Self {
        Self {
            frame_dur,
            leds,
            start_time: None,
            rst_after,
        }
    }

    // Check if the frame has finished
    pub(super) fn finished(&self) -> DisplayResult<bool> {
        let start_time = match self.start_time {
            Some(start_time) => start_time,
            None => return Err(Error::Uninitiated),
        };

        Ok(start_time + self.frame_dur < Instant::now())
    }
}

impl FromStr for Animation {
    type Err = AnimationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::AnimationParseError::*;

        let lowercased = s.to_lowercase();
        let mut lines = lowercased.trim().lines();
        let animation_loop;
        let animation_repeats: usize;
        let animation_keep_last;
        let mut animation_frames: Vec<AnimationFrame> = Vec::new();
        // check for animation keyword
        match lines.next() {
            Some(line) if line.trim() == "animation" => log::trace!("found keyword animation"),
            Some(line) => {
                log::error!("expected keyword animation, found: {line}");
                return Err(BadFormatting);
            }
            None => {
                log::error!("expected keyword animation, but lines ended");
                return Err(MissingParam);
            }
        }

        // get loop
        match lines.next() {
            Some(line) => {
                let mut vars = line.split_whitespace();

                // check loop keyword
                match vars.next() {
                    Some(var) if var == "loop" => log::trace!("found keyword loop"),
                    Some(var) => {
                        log::error!("expected keyword loop, found:  {var}");
                        return Err(BadFormatting);
                    }
                    None => return Err(MissingParam),
                }

                // get true or false
                match vars.next() {
                    Some(var) if var == "true" => {
                        log::trace!("found value {var}");
                        animation_loop = true;
                    }
                    Some(var) if var == "false" => {
                        log::trace!("found value {var}");
                        animation_loop = false;
                    }
                    Some(var) => {
                        log::error!("expected bool, found {var}");
                        return Err(BadFormatting);
                    }
                    None => {
                        log::error!("expected bool, found nothing");
                        return Err(MissingParam);
                    }
                }
            }
            None => {
                log::error!("expected line with loop info, but lines ended");
                return Err(MissingParam);
            }
        }

        // get repeats
        match lines.next() {
            Some(line) => {
                let mut vars = line.split_whitespace();

                // check repeats keyword
                match vars.next() {
                    Some(var) if var == "repeats" => log::trace!("found keyword repeats"),
                    Some(var) => {
                        log::error!("expected keyword repeats, found {var}");
                        return Err(BadFormatting);
                    }
                    None => {
                        log::error!("expected keyword repeats, found nothing");
                        return Err(MissingParam);
                    }
                }

                // parse repeats
                match vars.next() {
                    Some(var) => match var.parse() {
                        Ok(repeats) => {
                            log::trace!("found value {repeats}");
                            animation_repeats = repeats;
                        }
                        Err(_) => {
                            log::error!("expected usize, found {var}");
                            return Err(BadFormatting);
                        }
                    },
                    None => {
                        log::error!("expected usize, found nothing");
                        return Err(MissingParam);
                    }
                }
            }
            None => {
                log::error!("expected line with repeats info, but lines ended");
                return Err(MissingParam);
            }
        }

        // get keep_last
        match lines.next() {
            Some(line) => {
                let mut vars = line.split_whitespace();

                // check keep_last keyword
                match vars.next() {
                    Some(var) if var == "keep_last" => log::trace!("found keyword keep_last"),
                    Some(var) => {
                        log::error!("expected keyword keep_last, found {var}");
                        return Err(BadFormatting);
                    }
                    None => {
                        log::error!("expected keyword keep_last, found nothing");
                        return Err(MissingParam);
                    }
                }

                // get true or false
                match vars.next() {
                    Some(var) if var == "true" => {
                        log::trace!("found value {var}");
                        animation_keep_last = true;
                    }
                    Some(var) if var == "false" => {
                        log::trace!("found value {var}");
                        animation_keep_last = false;
                    }
                    Some(var) => {
                        log::error!("expected bool, found {var}");
                        return Err(BadFormatting);
                    }
                    None => {
                        log::error!("expected bool, found nothing");
                        return Err(MissingParam);
                    }
                }
            }
            None => {
                log::error!("expected line with keep_last info, but lines ended");
                return Err(MissingParam);
            }
        }

        match lines.next() {
            Some(line) if line.trim() == "" => (),
            _ => return Err(MissingSeperator),
        }

        let mut frame_str = String::new();
        for line in lines {
            match line.trim() {
                "" => {
                    animation_frames.push(AnimationFrame::from_str(frame_str.as_str())?);
                    frame_str.clear()
                }
                _ => {
                    frame_str.push_str(line);
                    frame_str.push('\n');
                }
            }
        }

        animation_frames.push(AnimationFrame::from_str(frame_str.as_str())?);

        Ok(Animation::new(
            animation_loop,
            animation_frames,
            animation_repeats,
            animation_keep_last,
        ))
    }
}

impl FromStr for AnimationFrame {
    type Err = AnimationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::AnimationParseError::*;

        let lowercased = s.to_lowercase();
        let mut lines = lowercased.trim().lines();
        let frame_dur: usize;
        let frame_rst;
        let mut frame_leds = Vec::new();

        // check if starts with frame keyword
        match lines.next() {
            Some(line) if line.trim() == "frame" => log::trace!("found keyword frame"),
            Some(line) => {
                log::error!("expected keyword frame, found {line}");
                return Err(MissingParam);
            }
            None => log::error!("expected line with keyword frame, but lines ended"),
        }

        // get frame duration
        match lines.next() {
            Some(line) => {
                let mut vars = line.split_whitespace();

                // check dur keyword
                match vars.next() {
                    Some(var_dur) if var_dur == "dur" => log::trace!("found keyword dur"),
                    Some(var) => {
                        log::error!("expected keyword dur, found {var}");
                        return Err(BadFormatting);
                    }
                    None => {
                        log::error!("expected keyword dur, found nothing");
                        return Err(MissingParam);
                    }
                }

                // parse duration
                match vars.next() {
                    Some(var) => match var.parse() {
                        Ok(dur) => {
                            log::trace!("found value {dur}");
                            frame_dur = dur;
                        }
                        Err(_) => {
                            log::error!("expected frame duration (usize), found {var}");
                            return Err(BadFormatting);
                        }
                    },
                    None => {
                        log::error!("expected frame duration (usize), found nothing");
                        return Err(MissingParam);
                    }
                }
            }
            None => {
                log::error!("expected line with duration info, but lines ended");
                return Err(MissingParam);
            }
        }

        // get rst_after flag
        match lines.next() {
            Some(line) => {
                let mut vars = line.split_whitespace();

                // check rst keyword
                match vars.next() {
                    Some(var) if var == "rst" => log::trace!("found keyword rst"),
                    Some(var) => {
                        log::error!("expected keyword rst, found {var}");
                        return Err(BadFormatting);
                    }
                    None => {
                        log::error!("expected keyword rst, found nothing");
                        return Err(MissingParam);
                    }
                }

                // get true or false
                match vars.next() {
                    Some(var) if var == "true" => {
                        log::trace!("found value {var}");
                        frame_rst = true;
                    }
                    Some(var) if var == "false" => {
                        log::trace!("found value {var}");
                        frame_rst = false;
                    }
                    Some(var) => {
                        log::error!("expected reset value (bool), found {var}");
                        return Err(BadFormatting);
                    }
                    None => {
                        log::error!("expected reset value (bool), found nothing");
                        return Err(MissingParam);
                    }
                }
            }
            None => {
                log::error!("expected line with reset info, but lines ended");
                return Err(MissingParam);
            }
        }

        // get leds
        for line in lines {
            let led_x: usize;
            let led_y: usize;
            let led_color: LedColor;
            let led_blink_dur: usize;
            let led_blink_int: usize;

            let mut vars = line.split_whitespace();

            // led x
            match vars.next() {
                Some(var) => match var.parse() {
                    Ok(x) => {
                        log::trace!("found x position {x}");
                        led_x = x;
                    }
                    Err(_) => {
                        log::error!("expected led x pos (usize), found {var}");
                        return Err(BadFormatting);
                    }
                },
                None => {
                    log::error!("expected led x pos (usize), found nothing");
                    return Err(MissingParam);
                }
            }

            // led y
            match vars.next() {
                Some(var) => match var.parse() {
                    Ok(y) => {
                        log::trace!("found y position {y}");
                        led_y = y;
                    }
                    Err(_) => {
                        log::error!("expected led y pos (usize), found {var}");
                        return Err(BadFormatting);
                    }
                },
                None => {
                    log::error!("expected led y pos (usize), found nothing");
                    return Err(MissingParam);
                }
            }

            // led color
            match vars.next() {
                Some(var) => {
                    led_color = match LedColor::from_str(var) {
                        Ok(color) => {
                            log::trace!("found color {color:?}");
                            color
                        }
                        Err(e) => {
                            log::error!("expected color, found {var} with error {e:?}");
                            return Err(BadFormatting);
                        }
                    }
                }
                None => {
                    log::error!("expected color, found nothing");
                    return Err(MissingParam);
                }
            }

            // blink dur
            match vars.next() {
                Some(var) => match var.parse() {
                    Ok(dur) => {
                        log::trace!("found blink duration {dur}");
                        led_blink_dur = dur;
                    }
                    Err(_) => {
                        log::error!("expected blink duration (usize), found {var}");
                        return Err(BadFormatting);
                    }
                },
                None => {
                    frame_leds.push((led_x, led_y, LedState::with_color(led_color)));
                    continue;
                }
            }

            // blink int
            match vars.next() {
                Some(var) => match var.parse() {
                    Ok(int) => {
                        log::trace!("found blink interval {int}");
                        led_blink_int = int
                    }
                    Err(_) => {
                        log::error!("expected blink interval (usize), found {var}");
                        return Err(BadFormatting);
                    }
                },
                None => {
                    log::error!("expected blink interval (usize), found nothing");
                    return Err(MissingParam);
                }
            }

            frame_leds.push((
                led_x,
                led_y,
                LedState {
                    color: led_color,
                    blink: Some(BlinkInfo {
                        dur: Duration::from_millis(led_blink_dur as u64),
                        int: Duration::from_millis(led_blink_int as u64),
                    }),
                },
            ));
        }

        return Ok(AnimationFrame::new(
            Duration::from_millis(frame_dur as u64),
            frame_leds,
            frame_rst,
        ));
    }
}
