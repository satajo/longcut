use crate::context::Context;
use crate::Dimensions;

pub mod column;
pub mod empty;
pub mod root;
pub mod row;
pub mod table;
pub mod text;

pub trait Component {
    fn render(&self, ctx: &Context);

    fn measure(&self, ctx: &Context) -> Dimensions;
}

impl<'a, T: Component> Component for &'a T {
    fn render(&self, ctx: &Context) {
        (*self).render(ctx)
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        (*self).measure(ctx)
    }
}

impl Component for Box<dyn Component> {
    fn render(&self, ctx: &Context) {
        self.as_ref().render(ctx)
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        self.as_ref().measure(ctx)
    }
}
