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

pub fn run(input: &impl Input, view: &impl View, executor: &impl Executor, config: Configuration) {
    let keys_retry = [Key::new(Symbol::Return)];
    let error_program = ErrorProgram::new(
        input,
        view,
        &config.keys_back,
        &config.keys_deactivate,
        &keys_retry,
    );

    let parameter_input_program = ParameterInputProgram::new(input, view, &config.keys_deactivate);

    let executor_program =
        CommandExecutionProgram::new(executor, &error_program, &parameter_input_program);

    let layer_program = LayerStackProgram::new(
        input,
        view,
        &executor_program,
        &config.keys_back,
        &config.keys_deactivate,
        &config.root_layer,
    );

    let activation_program =
        ActivationProgram::new(input, view, &config.keys_activate, &layer_program);

    loop {
        activation_program.run();
    }
}
