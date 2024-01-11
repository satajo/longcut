use longcut_config::{ConfigError, ConfigModule, Module};

use crate::config::Config;
use crate::logic::{run_longcut, Context};
use crate::model::key::{Key, Symbol};
use crate::port::executor::Executor;
use crate::port::input::Input;
use crate::port::view::View;

pub struct CoreModule<'a> {
    executor: &'a dyn Executor,
    input: &'a dyn Input,
    view: &'a dyn View,
    config: Config,
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

        Ok(Self {
            executor,
            input,
            view,
            config,
        })
    }

    pub fn run(&self) {
        let keys_retry = [Key::new(Symbol::Return)];
        let context = Context {
            executor: self.executor,
            input: self.input,
            view: self.view,
            keys_activate: &self.config.keys_activate,
            keys_back: &self.config.keys_back,
            keys_deactivate: &self.config.keys_deactivate,
            keys_retry: &keys_retry,
            root_layer: &self.config.root_layer,
        };

        loop {
            run_longcut(&context);
        }
    }
}
