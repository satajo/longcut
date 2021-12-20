use crate::model::color::Color;
use crate::model::dimensions::Dimensions;
use crate::{Component, Context};

pub struct HasBackground<C: Component> {
    color: Color,
    child: C,
}

impl<C: Component> Component for HasBackground<C> {
    fn render(&self, ctx: &Context) {
        ctx.with_color(&self.color, |ctx| ctx.draw_rectangle(&ctx.region));
        self.child.render(ctx);
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        self.child.measure(ctx)
    }
}

pub trait Background<C: Component> {
    fn background(self, color: Color) -> HasBackground<C>;
}

impl<C: Component> Background<C> for C {
    fn background(self, color: Color) -> HasBackground<C> {
        HasBackground { child: self, color }
    }
}
