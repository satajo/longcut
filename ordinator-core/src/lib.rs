pub mod model;
pub mod port;

use crate::model::key::KeyPress;
use crate::model::layer::Layer;
use crate::model::state_machine::{Fsm, FsmState};
use crate::port::input::Input;
use crate::port::view::View;

pub struct Configuration {
    pub launch_keys: Vec<KeyPress>,
    pub end_keys: Vec<KeyPress>,
    pub root_layer: Layer,
}

pub fn run(input: &impl Input, view: &impl View, config: Configuration) {
    let keys_reset = vec![KeyPress::from_keycode(101)];
    let keys_unbranch = vec![KeyPress::from_keycode(22)];
    let mut fsm = Fsm::new(
        config.root_layer,
        config.end_keys,
        keys_reset,
        config.launch_keys,
        keys_unbranch,
    );

    loop {
        fsm = match fsm {
            Fsm::Branch(state) => state.step(input, view),
            Fsm::Inactive(state) => state.step(input, view),
            Fsm::Root(state) => state.step(input, view),
            Fsm::Finished(_) => {
                break;
            }
        }
    }
}
