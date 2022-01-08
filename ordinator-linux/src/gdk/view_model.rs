use ordinator_core::model::key::{Key, Modifier, Symbol};
use ordinator_core::port::view::{LayerViewData, ViewAction, ViewState};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ActionType {
    Branch { layer: String },
    Execute { program: String },
    Unbranch,
    Deactivate,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Action {
    pub shortcut: String,
    pub kind: ActionType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LayerView {
    pub stack: Vec<String>,
    pub actions: Vec<Action>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ViewModel {
    Layer(LayerView),
    Invisible,
    InputCharacter,
    InputText(String),
}

impl From<&ViewState> for ViewModel {
    fn from(data: &ViewState) -> Self {
        match data {
            ViewState::Hidden => ViewModel::Invisible,
            ViewState::LayerView(data) => ViewModel::Layer(make_layer_view(data)),
            ViewState::InputCharacter => ViewModel::InputCharacter,
            ViewState::InputString { input } => ViewModel::InputText(input.clone()),
        }
    }
}

fn make_layer_view(data: &LayerViewData) -> LayerView {
    LayerView {
        stack: data.layers.clone(),
        actions: data.actions.iter().map(make_action).collect(),
    }
}

fn make_action((key, action): &(Key, ViewAction)) -> Action {
    let shortcut = show_shortcut(key);
    let kind = match action {
        ViewAction::Branch(layer) => ActionType::Branch {
            layer: layer.clone(),
        },
        ViewAction::Execute(command) => ActionType::Execute {
            program: command.clone(),
        },
        ViewAction::Unbranch() => ActionType::Unbranch,
        ViewAction::Deactivate() => ActionType::Deactivate,
    };

    Action { shortcut, kind }
}

fn show_shortcut(key: &Key) -> String {
    let mut modifiers = String::new();

    if key.modifiers.contains(&Modifier::Shift) {
        modifiers += "s-";
    }

    if key.modifiers.contains(&Modifier::Control) {
        modifiers += "c-";
    }

    if key.modifiers.contains(&Modifier::Alt) {
        modifiers += "a-";
    }

    if key.modifiers.contains(&Modifier::Super) {
        modifiers += "u-";
    }

    let symbol = match &key.symbol {
        Symbol::Character(c) => c.to_string(),
        otherwise => format!("{:?}", otherwise).to_lowercase(),
    };

    modifiers.push_str(&symbol);
    modifiers
}
