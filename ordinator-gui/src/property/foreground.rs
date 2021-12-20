use crate::model::color::Color;
use crate::model::dimensions::Dimensions;
use crate::{Component, Context};

pub struct HasForeground<C: Component> {
    color: Color,
    child: C,
}

impl<C: Component> Component for HasForeground<C> {
    fn render(&self, ctx: &Context) {
        ctx.with_color(&self.color, |ctx| {
            self.child.render(ctx);
        });
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        self.child.measure(ctx)
    }
}

pub trait Foreground<C: Component> {
    fn foreground(self, color: Color) -> HasForeground<C>;
}

impl<C: Component> Foreground<C> for C {
    fn foreground(self, color: Color) -> HasForeground<C> {
        HasForeground { child: self, color }
    }
}
