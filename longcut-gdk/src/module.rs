use crate::service::GdkService;
use longcut_config::Module;

pub struct GdkModule {
    pub gdk_service: GdkService,
}

impl Module for GdkModule {
    const IDENTIFIER: &'static str = "gdk";

    type Config = ();
}

impl GdkModule {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let gdk_service = GdkService::new();
        GdkModule { gdk_service }
    }
}
