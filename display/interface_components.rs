use crate::display::LedColor;

/// The types of message that can be sent to the display thread.
#[derive(Debug)]
pub(super) enum Message {
    Stop,
    Pause,
    Sync(SyncType),
}

/// Indicates the current state of the `DisplayInterface`.
pub trait State {}

/// The running state of `DisplayInterface`.
#[doc(hidden)]
pub struct Running;
impl State for Running {}

/// The paused state of `DisplayInterface`.
#[doc(hidden)]
pub struct Paused;
impl State for Paused {}

/// The stopped state of `DisplayInterface`.
#[doc(hidden)]
pub struct Stopped;
impl State for Stopped {}

/// Data struct to change a led's color.
#[derive(Debug)]
pub struct Sync {
    /// The x position of the led to be changed.
    pub x: usize,
    /// The y position of the led to be changed.
    pub y: usize,
    /// The new color the led should have.
    pub color: LedColor,
}

/// The amount to rotate.
#[derive(Debug)]
pub enum Rotation {
    /// Rotate 90° clockwise.
    Clockwise,
    /// Rotate 90° counterclockwise.
    CounterClockwise,
    /// Rotate 180°.
    OneEighty,
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
    /// Change the color of one led.
    Single(Sync),
    /// Change the color of a vector of leds.
    Multi(Vec<Sync>),
    /// Change the color of all leds.
    All(Vec<Vec<LedColor>>),
    /// Rotate the entire grid.
    Rotate(Rotation),
}
