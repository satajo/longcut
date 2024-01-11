use crate::port::window_manager::WindowManager;
use crate::{config::Config, GuiService};
use longcut_config::{ConfigError, ConfigModule, Module};

pub struct GuiModule<'a> {
    pub gui_service: GuiService<'a>,
}

impl Module for GuiModule<'_> {
    const IDENTIFIER: &'static str = "gui";

    type Config = Config;
}

impl<'a> GuiModule<'a> {
    pub fn new(
        config_module: &'a ConfigModule,
        window_manager: &'a dyn WindowManager,
    ) -> Result<Self, ConfigError> {
        let config = config_module.config_for_module::<Self>()?;
        let gui_service = GuiService::new(window_manager, config);
        Ok(Self { gui_service })
    }
}
