use crate::model::session::SessionMode;

/// Reports the user's requests for a session. The launcher owns whatever hotkey or signal the
/// request arrives through; the keyboard for the session is taken through the
/// [`Input`](super::input::Input) port once the request is in.
pub trait Launcher {
    /// Block until the user requests a session and return the mode they asked for.
    fn wait_for_launch(&self) -> SessionMode;
}
