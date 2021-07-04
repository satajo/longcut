use itertools::Itertools;
use ordinator_core::model::key::KeyPress;
use ordinator_core::model::layer::{Action, Layer};
use ordinator_core::model::state::Sequence;

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
    pub fn empty() -> Self {
        return ViewModel {
            visible: false,
            sequence: Vec::new(),
            continuations: Vec::new(),
            settings: Settings { padding: 8 },
        };
    }

    pub fn from_model(state: &Sequence) -> Self {
        let mut vm = Self::empty();
        vm.visible = true;
        for (keypress, action) in &state.active_layer().actions {
            vm.continuations.push(Continuation {
                shortcut: describe_keypress(&keypress),
                name: describe_action(&action),
            })
        }
        return vm;
    }
}
