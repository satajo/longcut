mod logic;
pub mod model;
pub mod port;

use crate::logic::program::{Program, RunProgram};
use crate::logic::state_machine::StateMachine;
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
    let state_machine = StateMachine::new(config.root_layer);
    let mut program = Program::new(
        input,
        view,
        state_machine,
        config.keys_activate,
        config.keys_back,
        config.keys_deactivate,
    );

    loop {
        program = match program {
            Program::Branch(state) => state.run(),
            Program::Inactive(state) => state.run(),
            Program::Root(state) => state.run(),
        }
    }
}
