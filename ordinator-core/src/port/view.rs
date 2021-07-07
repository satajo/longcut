use crate::model::key::KeyPress;
use crate::model::layer::Action;

pub struct ViewData {
    pub visible: bool,
    pub actions: Vec<(KeyPress, Action)>,
}

pub trait View {
    fn render(&mut self, state: &ViewData);
}
