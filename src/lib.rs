// // #![allow(dead_code)]
//! Library to more easily drive the led matrix.
// TODO add logging
// TODO ability to change dimming

#![warn(missing_docs)]
use std::time::{Duration, Instant};
mod display;
mod error;

// Crate API exports
pub use display::{
    Animation, AnimationFrame, BlinkInfo, DisplayInterface, LedColor, LedState, Paused, Rotation,
    Running, State, Stopped, Sync, SyncType,
};
pub use error::{DisplayResult, Error};

/// Time for gpio pins to switch state
const PSWT: std::time::Duration = std::time::Duration::from_nanos(100);

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
    pub type LEPinNr = u8;
    pub type E1PinNr = u8;
}

/// GPIO pin numbers to use for shift registers and decoders.
///
/// Pins starting with sr_ are used by the shift register,
/// whereas pins starting with dec_ are used by to the decoder.
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

    /// Output enable pin of the shift register (active low)
    pub sr_oe: pins::OePinNr, // shift register output enable (active low)

    /// First decoder bit. This is the least significant bit, equivalent to 1.
    pub dec_a0: pins::A0PinNr, // decoder pin 0

    /// Second decoder bit. This is the second least significant bit, equivalent to 2.
    pub dec_a1: pins::A1PinNr, // decoder pin 1

    /// Third decoder bit. This is the most significant bit, equivalent to 4.
    pub dec_a2: pins::A2PinNr, // decoder pin 2

    /// Decoder Latch Enable.
    /// If enabled the changes to the input of the decoder will nog affect the output.
    pub dec_le: pins::LEPinNr,

    /// Decoder Output Enable. (active low)
    /// If enabled the decoder outputs will all be low.
    pub dec_e1: pins::E1PinNr, // decoder output enable (active low)
}

#[inline]
/// Wait for the given duration `dur`
pub fn spin_wait(dur: Duration) {
    let t = Instant::now();
    while t.elapsed() < dur {
        std::hint::spin_loop();
    }
    // std::thread::sleep(dur);
}

// /// Stops code execution until an enter is received from `stdin`.
// ///
// /// Can be passed a string that will be logged with debug level.
// #[macro_export]
// macro_rules! breakpoint {
//     () => {
//         #[cfg(feature = "breakpoints")]
//         {
//             use std::io::{stdin, stdout, Write};

//             log::debug!("breakpoint!");
//             stdout().flush().unwrap();
//             stdin().read_line(&mut String::new()).unwrap();
//         }
//     };
//     ($($arg:tt)*) => {
//         #[cfg(feature = "breakpoints")]
//         {
//             use std::io::{stdin, stdout, Write};

//             log::debug!("breakpoint!\t{}", format_args!($($arg)*));
//             stdout().flush().unwrap();
//             stdin().read_line(&mut String::new()).unwrap();
//         }
//     };
// }

// /// Stops code execution until an enter is received from `stdin`.
// ///
// /// Can be passed a string that will be logged with debug level.
// #[macro_export]
// macro_rules! breakpoint_sbs {
//     () => {
//         #[cfg(feature = "sbs_debug")]
//         {
//             use std::io::{stdin, stdout, Write};

//             log::debug!("breakpoint!");
//             stdout().flush().unwrap();
//             stdin().read_line(&mut String::new()).unwrap();
//         }
//     };
//     ($($arg:tt)*) => {
//         #[cfg(feature = "sbs_debug")]
//         {
//             use std::io::{stdin, stdout, Write};

//             log::debug!("breakpoint!\t{}", format_args!($($arg)*));
//             stdout().flush().unwrap();
//             stdin().read_line(&mut String::new()).unwrap();
//         }
//     };
// }
