use ordinator_core::model::command::ParameterVariant;
use ordinator_core::model::key::{Key, Modifier, Symbol};
use ordinator_core::port::executor::ExecutorError;
use ordinator_core::port::view::{
    ErrorData, LayerNavigationData, ParameterInputData, ViewAction, ViewState,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ViewModel {
    Invisible,
    Error(ErrorViewModel),
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
            ViewState::Error(data) => ViewModel::Error(ErrorViewModel::from(data)),
        }
    }
}

//-----------------------------------------------------------------------------
// ErrorView
//-----------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub struct ErrorViewModel {
    pub actions: Vec<Action>,
    pub error_details: String,
    pub error_type: String,
}

impl From<ErrorData<'_>> for ErrorViewModel {
    fn from(data: ErrorData) -> Self {
        let error_type = match data.error {
            ExecutorError::RuntimeError(_) => "Runtime error".to_string(),
            ExecutorError::StartupError => "Startup error".to_string(),
            ExecutorError::UnknownError => "Unknown error".to_string(),
        };

        let error_details = match data.error {
            ExecutorError::RuntimeError(details) => details.clone(),
            ExecutorError::StartupError => "Failed to start the target command".to_string(),
            ExecutorError::UnknownError => "No error details available".to_string(),
        };

        let actions = data
            .actions
            .iter()
            .map(|(key, action)| make_action(key, action))
            .collect();

        Self {
            actions,
            error_details,
            error_type,
        }
    }
}

//-----------------------------------------------------------------------------
// LayerNavigationView
//-----------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub struct LayerNavigationViewModel {
    pub stack: Vec<String>,
    pub actions: Vec<Action>,
}

impl From<LayerNavigationData<'_>> for LayerNavigationViewModel {
    fn from(data: LayerNavigationData) -> Self {
        let stack = data.layers.iter().map(|layer| layer.name.clone()).collect();
        let actions = data
            .actions
            .iter()
            .map(|(key, action)| make_action(key, action))
            .collect();

        Self { stack, actions }
    }
}

//-----------------------------------------------------------------------------
// ParameterInputView
//-----------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
pub struct ParameterInputViewModel {
    pub current_input: String,
    pub parameter_name: String,
    pub parameter_placeholder: String,
    pub stack: Vec<String>,
}

impl From<ParameterInputData<'_>> for ParameterInputViewModel {
    fn from(data: ParameterInputData) -> Self {
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

//-----------------------------------------------------------------------------
// Utilities
//-----------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ActionType {
    Branch { layer: String },
    Execute { program: String },
    Unbranch,
    Deactivate,
    Retry,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Action {
    pub shortcut: String,
    pub kind: ActionType,
}

fn make_action(key: &Key, action: &ViewAction) -> Action {
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
        ViewAction::Retry() => ActionType::Retry,
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
