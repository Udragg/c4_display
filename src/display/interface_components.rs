use crate::display::LedColor;

#[derive(Debug)]
pub(super) enum Message {
    Stop,
    Pause,
    // Resume,
    Sync(SyncType),
}

/// Indicates the current state of the `DisplayInterface`
pub trait State {}

/// The running state of `DisplayInterface`
pub struct Running;
impl State for Running {}

/// The paused state of `DisplayInterface`
pub struct Paused;
impl State for Paused {}

/// The stopped state of `DisplayInterface`
pub struct Stopped;
impl State for Stopped {}

/// Data struct to change a led's color
#[derive(Debug)]
pub struct Sync {
    /// x position
    pub x: usize,
    /// y position
    pub y: usize,
    /// new color
    pub color: LedColor,
}

//? additional board manipulation options (rotate, shift, ...)
/// The synchronization type. Synchronization is used to update which led has which color.
///
/// Use [SyncType::Single] to change the color of one led.
///
/// Use [SyncType::Multi] to change the color of multiple leds.
///
/// Use [SyncType::All] to change the color of all leds at once.
#[derive(Debug)]
pub enum SyncType {
    /// Can change one led's color at a time.
    Single(Sync),
    /// Can change multiple led's colors at a time.
    Multi(Vec<Sync>),
    /// Can change all led colors at the same time.
    All(Vec<Vec<LedColor>>),
}

// #[derive(Debug, Clone, Copy)]
// pub enum State {
//     Running,
//     Paused,
//     Stopped,
// }
