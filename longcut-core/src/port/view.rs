use crate::model::command::{Command, CommandParameter};
use crate::model::key::Key;
use crate::model::layer::Layer;
use crate::port::executor::ExecutorError;

pub enum ViewAction {
    Branch(String),
    Execute(String),
    Unbranch,
    Deactivate,
    Retry,
}

pub type ActionShortcuts<'a> = &'a [(&'a Key, ViewAction)];

pub type LayerStack<'a> = &'a [&'a Layer];

pub struct ErrorViewModel<'a> {
    pub actions: ActionShortcuts<'a>,
    pub error: &'a ExecutorError,
}

pub struct LayerNavigationViewModel<'a> {
    pub actions: ActionShortcuts<'a>,
    pub layer_stack: LayerStack<'a>,
}

pub struct ParameterInputViewModel<'a> {
    pub input_value: &'a str,
    pub command: &'a Command,
    pub parameter: &'a CommandParameter,
    pub layer_stack: LayerStack<'a>,
}

pub enum ViewModel<'a> {
    None,
    Error(ErrorViewModel<'a>),
    LayerNavigation(LayerNavigationViewModel<'a>),
    ParameterInput(ParameterInputViewModel<'a>),
}

pub trait View {
    fn render(&self, state: ViewModel);
}
