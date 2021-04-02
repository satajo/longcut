use crate::model::state::State;

pub trait View {
    fn render(&mut self, model: &State);
}
