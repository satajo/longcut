use crate::model::command::{Command, ParameterDeclaration};
use crate::model::key::Key;
use crate::model::layer::Layer;

pub enum ViewAction {
    Branch(String),
    Execute(String),
    Unbranch(),
    Deactivate(),
}

pub struct LayerNavigationData<'a> {
    pub actions: &'a [(Key, ViewAction)],
    pub layers: &'a [&'a Layer],
}

pub struct ParameterInputData<'a> {
    pub command: &'a Command,
    pub input_value: &'a str,
    pub parameter: &'a ParameterDeclaration,
    pub layers: &'a [&'a Layer],
}

pub enum ViewState<'a> {
    None,
    LayerNavigation(LayerNavigationData<'a>),
    ParameterInput(ParameterInputData<'a>),
}

pub trait View {
    fn render(&self, state: ViewState);
}
