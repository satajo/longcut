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

pub struct ErrorViewModel<'a> {
    pub actions: &'a [(&'a Key, ViewAction)],
    pub error: &'a ExecutorError,
}

pub struct LayerNavigationViewModel<'a> {
    pub actions: &'a [(&'a Key, ViewAction)],
    pub layers: &'a [&'a Layer],
}

pub struct ParameterInputViewModel<'a> {
    pub command: &'a Command,
    pub input_value: &'a str,
    pub parameter: &'a CommandParameter,
    pub layers: &'a [&'a Layer],
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
