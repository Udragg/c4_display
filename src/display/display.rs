// use rppal::{gpio, gpio::Gpio, gpio::OutputPin};
use crate::{
    breakpoint_sbs,
    display::{Dec, Rotation, ShiftReg},
    error, spin_wait, PinConfig, Sync, SyncType,
};
use std::time::{Duration, Instant};

#[derive(Debug)]
#[allow(dead_code)]
pub(super) struct Display<const W: usize, const H: usize> {
    row: ShiftReg,
    column: Dec,
    display: [[LedColor; W]; H],
    // global_dim: f64, // global pwm
    tpl: Duration, // time per led in seconds, based on refresh rate
}

/// Colors that can be displayed
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum LedColor {
    /// No color. This is also the default.
    Off = 0,
    /// The color blue.
    Blue = 1,
    /// The color green.
    Green = 2,
    /// The color cyan.
    Cyan = 3,
    /// The color red.
    Red = 4,
    /// The color purple.
    Magenta = 5,
    /// The color yellow.
    Yellow = 6,
    /// The color white.
    White = 7,
}

impl<const W: usize, const H: usize> Display<W, H> {
    /// Set up a new display instance.
    pub(super) fn init(refresh: f64, pins: PinConfig) -> error::DisplayResult<Self> {
        let tpl = Duration::from_secs_f64(1.0 / (refresh * W as f64 * H as f64));
        #[cfg(feature = "disp_debug")]
        log::debug!("time per led: {}", tpl.as_secs_f64());

        let disp = Self {
            row: ShiftReg::new((
                pins.sr_serin,
                pins.sr_srclk,
                pins.sr_rclk,
                pins.sr_srclr,
                pins.sr_oe,
            ))?,
            column: Dec::new((pins.dec_a0, pins.dec_a1, pins.dec_a2, pins.dec_le))?,
            display: [[LedColor::default(); W]; H],
            tpl,
        };

        Ok(disp)
    }

    /// Iterate over the entire display once.
    pub(super) fn run_once(&mut self) {
        #[cfg(feature = "disp_debug")]
        log::debug!("Starting run");
        breakpoint_sbs!();
        let start_time = Instant::now();
        for (c_index, c) in self.display.iter().enumerate() {
            self.row.clear(); // empty the shift registers
                              // shift everything into the register
            for (r_index, r) in c.iter().enumerate() {
                self.row.shift_color(r);

                let acc_wait_time =
                    self.tpl * (r_index + 1) as u32 + (self.tpl * (c_index * W) as u32);
                spin_wait(acc_wait_time - start_time.elapsed().min(acc_wait_time));
                // adaptive sleep
            }
            self.row.disable(); // disable row during switching to prevent unwanted leds from turning on
            self.column.latch_on(); // lock column output
            self.column.set(c_index); // set column
            self.column.latch_off(); // unlock column output
            self.row.push(); // update register
            self.row.enable(); // enable row
        }
    }

    /// Update the colors of the leds.
    pub(super) fn sync(&mut self, sync_type: SyncType) {
        match sync_type {
            SyncType::Single(sync) => self.display[sync.y][sync.x] = sync.color,
            SyncType::Multi(sync_vec) => {
                for sync in sync_vec {
                    let Sync { x, y, color } = sync;
                    self.display[y][x] = color
                }
            }
            SyncType::All(board) => {
                assert_eq!(H, board.len()); // panic if the dimensions are unexpected
                for (y, height) in board.iter().enumerate() {
                    assert_eq!(W, height.len()); // panic if the dimensions are unexpected
                    for (x, color) in height.iter().enumerate() {
                        self.display[y][x] = *color;
                    }
                }
            }
            SyncType::Rotate(r) => match r {
                Rotation::Clockwise => {
                    let center = ((W - 1) as f64 / 2., (H - 1) as f64 / 2.);
                    let mut disp_rotated = [[LedColor::default(); W]; H];
                    for (y, r) in self.display.iter().enumerate() {
                        for (x, c) in r.iter().enumerate() {
                            // clockwise rotation
                            // x => -y
                            // y => x
                            let x_new = -(y as f64 - center.1) + center.0;
                            let y_new = x as f64 - center.0 + center.1;
                            disp_rotated[y_new as usize][x_new as usize] = *c;
                        }
                    }
                    self.display = disp_rotated;
                }
                Rotation::CounterClockwise => {
                    let center = ((W - 1) as f64 / 2., (H - 1) as f64 / 2.);
                    let mut disp_rotated = [[LedColor::default(); W]; H];
                    for (y, r) in self.display.iter().enumerate() {
                        for (x, c) in r.iter().enumerate() {
                            // counterclockwise rotation
                            // x => y
                            // y => -x
                            let x_new = y as f64 - center.1 + center.0;
                            let y_new = -(x as f64 - center.0) + center.1;
                            disp_rotated[y_new as usize][x_new as usize] = *c;
                        }
                    }
                    self.display = disp_rotated;
                }
                Rotation::OneEighty => {
                    // TODO improve with swap() and ranges 0..W/2   0..H/2
                    let center = ((W - 1) as f64 / 2., (H - 1) as f64 / 2.);
                    let mut disp_rotated = [[LedColor::default(); W]; H];
                    for (y, r) in self.display.iter().enumerate() {
                        for (x, c) in r.iter().enumerate() {
                            // 180° rotation
                            // x => -y
                            // y => -x
                            let x_new = -(x as f64 - center.0) + center.0;
                            let y_new = -(y as f64 - center.1) + center.1;
                            disp_rotated[y_new as usize][x_new as usize] = *c;
                        }
                    }
                    self.display = disp_rotated;

                    // for c in 0..H / 2 {
                    //     for r in 0..W / 2 {
                    //         std::mem::swap(&mut self.display[c][r], &mut self.display[H - c][W - r]);
                    //     }
                    // }
                }
            },
        }
    }
}

impl Default for LedColor {
    fn default() -> Self {
        Self::Off
    }
}
