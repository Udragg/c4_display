use std::{
    marker::PhantomData,
    sync::mpsc::{channel, Sender, TryRecvError},
    thread,
};

use crate::{
    display::{interface_components::*, Display, LedColor},
    PinConfig,
};

/// An interface for the display(s) created by the crate.
///
/// Can be used to create a new display, or communicate with an existing one.
#[derive(Debug)]
pub struct DisplayInterface<'d, S: State, const H: usize, const W: usize> {
    handle: Option<thread::JoinHandle<()>>,
    tx: Option<Sender<Message>>,
    state: PhantomData<S>,
    id: &'d str,
}

// ! REFACTOR INTO A STATE MACHINE
// ! state 1: stopped ; state after creation            ;   methods: start()
// ! state 2: running ; state after starting / resuming ;   methods: stop() pause()
// ! state 3: paused  ; state after pausing             ;   methods: resume()

impl<'d, const H: usize, const W: usize> DisplayInterface<'d, Stopped, H, W> {
    /// Create a new interface with the given id.
    pub fn new(id: &'d str) -> Self {
        Self {
            handle: None,
            tx: None,
            state: PhantomData,
            id,
        }
    }

    /// Start the display.
    ///
    /// This function creates a new thread with the name `disp: id` where `id` is the id given to the display interface upon creation.
    pub fn start(self, refresh: f64, pins: PinConfig) -> DisplayInterface<'d, Running, H, W> {
        let (tx, rx) = channel::<Message>();

        let mut disp = Display::<4, 4>::init(refresh, pins).expect("failed to configure gpio");

        let handle = thread::Builder::new()
            .name(String::from(format!("disp: {}", self.id)))
            .spawn(move || {
                // // move loop into a DisplayThreadManager struct
                loop {
                    match rx.try_recv() {
                        Ok(msg) => match msg {
                            Message::Pause => {
                                thread::park();
                                continue;
                            }
                            // Message::Resume => (),
                            Message::Stop => break,
                            Message::Sync(_) => (), // TODO sync board
                        },
                        Err(TryRecvError::Empty) => (),
                        Err(TryRecvError::Disconnected) => {
                            // panicked.store(true, Ordering::SeqCst);
                            log::error!("Display interface disconnected. Stopping thread...");
                            break;
                        }
                    }

                    disp.run_once();
                }
            })
            .expect("Failed to spawn display thread");

        DisplayInterface::<'d, Running, H, W> {
            handle: Some(handle),
            tx: Some(tx),
            id: self.id,
            state: PhantomData,
        }
    }
}

impl<'d, const H: usize, const W: usize> DisplayInterface<'d, Running, H, W> {
    /// Stops the display thread. All used pins will be reset to their default state and any information regarding the colors of the display will be lost.
    ///
    /// The display will only stop after it completes its current cycle. So it is possible it stops `1/refresh` seconds after it has been told to stop.
    ///
    /// The pin configuration however will be remembered.
    ///
    /// This is meant to be used when the display is no longer needed, and will be called automatically when the `DisplayInterface` instance is dropped.
    // send stop message to thread
    /// Calls join() on the display thread, prints the panic message if there is one and resets all thread related values to default.
    pub fn stop(self) -> DisplayInterface<'d, Stopped, H, W> {
        match self.tx {
            Some(tx) => tx.send(Message::Stop).expect("Failed to send message"),
            None => panic!("State machine broke: no transmitter found"),
        };

        match self.handle {
            Some(handle) => handle.join().unwrap(),
            None => panic!("State machine broke: no thread handle found"),
        }

        DisplayInterface::<'d, Stopped, H, W> {
            handle: None,
            tx: None,
            id: self.id,
            state: PhantomData,
        }
    }

    /// Pause the display thread. The display will no longer update but all data regarding its color and io pins state will remain.
    pub fn pause(self) -> DisplayInterface<'d, Paused, H, W> {
        match &self.tx {
            Some(tx) => tx.send(Message::Pause).expect("Failed to send message"),
            None => panic!("No transmitter connected"),
        }

        DisplayInterface::<'d, Paused, H, W> {
            handle: self.handle,
            tx: self.tx,
            id: self.id,
            state: PhantomData,
        }
    }
}

impl<'d, const H: usize, const W: usize> DisplayInterface<'d, Paused, H, W> {
    /// Resume the display thread.
    pub fn resume(self) -> DisplayInterface<'d, Running, H, W> {
        match &self.handle {
            Some(handle) => handle.thread().unpark(),
            None => panic!("No thread handle"),
        }

        DisplayInterface::<'d, Running, H, W> {
            handle: self.handle,
            tx: self.tx,
            id: self.id,
            state: PhantomData,
        }
    }
}

impl<'d, S: State, const H: usize, const W: usize> DisplayInterface<'d, S, H, W> {
    /// Returns the current state of the display
    pub fn get_state(&self) -> &str {
        stringify!(S)
    }
    /// Returns the id of the display thread
    pub fn get_id(&self) -> &str {
        self.id.clone()
    }

    /// Creates an empty board with
    pub fn sync_template() -> [[LedColor; W]; H] {
        [[LedColor::default(); W]; H]
    }
}

// impl<'d, S: State, const H: usize, const W: usize> Drop for DisplayInterface<'d, S, H, W> {
//     fn drop(&mut self) {
//         match S {
//             Running => (),
//             Stopped => (),
//             Paused => (),
//         }
//     }
// }

// impl<'d, S: State, const H: usize, const W: usize> Drop for DisplayInterface<'d, S, H, W> {
//     fn drop(&mut self) {
//         match self.state {
//             State::Running => self.stop(),
//             State::Paused => {
//                 self.resume();
//                 self.stop();
//             }
//             State::Stopped => (),
//         }
//     }
// }
