mod activation;
mod command_execution;
mod layer_stack;

use crate::logic::activation::ActivationProgram;
use crate::model::key::KeyPress;
use crate::model::layer::Layer;
use crate::port::input::Input;
use crate::port::view::View;

pub struct Context<'a> {
    pub input: &'a dyn Input,
    pub view: &'a dyn View,
    pub keys_activate: &'a [KeyPress],
    pub keys_back: &'a [KeyPress],
    pub keys_deactivate: &'a [KeyPress],
    pub root_layer: &'a Layer,
}

pub struct Program {}

impl Program {
    pub fn run(ctx: &Context) {
        ActivationProgram::run(ctx)
    }
}
