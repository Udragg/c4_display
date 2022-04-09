// use rppal::{gpio, gpio::Gpio, gpio::OutputPin};
use crate::{
    display::{Dec, Rotation, ShiftReg},
    error, spin_wait, PinConfig, Sync, SyncType,
};
use std::{
    str::FromStr,
    time::{Duration, Instant, SystemTime},
};

#[derive(Debug)]
#[allow(dead_code)]
pub(super) struct Display<const W: usize, const H: usize> {
    row: ShiftReg,
    column: Dec,
    display: [[LedState; W]; H],
    // global_dim: f64, // global pwm
    tpl: Duration, // time per led in seconds, based on refresh rate
}

/// Colors that can be displayed
// #[allow(dead_code)]
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
    /// The color cyan.
    Cyan = 6,
    /// The color purple.
    Magenta = 5,
    /// The color white.
    White = 7,
}

// ! this is a very crude solution to handeling animations
// ! it's only meant as a quick way to implement blinking
/// Blink duration and interval.
#[derive(Debug, Clone, Copy)]
pub struct BlinkInfo {
    /// The time the led is on. PWM equivalent: ton
    pub dur: Duration,
    /// The time of on blink period. PWM equivalent: t
    pub int: Duration,
}

/// Led state, contains color, blink duration and blink interval.
#[derive(Debug, Clone, Copy)]
pub struct LedState {
    /// The color of the led.
    pub color: LedColor,
    /// The blink information of the led.
    pub blink: Option<BlinkInfo>,
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
            column: Dec::new((
                pins.dec_a0,
                pins.dec_a1,
                pins.dec_a2,
                pins.dec_le,
                pins.dec_e1,
            ))?,
            display: [[LedState::default(); W]; H],
            tpl,
        };

        Ok(disp)
    }

    /// Iterate over the entire display once.
    pub(super) fn run_once(&mut self, start_time: Instant) {
        #[cfg(feature = "disp_debug")]
        log::debug!("Starting run");
        for (c_index, row) in self.display.iter().enumerate() {
            self.row.clear(); // empty the shift registers

            // shift everything into the register
            for led in row {
                let now = SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_micros();

                // blink led
                self.row.shift_color(match led.blink {
                    Some(blink) if now % blink.int.as_micros() > blink.dur.as_micros() => {
                        &LedColor::Off
                    }
                    _ => &led.color,
                });

                // adaptive sleep
                // let acc_wait_time =
                //     self.tpl * (r_index + 1) as u32 + (self.tpl * (c_index * W) as u32);
                // spin_wait(acc_wait_time - start_time.elapsed().min(acc_wait_time));
            }

            // disable row during switching to prevent unwanted leds from turning on
            self.row.disable();
            // lock column output
            self.column.latch_on();
            // set column
            self.column.set(c_index);
            // unlock column output
            self.column.latch_off();
            // update register
            self.row.push();
            // enable row
            self.row.enable();

            let wait_time = self.tpl * W as u32 * (c_index + 1) as u32; //? W or H?
            let subbed_wait_time = wait_time
                .checked_sub(start_time.elapsed())
                .unwrap_or(Duration::ZERO);
            #[cfg(feature = "disp_debug")]
            log::debug!("{wait_time:?}, {subbed_wait_time:?}");
            spin_wait(subbed_wait_time);
        }
    }

    /// Update the colors of the leds.
    pub(super) fn sync(&mut self, sync_type: SyncType) {
        match sync_type {
            SyncType::Single(sync) => {
                let Sync { x, y, state } = sync;
                match state.blink {
                    Some(blink) if blink.dur > blink.int => panic!(
                        "Blink duration larger than blink interval\nduration: {:?}, interval: {:?}",
                        blink.dur, blink.int
                    ),
                    _ => self.display[y][x] = state,
                }
            }
            SyncType::Multi(sync_vec) => {
                for sync in sync_vec {
                    let Sync { x, y, state } = sync;
                    match state.blink {
                        Some(blink) if blink.dur > blink.int => panic!(
                            "Blink duration larger than blink interval\nduration: {:?}, interval: {:?}",
                            blink.dur, blink.int
                        ),
                        _ => self.display[y][x] = state,
                    }
                }
            }
            SyncType::All(board) => {
                assert_eq!(H, board.len()); // panic if the dimensions are unexpected
                for (y, height) in board.iter().enumerate() {
                    assert_eq!(W, height.len()); // panic if the dimensions are unexpected
                    for (x, led) in height.iter().enumerate() {
                        match led.blink {
                            Some(blink) if blink.dur > blink.int => panic!(
                                "Blink duration larger than blink interval\nduration: {:?}, interval: {:?}",
                                blink.dur, blink.int
                            ),
                            _ => self.display[y][x] = *led,
                        }
                    }
                }
            }
            SyncType::Rotate(r) => match r {
                Rotation::Clockwise => {
                    let center = ((W - 1) as f64 / 2., (H - 1) as f64 / 2.);
                    let mut disp_rotated = [[LedState::default(); W]; H];
                    for (y, r) in self.display.iter().enumerate() {
                        for (x, l) in r.iter().enumerate() {
                            // clockwise rotation
                            // x => -y
                            // y => x
                            let x_new = -(y as f64 - center.1) + center.0;
                            let y_new = x as f64 - center.0 + center.1;
                            disp_rotated[y_new as usize][x_new as usize] = *l;
                        }
                    }
                    self.display = disp_rotated;
                }
                Rotation::CounterClockwise => {
                    let center = ((W - 1) as f64 / 2., (H - 1) as f64 / 2.);
                    let mut disp_rotated = [[LedState::default(); W]; H];
                    for (y, r) in self.display.iter().enumerate() {
                        for (x, l) in r.iter().enumerate() {
                            // counterclockwise rotation
                            // x => y
                            // y => -x
                            let x_new = y as f64 - center.1 + center.0;
                            let y_new = -(x as f64 - center.0) + center.1;
                            disp_rotated[y_new as usize][x_new as usize] = *l;
                        }
                    }
                    self.display = disp_rotated;
                }
                Rotation::OneEighty => {
                    // TODO improve with swap() and ranges 0..W/2   0..H/2
                    let center = ((W - 1) as f64 / 2., (H - 1) as f64 / 2.);
                    let mut disp_rotated = [[LedState::default(); W]; H];
                    for (y, r) in self.display.iter().enumerate() {
                        for (x, l) in r.iter().enumerate() {
                            // 180° rotation
                            // x => -y
                            // y => -x
                            let x_new = -(x as f64 - center.0) + center.0;
                            let y_new = -(y as f64 - center.1) + center.1;
                            disp_rotated[y_new as usize][x_new as usize] = *l;
                        }
                    }
                    self.display = disp_rotated;
                }
            },
        }
    }

    pub(super) fn clear_row(&mut self) {
        self.row.clear();
        self.row.push();
    }
}

impl Default for LedColor {
    fn default() -> Self {
        Self::Off
    }
}

impl FromStr for LedColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "off" | "black" => Ok(Self::Off),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "yellow" => Ok(Self::Yellow),
            "blue" => Ok(Self::Blue),
            "magenta" => Ok(Self::Magenta),
            "cyan" => Ok(Self::Cyan),
            "white" => Ok(Self::White),
            _ => Err("Could not parse string".to_string()),
        }
    }
}

impl Default for LedState {
    fn default() -> Self {
        Self {
            color: LedColor::default(),
            blink: None,
        }
    }
}

impl LedState {
    /// Create a new [LedState](self) with the given color and default blink.
    pub fn with_color(color: LedColor) -> Self {
        Self { color, blink: None }
    }
}
