use crate::model::key::KeyPress;

pub enum ViewAction {
    Branch(String),
    Execute(String),
    Unbranch(),
    Deactivate(),
}

pub struct ViewData {
    pub visible: bool,
    pub actions: Vec<(KeyPress, ViewAction)>,
}

pub trait ToViewData {
    fn to_view_data(&self) -> ViewData;
}

pub trait View {
    fn render(&self, state: &ViewData);
}
