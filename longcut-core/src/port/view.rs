use crate::model::command::Command;
use crate::model::key::Key;
use crate::model::layer::Layer;
use std::time::Duration;

#[derive(Debug)]
pub enum ViewAction {
    Branch(String),
    Execute(String),
    Unbranch,
    Exit,
    Retry,
    Kill,
}

pub type ActionShortcuts<'a> = &'a [(&'a Key, ViewAction)];

pub type LayerStack<'a> = &'a [&'a Layer];

/// A synchronous command that is still running, and the keys that kill it.
#[derive(Debug)]
pub struct CommandRunningViewModel<'a> {
    pub actions: ActionShortcuts<'a>,
    pub command_name: &'a str,
    pub program: &'a str,
    pub elapsed: Duration,
}

#[derive(Debug)]
pub struct ErrorViewModel<'a> {
    pub actions: ActionShortcuts<'a>,
    pub error_type: &'a str,
    pub error_details: &'a str,
}

#[derive(Debug)]
pub struct LayerNavigationViewModel<'a> {
    pub actions: ActionShortcuts<'a>,
    pub layer_stack: LayerStack<'a>,
}

#[derive(Debug)]
pub enum ParameterVariant<'a> {
    Character,
    Text { input_value: &'a str },
    Choose { options: &'a [(&'a Key, &'a str)] },
}

#[derive(Debug)]
pub struct ParameterInputViewModel<'a> {
    pub command: &'a Command,
    pub parameter_name: &'a str,
    pub parameter: ParameterVariant<'a>,
    pub layer_stack: LayerStack<'a>,
}

#[derive(Debug)]
pub enum ViewModel<'a> {
    None,
    CommandRunning(CommandRunningViewModel<'a>),
    Error(ErrorViewModel<'a>),
    LayerNavigation(LayerNavigationViewModel<'a>),
    ParameterInput(ParameterInputViewModel<'a>),
}

pub trait View {
    fn render(&self, state: ViewModel);
}
