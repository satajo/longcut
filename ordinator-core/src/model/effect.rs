use crate::model::layer::Layer;

#[derive(Clone)]
pub enum Effect {
    Branch(Layer),
    End(),
    Execute(&'static str),
    NotFound(),
}
