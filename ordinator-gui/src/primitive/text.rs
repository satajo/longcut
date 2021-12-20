use crate::model::dimensions::Dimensions;
use crate::{Component, Context};

pub struct Text(String);

impl Text {
    pub fn new(text: String) -> Self {
        Self(text)
    }
}

impl Component for Text {
    fn render(&self, ctx: &Context) {
        let mut text = self.0.clone();
        while !text.is_empty() && ctx.measure_text(&text).width > ctx.region.width {
            text.pop();
        }
        ctx.draw_text(&text)
    }

    fn measure(&self, ctx: &Context) -> Dimensions {
        ctx.measure_text(&self.0)
    }
}
