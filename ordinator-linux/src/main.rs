mod gtk;
mod x11;

use crate::gtk::GtkApplication;
use crate::x11::X11;
use ordinator_core::model::effect::Effect;
use ordinator_core::model::key::KeyPress;
use ordinator_core::model::layer::Layer;
use ordinator_core::{run, Configuration};

fn layer_stack() -> Layer {
    let layout = Layer::new("layout".to_string())
        .add_action(
            KeyPress::from_keycode(43),
            vec![Effect::Execute("horizontal")],
        )
        .add_action(
            KeyPress::from_keycode(55),
            vec![Effect::Execute("vertical")],
        );

    let volume = Layer::new("volume".to_string())
        .add_action(KeyPress::from_keycode(31), vec![Effect::Execute("up")])
        .add_action(KeyPress::from_keycode(42), vec![Effect::Execute("down")]);

    let system = Layer::new("system".to_string())
        .add_action(KeyPress::from_keycode(55), vec![Effect::Branch(volume)]);

    let media = Layer::new("media".to_string())
        .add_action(
            KeyPress::from_keycode(44),
            vec![Effect::Execute("next"), Effect::End()],
        )
        .add_action(
            KeyPress::from_keycode(27),
            vec![Effect::Execute("play"), Effect::End()],
        )
        .add_action(
            KeyPress::from_keycode(40),
            vec![Effect::Execute("stop"), Effect::End()],
        );

    let root = Layer::new("root".to_string())
        .add_action(KeyPress::from_keycode(30), vec![Effect::Branch(layout)])
        .add_action(KeyPress::from_keycode(40), vec![Effect::Branch(system)])
        .add_action(KeyPress::from_keycode(58), vec![Effect::Branch(media)]);

    return root;
}

fn configuration() -> Configuration {
    Configuration {
        launch_keys: vec![KeyPress::from_keycode(115)],
        end_keys: vec![KeyPress::from_keycode(9), KeyPress::from_keycode(115)],
        root_layer: layer_stack(),
    }
}

fn main() {
    let input = X11::new();
    let view = GtkApplication::new();
    run(input, view, configuration());
}
