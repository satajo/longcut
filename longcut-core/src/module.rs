use crate::config::Config;
use crate::logic::activation::ActivationProgram;
use crate::logic::command_execution::CommandExecutionProgram;
use crate::logic::error::ErrorProgram;
use crate::logic::layer_stack::LayerStackProgram;
use crate::logic::parameter_input::ParameterInputProgram;
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
        let error_program = ErrorProgram::new(
            self.input,
            self.view,
            &self.config.keys_back,
            &self.config.keys_deactivate,
            &keys_retry,
        );

        let parameter_input_program =
            ParameterInputProgram::new(self.input, self.view, &self.config.keys_deactivate);

        let executor_program =
            CommandExecutionProgram::new(self.executor, &error_program, &parameter_input_program);

        let layer_program = LayerStackProgram::new(
            self.input,
            self.view,
            &executor_program,
            &self.config.keys_back,
            &self.config.keys_deactivate,
            &self.config.root_layer,
        );

        let activation_program = ActivationProgram::new(
            self.input,
            self.view,
            &self.config.keys_activate,
            &layer_program,
        );

        loop {
            activation_program.run();
        }
    }
}
