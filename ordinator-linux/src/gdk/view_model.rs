use ordinator_core::model::key::{KeyPress, Symbol};
use ordinator_core::port::view::{LayerViewData, ViewAction, ViewState};

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

impl From<&ViewState> for ViewModel {
    fn from(data: &ViewState) -> Self {
        match data {
            ViewState::Hidden => ViewModel::Invisible,
            ViewState::LayerView(data) => ViewModel::Layer(LayerView {
                stack: data.layers.clone(),
                actions: data.actions.iter().map(make_action).collect(),
            }),
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
        shortcut: show_shortcut(press),
        name,
    }
}

fn show_shortcut(key: &KeyPress) -> String {
    match &key.symbol {
        Symbol::Character(c) => c.to_string(),
        otherwise => format!("{:?}", otherwise).to_lowercase(),
    }
}
