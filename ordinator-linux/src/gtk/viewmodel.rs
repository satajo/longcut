use itertools::Itertools;
use ordinator_core::model::key::KeyPress;
use ordinator_core::model::layer::{Action, Layer};
use ordinator_core::model::state_machine::Fsm;
use ordinator_core::port::view::ViewData;

pub struct Settings {
    pub padding: u16,
}

pub struct Continuation {
    pub shortcut: String,
    pub name: String,
}

pub struct ViewModel {
    pub visible: bool,
    pub sequence: Vec<Continuation>,
    pub continuations: Vec<Continuation>,
    pub settings: Settings,
}

fn describe_keypress(keypress: &KeyPress) -> String {
    keypress.code.to_string()
}

fn describe_action(action: &Action) -> String {
    match action {
        Action::Branch(layer) => {
            format!("Layer {}", layer.name)
        }
        Action::Command() => "Command".to_string(),
    }
}

impl ViewModel {
    pub fn new(data: &ViewData) -> ViewModel {
        let mut continuations = vec![];
        for (press, action) in &data.actions {
            continuations.push(Continuation {
                shortcut: describe_keypress(&press),
                name: describe_action(&action),
            })
        }

        Self {
            visible: data.visible,
            sequence: Vec::new(),
            continuations,
            settings: Settings { padding: 8 },
        }
    }
}
