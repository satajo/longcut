use crate::component::Component;
use crate::context::Context;
use crate::model::font::Font;
use crate::property::{Background, Border, FontStyle, Foreground, Margin, Property};
use crate::{Color, Dimensions};

pub struct Root<C: Component> {
    child: Foreground<FontStyle<Border<Background<Margin<C>>>>>,
}

impl<C: Component> Root<C> {
    pub fn new(background: Color, foreground: Color, font: Font, border: Color, child: C) -> Self {
        let child = child
            .margin(20)
            .background(background)
            .border(1, border)
            .font_style(font)
            .foreground(foreground);

        Self { child }
    }
}

impl<C: Component> Component for Root<C> {
    fn render(&self, ctx: &Context) {
        self.child.render(ctx)
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        self.child.measure(ctx)
    }
}
