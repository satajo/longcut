use crate::gdk::component::{Component, Context};
use crate::gdk::config::Dimensions;

pub struct Text {
    text: String,
}

impl Text {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl Component for Text {
    fn render(&self, context: &Context) {
        context.show_text(&self.text);
    }

    fn measure(&self, context: &Context) -> Dimensions {
        context.measure_text(&self.text)
    }
}
