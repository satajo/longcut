use crate::core::model::Model;

pub trait View {
    fn render(&self, model: &Model);
}
