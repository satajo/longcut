use crate::handle::{X11Error, X11Handle};

pub struct X11Module {
    pub x11_handle: X11Handle,
}

impl X11Module {
    /// Connects to the X server and fetches its keymap.
    ///
    /// # Errors
    ///
    /// Returns an error if the X server cannot be reached or its keymap cannot be fetched.
    pub fn new() -> Result<Self, X11Error> {
        Ok(Self {
            x11_handle: X11Handle::connect()?,
        })
    }
}
