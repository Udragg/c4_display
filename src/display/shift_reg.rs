use rppal::gpio::{Gpio, OutputPin};
use std::time::Duration;

use super::LedColor;
use crate::pins::{OePinNr, RclkPinNr, SerinPinNr, SrclkPinNr, SrclrPinNr};
use crate::{error, spin_wait};

#[derive(Debug)]
#[allow(dead_code)]
pub(super) struct ShiftReg {
    /// Serial input pin. Active high.
    serin: OutputPin,
    /// Serial clock pin. Active high.
    srclk: OutputPin,
    /// Register clock pin. Active high.
    rclk: OutputPin,
    /// Serial clear pin. Active high.
    srclr: OutputPin,
    /// Output enable pin. Active low.
    oe: OutputPin,
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
    ) -> error::DisplayResult<Self> {
        let mut sr = Self {
            serin: Gpio::new()?.get(pins.0)?.into_output(),
            srclk: Gpio::new()?.get(pins.1)?.into_output(),
            rclk: Gpio::new()?.get(pins.2)?.into_output(),
            srclr: Gpio::new()?.get(pins.3)?.into_output(),
            oe: Gpio::new()?.get(pins.4)?.into_output(),
        }
        ._clear();
        sr.serin.set_low();
        sr.srclk.set_low();
        sr.rclk.set_low();
        sr.srclr.set_high();
        sr.oe.set_low();
        Ok(sr)
    }

    /// Enable the shift register
    ///
    /// This function takes at least 1 microsecond
    pub(super) fn enable(&mut self) {
        self.oe.set_low();
        spin_wait(Duration::from_micros(1));
    }

    /// Disable the shift register
    ///
    /// This function takes at least 1 microsecond
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
    /// This function takes at least 9 microseconds.
    pub(super) fn shift_color(&mut self, color: &LedColor) {
        for c_bit in 0..3 {
            self.shift((*color as usize >> c_bit & 1) != 0);
        }
    }

    /// Shift one bit into the shift register.
    ///
    /// This function takes at least 3 microseconds.
    fn shift(&mut self, bit: bool) {
        match bit {
            true => {
                self.serin.set_high();
                spin_wait(Duration::from_micros(1));
                self.srclk.set_high();
                spin_wait(Duration::from_micros(1));
                self.srclk.set_low();
                spin_wait(Duration::from_micros(1));
            }
            false => {
                self.serin.set_low();
                spin_wait(Duration::from_micros(1));
                self.srclk.set_high();
                spin_wait(Duration::from_micros(1));
                self.srclk.set_low();
                spin_wait(Duration::from_micros(1));
            }
        }
    }

    /// Clear the register
    ///
    /// This function takes at least 4 microseconds.
    pub(super) fn clear(&mut self) {
        self.srclr.set_low();
        spin_wait(Duration::from_micros(1));
        self.srclr.set_high();
        spin_wait(Duration::from_micros(1));
    }

    /// Clear the register
    ///
    /// This function takes at least 4 microseconds.
    fn _clear(mut self) -> Self {
        self.srclr.set_high();
        spin_wait(Duration::from_micros(1));
        self.srclr.set_low();
        spin_wait(Duration::from_micros(1));
        self.rclk.set_high();
        spin_wait(Duration::from_micros(1));
        self.rclk.set_low();
        spin_wait(Duration::from_micros(1));
        self
    }
}
