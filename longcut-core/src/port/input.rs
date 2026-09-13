use crate::model::key::Key;

pub trait Input {
    /// Block until the next key press on the already-grabbed keyboard. This only reads; the
    /// [`Launcher`](super::launcher::Launcher) holds the keyboard for the session.
    fn capture_any(&self) -> Key;
}
