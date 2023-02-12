use gdk::cairo;
use gdk::cairo::{FontSlant, FontWeight};
use longcut_graphics_lib::model::color::Color;
use longcut_graphics_lib::model::dimensions::Dimensions;
use longcut_graphics_lib::model::font::Font;
use longcut_graphics_lib::model::position::Position;
use longcut_graphics_lib::port::renderer::Renderer;

#[derive(Debug)]
pub struct GraphicsLibRenderer<'a> {
    cairo_context: &'a cairo::Context,
}

impl<'a> GraphicsLibRenderer<'a> {
    pub fn new(cairo_context: &'a cairo::Context) -> Self {
        GraphicsLibRenderer { cairo_context }
    }

    fn set_font_family(&self, font_family: &str) {
        self.cairo_context
            .select_font_face(font_family, FontSlant::Normal, FontWeight::Normal);
    }

    fn set_font_size(&self, font_size: f64) {
        self.cairo_context.set_font_size(font_size);
    }

    fn set_draw_color(&self, color: &Color) {
        self.cairo_context
            .set_source_rgba(color.red, color.green, color.blue, color.alpha);
    }
}

impl<'a> Renderer for GraphicsLibRenderer<'a> {
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

    fn draw_text(&self, color: &Color, position: &Position, font: &Font, text: &str) {
        self.set_draw_color(color);
        self.set_font_family(&font.family);
        self.set_font_size(font.size as f64);

        // Cairo renders the text above the set position, but Gui renders it below the position.
        self.cairo_context.move_to(
            position.horizontal as f64,
            (position.vertical + font.size as u32) as f64,
        );
        self.cairo_context.show_text(text).unwrap();
    }

    fn measure_text(&self, font: &Font, text: &str) -> Dimensions {
        self.set_font_family(&font.family);
        self.set_font_size(font.size as f64);
        let font_extents = self.cairo_context.font_extents().unwrap();
        let text_extents = self.cairo_context.text_extents(text).unwrap();
        Dimensions::new(text_extents.width as u32, font_extents.height as u32)
    }
}
