// use rppal::{gpio, gpio::Gpio, gpio::OutputPin};
use crate::{
    display::{Dec, ShiftReg},
    spin_wait,
};
use std::time::{Duration, Instant};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Display<const W: usize, const H: usize> {
    row: ShiftReg,
    column: Dec,
    display: [[LedColor; H]; W],
    // refresh: usize,  // in Hz
    // global_dim: f64, // global pwm
    tpl: Duration, // time per led in seconds, based on refresh rate
}

/// Colors that can be displayed
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum LedColor {
    /// No color. This is also the default.
    Off = 0,
    /// The color red.
    Red = 1,
    /// The color green.
    Green = 2,
    /// The color yellow.
    Yellow = 3,
    /// The color blue.
    Blue = 4,
    /// The color purple.
    Purple = 5,
    /// The color cyan.
    Cyan = 6,
    /// The color white.
    White = 7,
}

impl<const W: usize, const H: usize> Display<W, H> {
    /// set up a new display instance
    //? leave out init and create new instance directly where needed with Display {...} ?
    pub fn init(refresh: f64, p: crate::PinConfig) -> Result<Self, String> {
        let tpl = Duration::from_secs_f64(1.0 / (refresh * W as f64 * H as f64));

        let disp = Self {
            row: ShiftReg::new((p.sr_serin, p.sr_srclk, p.sr_rclk, p.sr_srclr, p.sr_oe), 7)?,
            column: Dec::new((p.dec_a0, p.dec_a1, p.dec_a2))?,
            display: [[LedColor::default(); H]; W],
            tpl,
        };
        #[cfg(feature = "disp_debug")]
        log::debug!("time per led: {}", tpl.as_secs_f64());

        Ok(disp)
    }

    pub fn run_once(&mut self) {
        #[cfg(feature = "disp_debug")]
        log::debug!("Starting run");
        let start_time = Instant::now();
        self.column.set(0);
        for (c_index, c) in self.display.iter().enumerate() {
            for (r_index, r) in c.iter().enumerate() {
                self.row.shift_color(r);

                let acc_wait_time =
                    self.tpl * (r_index + 1) as u32 + (self.tpl * (c_index * W) as u32);
                spin_wait(acc_wait_time - start_time.elapsed().min(acc_wait_time));
                // adaptive sleep
            }
            self.row.disable(); // disable row during switching to prevent unwanted leds from turning on
            self.column += 1;
            self.row.push();
            self.row.enable();
        }
    }
}

impl Default for LedColor {
    fn default() -> Self {
        Self::Off
    }
}
