// // #![allow(dead_code)]
//! main library

#![warn(missing_docs)]
use std::time::{Duration, Instant};
mod display;
pub use display::{DisplayInterface, LedColor, Paused, Running, State, Stopped, Sync, SyncType};

/// Placeholder for rppal's OutputPin type
#[derive(Debug)]
pub(self) struct OutputPinPlaceholder;

/// Placeholder for rppal's Level type
#[allow(dead_code)]
#[derive(Debug)]
pub(self) enum LevelPlaceholder {
    Low,
    High,
}

#[allow(dead_code)]
pub(self) mod pins {
    pub type SerinPinNr = u8;
    pub type SrclkPinNr = u8;
    pub type RclkPinNr = u8;
    pub type SrclrPinNr = u8;
    pub type OePinNr = u8;
    pub type A0PinNr = u8;
    pub type A1PinNr = u8;
    pub type A2PinNr = u8;
    pub type PinInitError = String;
}

/// GPIO pin numbers to use for shift registers and decoders.
///
/// Pins starting with sr_ are used by the shift register, whereas pins starting with dec_ are used by to the decoder.
#[derive(Debug)]
pub struct PinConfig {
    /// Serial input pin of the shift register
    pub sr_serin: pins::SerinPinNr, // shift register serial input

    /// Serial clock pin of the shift register
    pub sr_srclk: pins::SrclkPinNr, // shift register serial clock

    /// Register clock pin of the shift register
    pub sr_rclk: pins::RclkPinNr, // shift register register clock

    /// Serial clear pin of the shift register
    pub sr_srclr: pins::SrclrPinNr, // shift register serial clear

    /// Output enable pin of the shift register
    pub sr_oe: pins::OePinNr, // shift register output enable

    /// First decoder bit. This is the least significant bit, equivalent to 1.
    pub dec_a0: pins::A0PinNr, // decoder pin 0

    /// Second decoder bit. This is the second least significant bit, equivalent to 2.
    pub dec_a1: pins::A1PinNr, // decoder pin 1

    /// Third decoder bit. This is the most significant bit, equivalent to 4.
    pub dec_a2: pins::A2PinNr, // decoder pin 2
}

#[inline]
// pub fn spin_wait(dur: Duration) {
/// Wait for the given duration `dur`
pub(crate) fn spin_wait(dur: Duration) {
    let t = Instant::now();
    while t.elapsed() < dur {
        std::hint::spin_loop();
    }
}

impl OutputPinPlaceholder {
    fn set_low(&mut self) {}

    fn set_high(&mut self) {}

    #[allow(unused_variables)]
    fn write(&mut self, level: LevelPlaceholder) {}
}

/// Stops code execution until an enter is received from `stdin`.
///
/// Can be passed a string that will be logged with debug level.
#[macro_export]
macro_rules! breakpoint {
    () => {
        #[cfg(feature = "breakpoints")]
        {
            use std::io::{stdin, stdout, Write};

            log::debug!("breakpoint!");
            stdout().flush().unwrap();
            stdin().read_line(&mut String::new()).unwrap();
        }
    };
    ($($arg:tt)*) => {
        #[cfg(feature = "breakpoints")]
        {
            use std::io::{stdin, stdout, Write};

            log::debug!("breakpoint!\t{}", format_args!($($arg)*));
            stdout().flush().unwrap();
            stdin().read_line(&mut String::new()).unwrap();
        }
    };
}
