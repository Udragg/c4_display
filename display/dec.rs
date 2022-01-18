use std::time::Duration;

use rppal::gpio::{Gpio, Level, OutputPin};

use crate::{
    error,
    pins::{A0PinNr, A1PinNr, A2PinNr, LEPinNr},
    spin_wait,
};

#[derive(Debug)]
pub(super) struct Dec {
    // a: [OutputPin; 3],
    a0: OutputPin,
    a1: OutputPin,
    a2: OutputPin,
    le: OutputPin,
    output: DecOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecOutput {
    Y0 = 0,
    Y1 = 1,
    Y2 = 2,
    Y3 = 3,
    Y4 = 4,
    Y5 = 5,
    Y6 = 6,
    Y7 = 7,
}

// TODO make use of LE pin
impl Dec {
    pub(super) fn new(pins: (A0PinNr, A1PinNr, A2PinNr, LEPinNr)) -> error::DisplayResult<Self> {
        let mut dec = Self {
            // a: [
            //     Gpio::new()?.get(pins.0)?.into_output_low(),
            //     Gpio::new()?.get(pins.1)?.into_output_low(),
            //     Gpio::new()?.get(pins.2)?.into_output_low(),
            // ],
            a0: Gpio::new()?.get(pins.0)?.into_output(),
            a1: Gpio::new()?.get(pins.1)?.into_output(),
            a2: Gpio::new()?.get(pins.2)?.into_output(),
            le: Gpio::new()?.get(pins.3)?.into_output(),
            output: DecOutput::default(),
        };

        dec.a0.set_low();
        dec.a1.set_low();
        dec.a2.set_low();
        dec.le.set_low();

        Ok(dec)
    }

    /// Update the decoder output.
    ///
    /// This function takes at least 1 microsecond
    fn update(&mut self) {
        self.a0.write(match self.output as u8 & 0b1 {
            0 => Level::Low,
            1 => Level::High,
            _ => unreachable!(),
        });
        self.a1.write(match (self.output as u8 >> 1) & 0b1 {
            0 => Level::Low,
            1 => Level::High,
            _ => unreachable!(),
        });
        self.a2.write(match (self.output as u8 >> 2) & 0b1 {
            0 => Level::Low,
            1 => Level::High,
            _ => unreachable!(),
        });

        // for b in 0..3 {
        //     match self.output as usize >> b & 1 {
        //         0 => self.a[b].set_low(),
        //         1 => self.a[b].set_high(),
        //         _ => unreachable!(),
        //     }
        // }

        spin_wait(Duration::from_micros(1));
    }

    /// Set the decoder output to a specific position.
    ///
    /// This function takes at least 1 microsecond.
    pub(super) fn set(&mut self, num: usize) {
        self.output = DecOutput::from(num);
        self.update();
    }

    /// Lock the decoder output.
    ///
    /// This function takes at least 1 microsecond.
    pub(super) fn latch_on(&mut self) {
        self.le.set_high();
        spin_wait(Duration::from_micros(1));
    }

    /// Unlock the decoder output.
    ///
    /// This function takes at least 1 microsecond.
    pub(super) fn latch_off(&mut self) {
        self.le.set_low();
        spin_wait(Duration::from_micros(1));
    }
}

impl std::ops::AddAssign<usize> for Dec {
    fn add_assign(&mut self, rhs: usize) {
        self.output += rhs;
        self.update();
    }
}

impl std::ops::SubAssign<usize> for Dec {
    fn sub_assign(&mut self, rhs: usize) {
        self.output -= rhs;
        self.update();
    }
}

impl From<usize> for DecOutput {
    fn from(num: usize) -> Self {
        match num.clamp(0, 7) {
            0 => DecOutput::Y0,
            1 => DecOutput::Y1,
            2 => DecOutput::Y2,
            3 => DecOutput::Y3,
            4 => DecOutput::Y4,
            5 => DecOutput::Y5,
            6 => DecOutput::Y6,
            7 => DecOutput::Y7,
            _ => unimplemented!(),
        }
    }
}

impl std::ops::Add<usize> for DecOutput {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        let member_arr = [
            Self::Y0,
            Self::Y1,
            Self::Y2,
            Self::Y3,
            Self::Y4,
            Self::Y5,
            Self::Y6,
            Self::Y7,
        ];
        member_arr[(self as usize + rhs) % 8]
    }
}

impl std::ops::AddAssign<usize> for DecOutput {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl std::ops::Sub<usize> for DecOutput {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        let member_arr = [
            Self::Y0,
            Self::Y1,
            Self::Y2,
            Self::Y3,
            Self::Y4,
            Self::Y5,
            Self::Y6,
            Self::Y7,
        ];
        member_arr[(((self as isize - rhs as isize) % 8) + 8) as usize % 8] // convert to positive valid index
    }
}

impl std::ops::SubAssign<usize> for DecOutput {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

impl Default for DecOutput {
    fn default() -> Self {
        Self::Y0
    }
}

mod test_add_sub {
    #[allow(unused_imports)]
    use super::DecOutput;

    #[test]
    fn add_1() {
        assert_eq!(DecOutput::Y0 + 1, DecOutput::Y1);
    }

    #[test]
    fn add_1_overflow() {
        assert_eq!(DecOutput::Y7 + 1, DecOutput::Y0);
    }

    #[test]
    fn add_3() {
        assert_eq!(DecOutput::Y0 + 3, DecOutput::Y3);
    }

    #[test]
    fn add_3_overflow() {
        assert_eq!(DecOutput::Y6 + 3, DecOutput::Y1);
    }

    #[test]
    fn add_10_loopback() {
        assert_eq!(DecOutput::Y0 + 10, DecOutput::Y2);
    }

    #[test]
    fn add_10_double_overflow() {
        assert_eq!(DecOutput::Y6 + 10, DecOutput::Y0);
    }

    #[test]
    fn sub_1() {
        assert_eq!(DecOutput::Y7 - 1, DecOutput::Y6);
    }

    #[test]
    fn sub_1_underflow() {
        assert_eq!(DecOutput::Y0 - 1, DecOutput::Y7);
    }

    #[test]
    fn sub_3() {
        assert_eq!(DecOutput::Y7 - 3, DecOutput::Y4);
    }

    #[test]
    fn sub_3_underflow() {
        assert_eq!(DecOutput::Y7 - 3, DecOutput::Y4);
    }

    #[test]
    fn sub_10_loopback() {
        assert_eq!(DecOutput::Y7 - 10, DecOutput::Y5);
    }

    #[test]
    fn sub_10_double_underflow() {
        assert_eq!(DecOutput::Y1 - 10, DecOutput::Y7);
    }
}
