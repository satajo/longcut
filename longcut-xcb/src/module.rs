use crate::service::XcbService;
use longcut_config::Module;

pub struct XcbModule {
    pub xcb_service: XcbService,
}

impl Module for XcbModule {
    const IDENTIFIER: &'static str = "xcb";

    type Config = ();
}

impl XcbModule {
    #[expect(
        clippy::new_without_default,
        reason = "delegates to XcbService::new() which connects to the X server; Default would hide this"
    )]
    #[must_use]
    pub fn new() -> Self {
        let xcb_service = XcbService::new();
        XcbModule { xcb_service }
    }
}
