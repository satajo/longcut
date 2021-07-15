use ordinator_core::model::key::KeyPress;
use ordinator_core::port::view::{ViewAction, ViewData};

pub struct Continuation {
    pub shortcut: String,
    pub name: String,
}

pub struct ViewModel {
    pub visible: bool,
    pub sequence: Vec<Continuation>,
    pub continuations: Vec<Continuation>,
}

fn describe_keypress(keypress: &KeyPress) -> String {
    keypress.code.to_string()
}

fn describe_action(action: &ViewAction) -> String {
    match action {
        ViewAction::Branch(layer_name) => format!("Layer {}", layer_name),
        ViewAction::Execute(command_name) => format!("Run {}", command_name),
        ViewAction::Unbranch() => "Unbranch".to_string(),
        ViewAction::Deactivate() => "Abort".to_string(),
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
        }
    }
}
