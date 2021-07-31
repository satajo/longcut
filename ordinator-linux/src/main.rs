mod gdk;
mod x11;

use crate::gdk::GdkApplication;
use crate::x11::X11;
use ordinator_core::model::key::{KeyPress, Symbol};
use ordinator_core::model::layer::{Action, Layer};
use ordinator_core::{run, Configuration};

fn layer_stack() -> Layer {
    let layout = Layer::new("layout".to_string())
        .add_action(KeyPress::from_character('h'), Action::Command())
        .add_action(KeyPress::from_character('v'), Action::Command());

    let volume = Layer::new("volume".to_string())
        .add_action(KeyPress::from_character('d'), Action::Command())
        .add_action(KeyPress::from_character('u'), Action::Command());

    let system = Layer::new("system".to_string())
        .add_action(KeyPress::from_character('v'), Action::Branch(volume));

    let media = Layer::new("media".to_string())
        .add_action(KeyPress::from_character('n'), Action::Command())
        .add_action(KeyPress::from_character('p'), Action::Command())
        .add_action(KeyPress::from_character('s'), Action::Command());

    Layer::new("root".to_string())
        .add_action(KeyPress::from_character('l'), Action::Branch(layout))
        .add_action(KeyPress::from_character('m'), Action::Branch(media))
        .add_action(KeyPress::from_character('s'), Action::Branch(system))
}

fn configuration() -> Configuration {
    Configuration {
        keys_activate: vec![KeyPress::from_symbol(Symbol::Home)],
        keys_back: vec![KeyPress::from_symbol(Symbol::BackSpace)],
        keys_deactivate: vec![
            KeyPress::from_symbol(Symbol::Escape),
            KeyPress::from_symbol(Symbol::Home),
        ],
        root_layer: layer_stack(),
    }
}

fn main() {
    let input = X11::new();
    let view = GdkApplication::new();
    run(&input, &view, configuration());
}
