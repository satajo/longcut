use crate::model::key::KeyPress;

pub enum ViewAction {
    Branch(String),
    Execute(String),
    Unbranch(),
    Deactivate(),
}

pub struct ViewData {
    pub actions: Vec<(KeyPress, ViewAction)>,
    pub visible: bool,
    pub layers: Vec<String>,
}

pub trait ToViewData {
    fn to_view_data(&self) -> ViewData;
}

pub trait View {
    fn render(&self, state: &ViewData);
}
