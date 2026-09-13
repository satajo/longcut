use crate::model::session::SessionMode;

/// Starts sessions when the user asks for one. The launcher owns whatever hotkey or signal the
/// request arrives through, and it delivers the keyboard to the [`Input`](super::input::Input)
/// port for the session that follows.
pub trait Launcher {
    /// Block until the user requests a session. The session lasts until the returned value is
    /// dropped.
    fn wait_for_launch(&self) -> Box<dyn Session + '_>;
}

/// A session the user requested. Dropping it ends the session.
pub trait Session {
    fn mode(&self) -> SessionMode;
}
