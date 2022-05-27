use crate::logic::layer_stack::LayerStackProgram;
use crate::model::key::Key;
use crate::port::input::Input;
use crate::port::view::{View, ViewModel};

pub struct ActivationProgram<'a> {
    input: &'a dyn Input,
    view: &'a dyn View,
    // Configuration
    keys_activate: &'a [Key],
    layer_stack: &'a LayerStackProgram<'a>,
}

impl<'a> ActivationProgram<'a> {
    pub fn new(
        input: &'a impl Input,
        view: &'a impl View,
        keys_activate: &'a [Key],
        layer_stack: &'a LayerStackProgram<'a>,
    ) -> Self {
        Self {
            input,
            view,
            keys_activate,
            layer_stack,
        }
    }

    pub fn run(&self) {
        self.input.capture_one(self.keys_activate);
        self.layer_stack.run();
        self.view.render(ViewModel::None);
    }
}
