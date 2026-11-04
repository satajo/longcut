use std::time::{Duration, Instant};

/// The clock the session runs by.
pub trait Clock {
    /// The current moment.
    fn now(&self) -> Instant;

    /// Pauses the session for the duration. Nothing else happens meanwhile.
    fn sleep(&self, duration: Duration);
}
