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
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let xcb_service = XcbService::new();
        XcbModule { xcb_service }
    }
}
