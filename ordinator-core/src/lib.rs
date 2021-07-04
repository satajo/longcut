pub mod model;
pub mod port;

use crate::model::event::Event;
use crate::model::key::KeyPress;
use crate::model::layer::Layer;
use crate::model::state::{GlobalKeys, InitialState, Sequence, SequenceState};
use crate::port::input::Input;
use crate::port::view::View;

pub struct Configuration {
    pub launch_keys: Vec<KeyPress>,
    pub end_keys: Vec<KeyPress>,
    pub root_layer: Layer,
}

pub fn handle_events(events: Vec<Event>) {
    for event in events {
        println!("Handling event: {:?}", event)
    }
}

pub fn run(mut input: impl Input, mut view: impl View, config: Configuration) {
    let keybindings = GlobalKeys {
        cancel: config.end_keys,
        start: config.launch_keys,
        unbranch: vec![],
        exit: vec![],
    };
    let initial = InitialState::new(config.root_layer, keybindings);
    loop {
        input.capture_one(initial.launch_keys());
        let mut state = SequenceState::Active(initial.begin_sequence());

        // Handling the run of a single sequence until completion.
        loop {
            match state {
                SequenceState::Active(sequence) => {
                    view.show(&sequence);

                    let press = input.capture_any();
                    let (result, events) = sequence.handle_keypress(&press);
                    handle_events(events);
                    state = result;
                }
                SequenceState::Done => break,
                SequenceState::Exit => return,
            }
        }

        view.hide();
    }
}
