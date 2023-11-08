use longcut_graphics_lib::component::Component;
use longcut_graphics_lib::context::Context;
use longcut_graphics_lib::model::color::Color;
use longcut_graphics_lib::model::dimensions::Dimensions;
use longcut_graphics_lib::model::font::Font;
use longcut_graphics_lib::property::{Background, Border, FontStyle, Foreground, Margin, Property};

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
