pub mod logic;
pub mod model;
pub mod port;

use crate::logic::activation::ActivationProgram;
use crate::logic::command_execution::CommandExecutionProgram;
use crate::logic::layer_stack::LayerStackProgram;
use crate::model::key::KeyPress;
use crate::model::layer::Layer;
use crate::port::input::Input;
use crate::port::view::View;

pub struct Configuration {
    pub keys_activate: Vec<KeyPress>,
    pub keys_back: Vec<KeyPress>,
    pub keys_deactivate: Vec<KeyPress>,
    pub root_layer: Layer,
}

pub fn run(input: &impl Input, view: &impl View, config: Configuration) {
    let executor_program = CommandExecutionProgram::new();
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
