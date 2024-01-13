use crate::handle::X11Handle;

pub struct X11Module {
    pub x11_handle: X11Handle,
}

impl X11Module {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let x11_handle = X11Handle::new();
        Self { x11_handle }
    }
}
