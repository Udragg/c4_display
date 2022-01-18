/// Types of error
#[derive(Debug)]
pub enum Error {
    /// The provided dimensions do not match or exceed the dimensions of the display.
    InvalidDim,
    /// GPIO error return by rppal.
    // Gpio(rppal::gpio::error),
    Gpio(rppal::gpio::Error),
}

/// Result used by functions in this crate.
pub type DisplayResult<T> = Result<T, Error>;

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<rppal::gpio::Error> for Error {
    fn from(e: rppal::gpio::Error) -> Self {
        Self::Gpio(e)
    }
}
