use crate::model::key::Key;
use std::fmt;

/// Why the keyboard could not be taken.
#[derive(Debug)]
pub struct InputError {
    pub reason: String,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl std::error::Error for InputError {}

/// The keyboard the user types at.
pub trait Input {
    /// Takes the keyboard. Every press until the returned value is dropped reaches longcut and
    /// nothing else.
    ///
    /// # Errors
    ///
    /// Returns an error if the keyboard cannot be taken, for instance because another program
    /// holds it.
    fn take_keyboard(&self) -> Result<Box<dyn Keyboard + '_>, InputError>;
}

/// The keyboard while it is taken. Dropping it gives the keyboard back.
pub trait Keyboard {
    /// Block until the next key press.
    fn capture_any(&self) -> Key;
}
