use cairo::{FontSlant, FontWeight};
use longcut_graphics_lib::model::alignment::Alignment;
use longcut_graphics_lib::model::color::Color;
use longcut_graphics_lib::model::dimensions::Dimensions;
use longcut_graphics_lib::model::font::Font;
use longcut_graphics_lib::model::position::Position;
use longcut_graphics_lib::port::renderer::Renderer;
use longcut_gui::WindowProperties;
use longcut_gui::port::window_manager::{RenderPassFn, WindowManager};
use longcut_xcb::{Window, WindowGeometry, XcbService};
use std::cell::RefCell;

#[derive(Debug)]
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

    /// A window that the X protocol cannot place in full is cut at the largest coordinate and
    /// size the protocol carries.
    fn calculate_window_geometry(&self, requested_properties: &WindowProperties) -> WindowGeometry {
        let screen_size_raw = self.xcb.get_screen_dimensions();
        let screen_size = Dimensions::new(screen_size_raw.0, screen_size_raw.1);
        let window_size = screen_size.intersect(requested_properties.size);

        let horizontal = align_position(
            requested_properties.alignment.horizontal,
            window_size.width,
            screen_size.width,
        );
        let vertical = align_position(
            requested_properties.alignment.vertical,
            window_size.height,
            screen_size.height,
        );

        WindowGeometry {
            x: i16::try_from(horizontal).unwrap_or(i16::MAX),
            y: i16::try_from(vertical).unwrap_or(i16::MAX),
            width: u16::try_from(window_size.width).unwrap_or(u16::MAX),
            height: u16::try_from(window_size.height).unwrap_or(u16::MAX),
        }
    }
}

impl WindowManager for XcbWindowManager<'_> {
    fn show_window(&self, requested_properties: WindowProperties, callback: RenderPassFn) {
        let geometry = self.calculate_window_geometry(&requested_properties);

        let mut window_opt = self.window.borrow_mut();

        // Recreate the window if geometry has changed.
        if window_opt
            .as_ref()
            .is_some_and(|window| window.geometry() != geometry)
        {
            *window_opt = None;
        }

        let window = window_opt.get_or_insert_with(|| self.xcb.create_window(geometry));

        window.show(move |cr, width, height| {
            let cairo_renderer = CairoRenderer::new(cr);
            callback(Dimensions::new(width, height), &cairo_renderer);
        });
    }

    fn hide_window(&self) {
        let window_opt = self.window.borrow();
        if let Some(window) = window_opt.as_ref() {
            window.hide();
        }
    }
}

/// The start coordinate of a `size` long span aligned inside a `max_size` long extent. A span
/// longer than the extent starts at 0, pinned to the start edge, whatever its alignment.
fn align_position(alignment: Alignment, size: u32, max_size: u32) -> u32 {
    let slack = max_size.saturating_sub(size);
    match alignment {
        Alignment::Beginning => 0,
        Alignment::Center => slack / 2,
        Alignment::End => slack,
    }
}

// ----------------------------------------------------------------------------
// CairoRenderer
// ----------------------------------------------------------------------------

/// Cairo reports an error only from a context in an error state, and the window hands out a
/// context on an image surface it has just created.
const CAIRO_CONTEXT_IS_VALID: &str =
    "the cairo context draws on the image surface the window created";

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
    fn draw_rectangle(&self, color: &Color, position: Position, size: Dimensions) {
        self.set_draw_color(color);
        self.cairo_context.rectangle(
            f64::from(position.horizontal),
            f64::from(position.vertical),
            f64::from(size.width),
            f64::from(size.height),
        );
        self.cairo_context.fill().expect(CAIRO_CONTEXT_IS_VALID);
    }

    fn draw_text(&self, color: &Color, position: Position, font: &Font, text: &str) {
        self.set_draw_color(color);
        self.set_font_family(&font.family);
        self.set_font_size(f64::from(font.size));

        // Cairo renders the text above the set position, but Gui renders it below the position.
        self.cairo_context.move_to(
            f64::from(position.horizontal),
            f64::from(position.vertical.saturating_add(u32::from(font.size))),
        );
        self.cairo_context
            .show_text(text)
            .expect(CAIRO_CONTEXT_IS_VALID);
    }

    fn measure_text(&self, font: &Font, text: &str) -> Dimensions {
        self.set_font_family(&font.family);
        self.set_font_size(f64::from(font.size));
        let font_extents = self
            .cairo_context
            .font_extents()
            .expect(CAIRO_CONTEXT_IS_VALID);
        let text_extents = self
            .cairo_context
            .text_extents(text)
            .expect(CAIRO_CONTEXT_IS_VALID);
        Dimensions::new(pixels(text_extents.width()), pixels(font_extents.height()))
    }
}

/// Rounds a cairo measurement to whole pixels.
fn pixels(measure: f64) -> u32 {
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "cairo reports text and font extents as finite, non-negative pixel counts of a valid context, far below u32::MAX"
    )]
    let pixels = measure.round() as u32;
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_that_fits_is_placed_by_its_alignment() {
        assert_eq!(align_position(Alignment::Beginning, 10, 100), 0);
        assert_eq!(align_position(Alignment::Center, 10, 100), 45);
        assert_eq!(align_position(Alignment::End, 10, 100), 90);
    }

    #[test]
    fn a_span_longer_than_its_extent_is_pinned_to_the_start_edge() {
        assert_eq!(align_position(Alignment::Beginning, 200, 100), 0);
        assert_eq!(align_position(Alignment::Center, 200, 100), 0);
        assert_eq!(align_position(Alignment::End, 200, 100), 0);
    }
}
