mod gtk;
mod x11;

use crate::gtk::GtkApplication;
use crate::x11::X11;
use ordinator_core::model::key::KeyPress;
use ordinator_core::model::layer::{Action, Layer};
use ordinator_core::{run, Configuration};

fn layer_stack() -> Layer {
    let layout = Layer::new("layout".to_string())
        .add_action(KeyPress::from_keycode(43), Action::Command())
        .add_action(KeyPress::from_keycode(55), Action::Command());

    let volume = Layer::new("volume".to_string())
        .add_action(KeyPress::from_keycode(31), Action::Command())
        .add_action(KeyPress::from_keycode(42), Action::Command());

    let system = Layer::new("system".to_string())
        .add_action(KeyPress::from_keycode(55), Action::Branch(volume));

    let media = Layer::new("media".to_string())
        .add_action(KeyPress::from_keycode(44), Action::Command())
        .add_action(KeyPress::from_keycode(27), Action::Command())
        .add_action(KeyPress::from_keycode(40), Action::Command());

    Layer::new("root".to_string())
        .add_action(KeyPress::from_keycode(30), Action::Branch(layout))
        .add_action(KeyPress::from_keycode(40), Action::Branch(system))
        .add_action(KeyPress::from_keycode(58), Action::Branch(media))
}

fn configuration() -> Configuration {
    Configuration {
        keys_activate: vec![KeyPress::from_keycode(115)],
        keys_back: vec![KeyPress::from_keycode(22)],
        keys_deactivate: vec![KeyPress::from_keycode(9), KeyPress::from_keycode(115)],
        root_layer: layer_stack(),
    }
}

fn main() {
    let input = X11::new();
    let view = GtkApplication::new();
    run(&input, &view, configuration());
}
