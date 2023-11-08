use crate::context::Context;
use crate::Dimensions;

pub mod column;
pub mod empty;
pub mod row;
pub mod table;
pub mod text;

pub trait Component {
    fn render(&self, ctx: &Context);

    /// Returns the "desired size" of the component. If the component could be any size regardless
    /// of the limitations of the Context, what size would it *want* to be? The current context can
    /// be used as a hint, but it is not required.
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
