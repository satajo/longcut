use crate::service::GtkService;
use longcut_config::Module;

pub struct GtkModule {
    pub gtk_service: GtkService,
}

impl Module for GtkModule {
    const IDENTIFIER: &'static str = "gtk";

    type Config = ();
}

impl GtkModule {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let gtk_service = GtkService::new();
        GtkModule { gtk_service }
    }
}
