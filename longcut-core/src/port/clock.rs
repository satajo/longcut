use std::time::Instant;

/// The clock the session runs by.
pub trait Clock {
    /// The current moment.
    fn now(&self) -> Instant;
}
