use crate::handle::X11Handle;

pub struct X11Module {
    pub x11_handle: X11Handle,
}

impl X11Module {
    #[expect(
        clippy::new_without_default,
        reason = "delegates to X11Handle::new() which opens the X11 display; Default would hide this"
    )]
    #[must_use]
    pub fn new() -> Self {
        let x11_handle = X11Handle::new();
        Self { x11_handle }
    }
}
