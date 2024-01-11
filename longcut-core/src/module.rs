use longcut_config::{ConfigError, ConfigModule, Module};

use crate::config::Config;
use crate::port::executor::Executor;
use crate::port::input::Input;
use crate::port::view::View;
use crate::service::CoreService;

pub struct CoreModule<'a> {
    pub longcut_service: CoreService<'a>,
}

impl Module for CoreModule<'_> {
    const IDENTIFIER: &'static str = "core";

    type Config = Config;
}

impl<'a> CoreModule<'a> {
    pub fn new(
        config_module: &'a ConfigModule,
        input: &'a impl Input,
        view: &'a impl View,
        executor: &'a impl Executor,
    ) -> Result<Self, ConfigError> {
        let config = config_module.config_for_module::<Self>()?;
        let longcut_service = CoreService::new(executor, input, view, config);
        Ok(Self { longcut_service })
    }
}
