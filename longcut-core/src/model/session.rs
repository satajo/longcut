/// Which root layer a session navigates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionMode {
    /// The global root layer.
    Global,
    /// The root layer configured for the currently active window. Shows the "application
    /// unconfigured" error when no configured application pattern matches the active window.
    Window,
}
