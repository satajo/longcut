use cairo::{FontSlant, FontWeight};
use longcut_graphics_lib::model::alignment::Alignment;
use longcut_graphics_lib::model::color::Color;
use longcut_graphics_lib::model::dimensions::Dimensions;
use longcut_graphics_lib::model::font::Font;
use longcut_graphics_lib::model::position::Position;
use longcut_graphics_lib::port::renderer::Renderer;
use longcut_gui::WindowProperties;
use longcut_gui::port::window_manager::{RenderPassFn, WindowManager};
use longcut_xcb::{Window, XcbService};
use std::cell::RefCell;

pub struct XcbWindowManager<'a> {
    xcb: &'a XcbService,
    window: RefCell<Option<Window<'a>>>,
}

impl<'a> XcbWindowManager<'a> {
    pub fn new(xcb: &'a XcbService) -> Self {
        Self {
            xcb,
            window: RefCell::new(None),
        }
    }

    fn calculate_window_geometry(
        &self,
        requested_properties: &WindowProperties,
    ) -> (Dimensions, Position) {
        let align_position = |alignment: &Alignment, size: u32, max_size: u32| -> u32 {
            match alignment {
                Alignment::Beginning => 0,
                Alignment::Center => (max_size - size) / 2,
                Alignment::End => max_size - size,
            }
        };

        let screen_size_raw = self.xcb.get_screen_dimensions();
        let screen_size = Dimensions::new(screen_size_raw.0, screen_size_raw.1);
        let window_size = screen_size.intersect(&requested_properties.size);

        let window_position = Position {
            horizontal: align_position(
                &requested_properties.alignment.horizontal,
                window_size.width,
                screen_size.width,
            ),
            vertical: align_position(
                &requested_properties.alignment.vertical,
                window_size.height,
                screen_size.height,
            ),
        };

        (window_size, window_position)
    }
}

impl WindowManager for XcbWindowManager<'_> {
    fn show_window(&self, requested_properties: WindowProperties, callback: RenderPassFn) {
        let (dimensions, position) = self.calculate_window_geometry(&requested_properties);

        let mut window_opt = self.window.borrow_mut();

        // Recreate the window if geometry has changed.
        if let Some(window) = window_opt.as_ref() {
            let (w, h) = window.size();
            let (x, y) = window.position();
            if w != dimensions.width
                || h != dimensions.height
                || x != position.horizontal
                || y != position.vertical
            {
                *window_opt = None;
            }
        }

        if window_opt.is_none() {
            *window_opt = Some(self.xcb.create_window(
                position.horizontal,
                position.vertical,
                dimensions.width,
                dimensions.height,
            ));
        }

        let window = window_opt.as_ref().unwrap();
        let (w, h) = window.size();

        window.show(move |cr, _w, _h| {
            let cairo_renderer = CairoRenderer::new(cr);
            let render_area_dimensions = Dimensions::new(w, h);
            callback(render_area_dimensions, &cairo_renderer);
        });
    }

    fn hide_window(&self) {
        let window_opt = self.window.borrow();
        if let Some(window) = window_opt.as_ref() {
            window.hide();
        }
    }
}

// ----------------------------------------------------------------------------
// CairoRenderer
// ----------------------------------------------------------------------------

#[derive(Debug)]
struct CairoRenderer<'a> {
    cairo_context: &'a cairo::Context,
}

impl<'a> CairoRenderer<'a> {
    fn new(cairo_context: &'a cairo::Context) -> Self {
        CairoRenderer { cairo_context }
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

impl Renderer for CairoRenderer<'_> {
    fn draw_rectangle(&self, color: &Color, position: &Position, size: &Dimensions) {
        self.set_draw_color(color);
        self.cairo_context.rectangle(
            f64::from(position.horizontal),
            f64::from(position.vertical),
            f64::from(size.width),
            f64::from(size.height),
        );
        self.cairo_context.fill().unwrap();
    }

    fn draw_text(&self, color: &Color, position: &Position, font: &Font, text: &str) {
        self.set_draw_color(color);
        self.set_font_family(&font.family);
        self.set_font_size(f64::from(font.size));

        // Cairo renders the text above the set position, but Gui renders it below the position.
        self.cairo_context.move_to(
            f64::from(position.horizontal),
            f64::from(position.vertical + u32::from(font.size)),
        );
        self.cairo_context.show_text(text).unwrap();
    }

    fn measure_text(&self, font: &Font, text: &str) -> Dimensions {
        self.set_font_family(&font.family);
        self.set_font_size(f64::from(font.size));
        let font_extents = self.cairo_context.font_extents().unwrap();
        let text_extents = self.cairo_context.text_extents(text).unwrap();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "cairo pixel measurements are always small positive values"
        )]
        Dimensions::new(text_extents.width() as u32, font_extents.height() as u32)
    }
}
