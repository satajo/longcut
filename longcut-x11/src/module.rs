use crate::handle::X11Handle;

pub struct X11Module {
    pub handle: X11Handle,
}

impl X11Module {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let handle = X11Handle::new();
        Self { handle }
    }
}
