use gdk::cairo;
use ordinator_gui::model::color::Color;
use ordinator_gui::model::dimensions::Dimensions;
use ordinator_gui::model::position::Position;
use ordinator_gui::port::renderer::Renderer;

#[derive(Debug)]
pub struct CairoRenderer<'a> {
    cairo_context: &'a cairo::Context,
    font_size: u32,
}

impl<'a> CairoRenderer<'a> {
    pub fn new(cairo_context: &'a cairo::Context) -> Self {
        CairoRenderer {
            cairo_context,
            font_size: 0,
        }
    }

    pub fn with_font_size(&mut self, font_size: u32) -> Self {
        self.cairo_context.set_font_size(font_size as f64);
        Self { font_size, ..*self }
    }

    fn set_draw_color(&self, color: &Color) {
        self.cairo_context
            .set_source_rgba(color.red, color.green, color.blue, color.alpha);
    }
}

impl<'a> Renderer for CairoRenderer<'a> {
    fn draw_rectangle(&self, color: &Color, position: &Position, size: &Dimensions) {
        self.set_draw_color(color);
        self.cairo_context.rectangle(
            position.horizontal as f64,
            position.vertical as f64,
            size.width as f64,
            size.height as f64,
        );
        self.cairo_context.fill().unwrap();
    }

    fn draw_text(&self, color: &Color, position: &Position, text: &str) {
        self.set_draw_color(color);

        // Cairo renders the text above the set position, but Gui renders it below the position.
        self.cairo_context.move_to(
            position.horizontal as f64,
            (position.vertical + self.font_size) as f64,
        );
        self.cairo_context.show_text(text).unwrap();
    }

    fn measure_text(&self, text: &str) -> Dimensions {
        let font_extents = self.cairo_context.font_extents().unwrap();
        let text_extents = self.cairo_context.text_extents(text).unwrap();
        Dimensions::new(text_extents.width as u32, font_extents.height as u32)
    }
}
