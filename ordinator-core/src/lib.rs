pub mod model;
pub mod port;

use crate::model::effect::Effect;
use crate::model::key::KeyPress;
use crate::model::layer::Layer;
use crate::model::state::State;
use crate::port::input::Input;
use crate::port::view::View;

pub struct Configuration {
    pub launch_keys: Vec<KeyPress>,
    pub end_keys: Vec<KeyPress>,
    pub root_layer: Layer,
}

pub fn run(mut input: impl Input, mut view: impl View, config: Configuration) {
    let mut state = State::new(config.root_layer, config.launch_keys, config.end_keys);
    loop {
        if !state.is_active() {
            input.capture_one(state.get_launch_keys());
            state.begin_sequence();
        } else {
            let press = input.capture_any();
            println!("Pressed {:?}", press);
            for effect in state.handle_keypress(&press) {
                match effect {
                    Effect::Branch(layer) => {
                        println!("Switching layers!");
                        state.set_active_layer(layer);
                    }
                    Effect::End() => {
                        println!("Ending sequence!");
                        state.end_sequence();
                    }
                    Effect::Execute(name) => {
                        println!("Executing command {}!", name)
                    }
                    Effect::NotFound() => {
                        println!("Command not found!")
                    }
                }
            }
        }

        view.render(&state);
    }
}
