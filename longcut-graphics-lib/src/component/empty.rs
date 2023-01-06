use crate::component::Component;
use crate::context::Context;
use crate::model::dimensions::Dimensions;

pub struct Empty;

impl Component for Empty {
    fn render(&self, _: &Context) {}

    fn measure(&self, _: &Context) -> Dimensions {
        Dimensions::new(0, 0)
    }
}
