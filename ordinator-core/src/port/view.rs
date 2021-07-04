use crate::model::state::Sequence;

pub trait View {
    fn show(&mut self, model: &Sequence);

    fn hide(&mut self);
}
