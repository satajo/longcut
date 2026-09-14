use crate::service::{XcbError, XcbService};
use longcut_config::Module;

#[derive(Debug)]
pub struct XcbModule {
    pub xcb_service: XcbService,
}

impl Module for XcbModule {
    const IDENTIFIER: &'static str = "xcb";

    type Config = ();
}

impl XcbModule {
    /// Connects to the X server.
    ///
    /// # Errors
    ///
    /// Returns an error if the X server cannot be connected to.
    pub fn new() -> Result<Self, XcbError> {
        let xcb_service = XcbService::new()?;
        Ok(XcbModule { xcb_service })
    }
}
