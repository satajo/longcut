pub mod model;
pub mod port;

use crate::model::event::Event;
use crate::model::key::KeyPress;
use crate::model::layer::Layer;
use crate::model::state::{EndCondition, State};
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
    let mut state: Option<State> = None;

    loop {
        view.render(&state);

        if let Some(sequence) = state {
            let press = input.capture_any();
            let (result, events) = sequence.handle_keypress(&press);

            handle_events(events);

            match result {
                Ok(new_state) => {
                    state = Some(new_state);
                }
                Err(end_condition) => match end_condition {
                    EndCondition::Done => {
                        state = None;
                    }
                    EndCondition::Exit => {
                        return;
                    }
                },
            }
        } else {
            input.capture_one(&config.launch_keys);
            state = Some(State::new(&config.root_layer));
        }
    }
}
