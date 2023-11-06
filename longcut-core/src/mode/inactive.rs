use crate::mode::layer_navigation::LayerNavigationMode;
use crate::model::key::Key;
use crate::port::input::Input;
use crate::port::view::{View, ViewModel};

/// Waits idly for an activation signal, after which launches the [LayerNavigationMode] mode.
pub struct InactiveMode<'a> {
    input: &'a dyn Input,
    view: &'a dyn View,
    // Configuration
    keys_activate: &'a [Key],
    layer_navigation_mode: &'a LayerNavigationMode<'a>,
}

impl<'a> InactiveMode<'a> {
    pub fn new(
        input: &'a dyn Input,
        view: &'a dyn View,
        keys_activate: &'a [Key],
        layer_navigation_mode: &'a LayerNavigationMode<'a>,
    ) -> Self {
        Self {
            input,
            view,
            keys_activate,
            layer_navigation_mode,
        }
    }

    pub fn run(&self) {
        self.input.capture_one(self.keys_activate);
        self.layer_navigation_mode.run();
        self.view.render(ViewModel::None);
    }
}
