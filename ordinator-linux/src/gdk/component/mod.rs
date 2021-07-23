pub mod column;
pub mod rectangle;
pub mod row;
pub mod text;

use crate::gdk::config::Dimensions;
use gdk::cairo;

#[derive(Debug)]
pub struct Context<'a> {
    cairo_context: &'a cairo::Context,
    offset: Dimensions,
    font_size: u32,
}

impl<'a> Context<'a> {
    pub fn new(cairo_context: &'a cairo::Context) -> Self {
        Context {
            cairo_context,
            offset: Dimensions::default(),
            font_size: 0,
        }
    }

    pub fn with_font_size(&mut self, font_size: u32) -> Self {
        self.cairo_context.set_font_size(font_size as f64);
        Self { font_size, ..*self }
    }

    fn offset(&self, dimensions: Dimensions) -> Self {
        Self {
            offset: self.offset + dimensions,
            ..*self
        }
    }

    fn show_text(&self, text: &str) {
        self.cairo_context.move_to(
            self.offset.horizontal as f64,
            (self.offset.vertical + self.font_size) as f64,
        );
        self.cairo_context.show_text(text).unwrap();
    }

    fn measure_text(&self, text: &str) -> Dimensions {
        let font_extents = self.cairo_context.font_extents().unwrap();
        let text_extents = self.cairo_context.text_extents(text).unwrap();
        Dimensions {
            horizontal: text_extents.width as u32,
            vertical: font_extents.height as u32,
        }
    }
}

pub trait Component {
    fn render(&self, context: &Context);

    fn measure(&self, context: &Context) -> Dimensions;
}

impl Component for Box<dyn Component> {
    fn render(&self, context: &Context) {
        self.as_ref().render(context)
    }

    fn measure(&self, context: &Context) -> Dimensions {
        self.as_ref().measure(context)
    }
}
