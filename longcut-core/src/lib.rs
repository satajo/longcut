pub mod logic;
pub mod model;
pub mod port;

use crate::logic::activation::ActivationProgram;
use crate::logic::command_execution::CommandExecutionProgram;
use crate::logic::error::ErrorProgram;
use crate::logic::layer_stack::LayerStackProgram;
use crate::logic::parameter_input::ParameterInputProgram;
use crate::model::key::{Key, Symbol};
use crate::model::layer::Layer;
use crate::port::executor::Executor;
use crate::port::input::Input;
use crate::port::view::View;

#[derive(Debug)]
pub struct Configuration {
    pub keys_activate: Vec<Key>,
    pub keys_back: Vec<Key>,
    pub keys_deactivate: Vec<Key>,
    pub root_layer: Layer,
}

pub struct CoreModule<'a> {
    executor: &'a dyn Executor,
    input: &'a dyn Input,
    view: &'a dyn View,
    config: Configuration,
}

impl<'a> CoreModule<'a> {
    pub fn new(
        input: &'a impl Input,
        view: &'a impl View,
        executor: &'a impl Executor,
        config: Configuration,
    ) -> Self {
        Self {
            executor,
            input,
            view,
            config,
        }
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
