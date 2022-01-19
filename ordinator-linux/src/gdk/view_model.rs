use ordinator_core::model::command::ParameterVariant;
use ordinator_core::model::key::{Key, Modifier, Symbol};
use ordinator_core::port::view::{LayerNavigationData, ParameterInputData, ViewAction, ViewState};

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
pub struct LayerNavigationViewModel {
    pub stack: Vec<String>,
    pub actions: Vec<Action>,
}

impl<'a> From<LayerNavigationData<'a>> for LayerNavigationViewModel {
    fn from(data: LayerNavigationData<'a>) -> Self {
        Self {
            stack: data.layers.iter().map(|layer| layer.name.clone()).collect(),
            actions: data.actions.iter().map(make_action).collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParameterInputViewModel {
    pub current_input: String,
    pub parameter_name: String,
    pub parameter_placeholder: String,
    pub stack: Vec<String>,
}

impl<'a> From<ParameterInputData<'a>> for ParameterInputViewModel {
    fn from(data: ParameterInputData<'a>) -> Self {
        let mut stack: Vec<String> = data.layers.iter().map(|layer| layer.name.clone()).collect();
        stack.push(data.command.name.clone());

        let parameter_placeholder = match data.parameter.variant {
            ParameterVariant::Character => "Any character",
            ParameterVariant::Text => "Text",
        }
        .to_string();

        Self {
            current_input: data.input_value.to_string(),
            parameter_name: data.parameter.name.clone(),
            parameter_placeholder,
            stack,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ViewModel {
    Invisible,
    LayerNavigation(LayerNavigationViewModel),
    ParameterInput(ParameterInputViewModel),
}

impl<'a> From<ViewState<'a>> for ViewModel {
    fn from(data: ViewState) -> Self {
        match data {
            ViewState::None => ViewModel::Invisible,
            ViewState::LayerNavigation(data) => {
                ViewModel::LayerNavigation(LayerNavigationViewModel::from(data))
            }
            ViewState::ParameterInput(data) => {
                ViewModel::ParameterInput(ParameterInputViewModel::from(data))
            }
        }
    }
}

//-----------------------------------------------------------------------------
// Utilities
//-----------------------------------------------------------------------------

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
