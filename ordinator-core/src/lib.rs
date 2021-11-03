pub mod logic;
pub mod model;
pub mod port;

use crate::logic::{Context, Program};
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
    let context = Context {
        input,
        view,
        keys_activate: config.keys_activate.as_slice(),
        keys_back: config.keys_back.as_slice(),
        keys_deactivate: config.keys_deactivate.as_slice(),
        root_layer: &config.root_layer,
    };

    loop {
        Program::run(&context);
    }
}
