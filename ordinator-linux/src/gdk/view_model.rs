use ordinator_core::model::key::KeyPress;
use ordinator_core::port::view::{ViewAction, ViewData};

pub struct Action {
    pub shortcut: String,
    pub name: String,
}

pub struct LayerView {
    pub stack: Vec<String>,
    pub actions: Vec<Action>,
}

pub enum ViewModel {
    Layer(LayerView),
    Invisible,
}

impl From<&ViewData> for ViewModel {
    fn from(data: &ViewData) -> Self {
        if data.visible {
            ViewModel::Layer(LayerView {
                stack: data.layers.clone(),
                actions: data.actions.iter().map(make_action).collect(),
            })
        } else {
            ViewModel::Invisible
        }
    }
}

fn make_action((press, action): &(KeyPress, ViewAction)) -> Action {
    let name = match action {
        ViewAction::Branch(layer) => format!("Branch {}", layer),
        ViewAction::Execute(command) => format!("Execute {}", command),
        ViewAction::Unbranch() => "Unbranch".to_string(),
        ViewAction::Deactivate() => "Deactivate".to_string(),
    };

    Action {
        shortcut: press.code.to_string(),
        name,
    }
}
