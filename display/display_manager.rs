use crate::{breakpoint_sbs, display::Display, display::Message};
use std::{
    sync::mpsc::{Receiver, TryRecvError},
    thread,
};

pub(super) struct DisplayManager<const W: usize, const H: usize> {
    disp: Display<W, H>,
    rx: Receiver<Message>,
}

impl<const W: usize, const H: usize> DisplayManager<W, H> {
    /// Create a new `DisplayManager` with the given `Display` and `Receiver`.
    pub(super) fn new(disp: Display<W, H>, rx: Receiver<Message>) -> Self {
        Self { disp, rx }
    }

    /// Start the display.
    pub(super) fn start(&mut self) {
        loop {
            match self.rx.try_recv() {
                Ok(msg) => match msg {
                    Message::Pause => {
                        thread::park();
                        continue;
                    }
                    Message::Stop => break,
                    Message::Sync(sync_type) => self.disp.sync(sync_type),
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    log::error!("Display interface disconnected. Stopping thread...");
                    break;
                }
            }

            self.disp.run_once();
            breakpoint_sbs!();
        }
    }
}
