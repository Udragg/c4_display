use std::time::Duration;

use super::LedColor;
use crate::pins::{OePinNr, PinInitError, RclkPinNr, SerinPinNr, SrclkPinNr, SrclrPinNr};
use crate::{spin_wait, OutputPinPlaceholder};

#[derive(Debug)]
#[allow(dead_code)]
pub(super) struct ShiftReg {
    serin: OutputPinPlaceholder, // active high
    srclk: OutputPinPlaceholder, // active high
    rclk: OutputPinPlaceholder,  // active high
    srclr: OutputPinPlaceholder, // active high
    oe: OutputPinPlaceholder,    // active low
    // size: usize, //? size unnecessary, handled in Display::run_once()
    buffer: Vec<LedColor>,
}

impl ShiftReg {
    /// create new shift register instance
    ///
    /// pin order:
    ///
    /// 1: SerinPinNr (u8)
    ///
    /// 2: SrclkPinNr (u8)
    ///
    /// 3: RclkPinNr (u8)
    ///
    /// 4: SrclrPinNr (u8)
    ///
    /// 5: OePinNr (u8)
    pub(super) fn new(
        pins: (SerinPinNr, SrclkPinNr, RclkPinNr, SrclrPinNr, OePinNr),
        size: usize,
    ) -> Result<Self, PinInitError> {
        // TODO replace PinInitError with rppal::gpio::Error
        Ok(Self {
            serin: OutputPinPlaceholder,
            srclk: OutputPinPlaceholder,
            rclk: OutputPinPlaceholder,
            srclr: OutputPinPlaceholder,
            oe: OutputPinPlaceholder,
            // size,
            // buffer: Vec::with_capacity(size),
            buffer: vec![LedColor::Off; size],
        })
    }

    pub(super) fn enable(&mut self) {
        self.oe.set_low();
        spin_wait(Duration::from_micros(1));
    }

    pub(super) fn disable(&mut self) {
        self.oe.set_high();
        spin_wait(Duration::from_micros(1));
    }

    /// Push the input register to the output register
    ///
    /// This function takes at least 2 microseconds
    pub(super) fn push(&mut self) {
        self.rclk.set_high();
        spin_wait(Duration::from_micros(1));
        self.rclk.set_low();
        spin_wait(Duration::from_micros(1));
    }

    /// Shift a [LedColor] into the shift register.
    ///
    /// This function takes at least 6 microseconds.
    pub(super) fn shift_color(&mut self, color: &LedColor) {
        for c_bit in 0..3 {
            self.shift((*color as usize >> c_bit & 1) != 0);
        }
    }

    /// Shift one bit into the shift register.
    ///
    /// This function takes at least 2 microseconds.
    fn shift(&mut self, bit: bool) {
        match bit {
            true => {
                self.serin.set_high();
                spin_wait(Duration::from_micros(1));
                self.srclk.set_high();
                spin_wait(Duration::from_micros(1));
                self.srclk.set_low();
            }
            false => {
                self.serin.set_low();
                spin_wait(Duration::from_micros(1));
                self.srclk.set_high();
                spin_wait(Duration::from_micros(1));
                self.srclk.set_low();
            }
        }
    }
}
