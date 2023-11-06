use crate::config::Config;
use crate::service::ShellService;
use longcut_config::{ConfigError, ConfigModule, Module};

pub struct ShellModule {
    pub service: ShellService,
}

impl Module for ShellModule {
    const IDENTIFIER: &'static str = "shell";

    type Config = Config;
}

impl ShellModule {
    pub fn new(config_module: &ConfigModule) -> Result<Self, ConfigError> {
        let config = config_module.config_for_module::<Self>()?;
        let service = ShellService::new(config.default_timeout);

        Ok(Self { service })
    }
}
