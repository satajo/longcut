use crate::model::key::KeyPress;
use crate::model::layer::Action;

pub struct ViewData {
    pub visible: bool,
    pub actions: Vec<(KeyPress, Action)>,
}

pub trait View {
    fn render(&self, state: &ViewData);
}
