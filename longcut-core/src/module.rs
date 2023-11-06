use crate::config::Config;
use crate::mode::command_execution::CommandExecutionMode;
use crate::mode::error::ErrorMode;
use crate::mode::inactive::InactiveMode;
use crate::mode::layer_navigation::LayerNavigationMode;
use crate::mode::parameter_input::ParameterInputMode;
use crate::model::key::{Key, Symbol};
use crate::port::executor::Executor;
use crate::port::input::Input;
use crate::port::view::View;
use longcut_config::{ConfigError, ConfigModule, Module};

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

        let error_mode = ErrorMode::new(
            self.input,
            self.view,
            &self.config.keys_back,
            &self.config.keys_deactivate,
            &keys_retry,
        );

        let parameter_input_mode = ParameterInputMode::new(
            self.input,
            self.view,
            &self.config.keys_back,
            &self.config.keys_deactivate,
        );

        let command_executor_mode =
            CommandExecutionMode::new(self.executor, &error_mode, &parameter_input_mode);

        let layer_navigation_mode = LayerNavigationMode::new(
            self.input,
            self.view,
            &command_executor_mode,
            &self.config.keys_back,
            &self.config.keys_deactivate,
            &self.config.root_layer,
        );

        let inactive_mode = InactiveMode::new(
            self.input,
            self.view,
            &self.config.keys_activate,
            &layer_navigation_mode,
        );

        loop {
            inactive_mode.run();
        }
    }
}
