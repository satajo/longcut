use crate::model::dimensions::Dimensions;
use crate::{Component, Context};

pub struct Empty;

impl Component for Empty {
    fn render(&self, _: &Context) {}

    fn measure(&self, _: &Context) -> Dimensions {
        Dimensions::new(0, 0)
    }
}
