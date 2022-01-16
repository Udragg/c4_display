use std::{
    marker::PhantomData,
    sync::mpsc::{channel, Sender},
    thread,
};

use crate::{
    display::{interface_components::*, Display, DisplayManager, LedColor},
    error, PinConfig,
};

/// An interface for the display created by the crate.
///
/// If this gets dropped or goes out of scope the display will stop working.
#[derive(Debug)]
pub struct DisplayInterface<'d, S: State, const W: usize, const H: usize> {
    handle: Option<thread::JoinHandle<()>>,
    tx: Option<Sender<Message>>,
    state: PhantomData<S>,
    id: &'d str,
}

impl<'d, const W: usize, const H: usize> DisplayInterface<'d, Stopped, W, H> {
    /// Create a new interface with the given id.
    ///
    /// # Example
    ///
    /// ```
    /// // Create a variable with the pin configuration
    /// let pin_config = PinConfig {
    ///     sr_serin: 0,
    ///     sr_srclk: 0,
    ///     sr_rclk: 0,
    ///     sr_srclr: 0,
    ///     sr_oe: 0,
    ///     dec_a0: 0,
    ///     dec_a1: 0,
    ///     dec_a2: 0,
    /// };
    ///
    /// // Create and start the display
    /// let display = DisplayInterface::<_, 4, 4>::new("id").start(
    ///     refresh: 30.0,
    ///     pins: pin_config,
    /// );
    ///
    /// // Wait 5 seconds
    /// std::thread::sleep(std::time::Duration::from_secs(5));
    ///
    /// // Stop the display
    /// display.stop();
    /// ```
    pub fn new(id: &'d str) -> Self {
        Self {
            handle: None,
            tx: None,
            state: PhantomData,
            id,
        }
    }

    /// Start the display. It will run at the given refresh rate and make use of the gpio pins provided in `PinConfig`.
    ///
    /// This function creates a new thread with the name `disp: id` where `id` is the id given to the display interface upon creation.
    pub fn start(self, refresh: f64, pins: PinConfig) -> DisplayInterface<'d, Running, W, H> {
        let (tx, rx) = channel::<Message>();
        let disp = match Display::<W, H>::init(refresh, pins) {
            Ok(disp) => disp,
            Err(e) => panic!("failed to initialise diplay: {}", e), // TODO return error to user.
        };
        let handle = thread::Builder::new()
            .name(String::from(format!("disp: {}", self.id)))
            .spawn(move || DisplayManager::new(disp, rx).start())
            .expect("Failed to spawn display thread");

        DisplayInterface::<'d, Running, W, H> {
            handle: Some(handle),
            tx: Some(tx),
            id: self.id,
            state: PhantomData,
        }
    }
}

impl<'d, const W: usize, const H: usize> DisplayInterface<'d, Running, W, H> {
    /// Stops the display thread. All used pins will be reset to their default state and any information regarding the colors of the display will be lost.
    ///
    /// The display will only stop after it completes its current cycle. So it is possible it stops `1/refresh` seconds after it has been told to stop.
    ///
    /// The pin configuration however will be remembered.
    ///
    /// This is meant to be used when the display is no longer needed, and will be called automatically when the `DisplayInterface` instance is dropped.
    ///
    /// # Panics
    ///
    /// Panics if no `Sender` or no thread `JoinHandle` is present or if sending the stop message failed. (Neither of these should happen. If it does it is a fault inside of the crate.)
    pub fn stop(self) -> DisplayInterface<'d, Stopped, W, H> {
        match self.tx {
            Some(tx) => tx.send(Message::Stop).expect("Failed to send message"),
            None => panic!("State machine broke: no sender found"),
        };

        match self.handle {
            Some(handle) => handle.join().unwrap(),
            None => panic!("State machine broke: no thread handle found"),
        }

        DisplayInterface::<'d, Stopped, W, H> {
            handle: None,
            tx: None,
            id: self.id,
            state: PhantomData,
        }
    }

    /// Pause the display thread. The display will no longer update but all data regarding its color and io pins state will remain.
    ///
    /// # Panics
    ///
    /// Panics if no `Sender` is present or if sending the pause message failed. (Neither of these should happen. If it does it is a fault inside of the crate.)
    pub fn pause(self) -> DisplayInterface<'d, Paused, W, H> {
        match &self.tx {
            Some(tx) => tx.send(Message::Pause).expect("Failed to send message"),
            None => panic!("State machine broke: no thread handle found"),
        }
        DisplayInterface::<'d, Paused, W, H> {
            handle: self.handle,
            tx: self.tx,
            id: self.id,
            state: PhantomData,
        }
    }

    /// Update the color of one, multiple or all the leds.
    ///
    /// # Errors
    ///
    /// Returns a `c4_display::error::Error::InvalidDim` if the dimensions are out of bounds in the case of `SyncType::Single` or `SyncType::Multi`.
    /// In The case of `SyncType::All` this error is returned if the length of any of the vectors do not match
    ///
    /// Returns a `c4_display::error::Error::InvalidDim` if the length of the vectors do not match the provided width and height in the case of `SyncType::All`.
    ///
    /// # Panics
    ///
    /// Panics if no `Sender` is present or if the sync data failed to send. (Neither of this should happen. If it does it is a fault inside of the crate.)
    pub fn sync(&mut self, sync_type: SyncType) -> error::DisplayResult<()> {
        match &sync_type {
            SyncType::Single(sync) => {
                if sync.x >= W || sync.y >= H {
                    return Err(error::Error::InvalidDim);
                }
            }
            SyncType::Multi(sync_vec) => {
                for sync in sync_vec {
                    if sync.x >= W || sync.y >= H {
                        return Err(error::Error::InvalidDim);
                    }
                }
            }
            SyncType::All(board) => {
                if board.len() != H {
                    return Err(error::Error::InvalidDim);
                }
                for h in board {
                    if h.len() != W {
                        return Err(error::Error::InvalidDim);
                    }
                }
            }
        }
        match &self.tx {
            Some(tx) => tx
                .send(Message::Sync(sync_type))
                .expect("Failed to send message"),
            None => panic!("State machine broke: no thread handle found"),
        }
        Ok(())
    }
}

impl<'d, const W: usize, const H: usize> DisplayInterface<'d, Paused, W, H> {
    /// Resume the display thread.
    ///
    /// # Panics
    ///
    /// Panics if no thread `JoinHandle` is present. (This should not happen. If it does it is a fault inside of the crate.)
    pub fn resume(self) -> DisplayInterface<'d, Running, W, H> {
        match &self.handle {
            Some(handle) => handle.thread().unpark(),
            None => panic!("No thread handle"),
        }

        DisplayInterface::<'d, Running, W, H> {
            handle: self.handle,
            tx: self.tx,
            id: self.id,
            state: PhantomData,
        }
    }
}

impl<'d, S: State, const W: usize, const H: usize> DisplayInterface<'d, S, W, H> {
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

    /// Returns the width and height of the display. The witdh is stored in the first place and height in the second.
    pub fn get_dim(&self) -> (usize, usize) {
        (W, H)
    }
}
