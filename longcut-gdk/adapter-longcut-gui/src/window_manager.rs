use gdk::cairo;
use gdk::cairo::{FontSlant, FontWeight};
use longcut_gdk::{GdkHandle, GdkObjectHandle, GdkService, Window};
use longcut_graphics_lib::model::alignment::Alignment;
use longcut_graphics_lib::model::color::Color;
use longcut_graphics_lib::model::dimensions::Dimensions;
use longcut_graphics_lib::model::font::Font;
use longcut_graphics_lib::model::position::Position;
use longcut_graphics_lib::port::renderer::Renderer;
use longcut_gui::WindowProperties;
use longcut_gui::port::window_manager::{RenderPassFn, WindowManager};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct GdkWindowManager<'a> {
    gdk: &'a GdkService,
    window_mutex: Arc<Mutex<Option<GdkObjectHandle>>>,
}

impl<'a> GdkWindowManager<'a> {
    pub fn new(gdk: &'a GdkService) -> Self {
        Self {
            gdk,
            window_mutex: Arc::new(Mutex::new(None)),
        }
    }
}

impl GdkWindowManager<'_> {
    fn get_existing_window<'a>(
        handle: &'a mut GdkHandle,
        window_handle_guard: &'a MutexGuard<Option<GdkObjectHandle>>,
    ) -> Option<&'a mut Window> {
        window_handle_guard
            .as_ref()
            .and_then(|window_handle| handle.windows.get_mut(window_handle))
    }

    fn spawn_new_window<'a>(
        handle: &'a mut GdkHandle,
        window_handle_guard: &'a mut MutexGuard<Option<GdkObjectHandle>>,
        requested_properties: &WindowProperties,
    ) -> &'a mut Window {
        let (dimensions, position) =
            GdkWindowManager::calculate_window_geometry(handle, requested_properties);
        let (window_handle, window) = Window::new(
            handle,
            position.horizontal,
            position.vertical,
            dimensions.width,
            dimensions.height,
        );
        let _ = window_handle_guard.insert(window_handle);
        window
    }

    fn calculate_window_geometry(
        handle: &mut GdkHandle,
        requested_properties: &WindowProperties,
    ) -> (Dimensions, Position) {
        let align_position = |alignment: &Alignment, size: u32, max_size: u32| -> i32 {
            (match alignment {
                Alignment::Beginning => 0,
                Alignment::Center => (max_size - size) / 2,
                Alignment::End => max_size - size,
            }) as i32
        };

        let screen_size_raw = handle.get_screen_dimensions();
        let screen_size = Dimensions::new(screen_size_raw.0, screen_size_raw.1);
        let window_size = screen_size.intersect(&requested_properties.size);

        let window_position = Position {
            horizontal: align_position(
                &requested_properties.alignment.horizontal,
                window_size.width,
                screen_size.width,
            ) as u32,
            vertical: align_position(
                &requested_properties.alignment.vertical,
                window_size.height,
                screen_size.height,
            ) as u32,
        };

        (window_size, window_position)
    }
}

impl<'a> WindowManager for GdkWindowManager<'a> {
    fn show_window(&self, requested_properties: WindowProperties, callback: RenderPassFn) {
        let mutex = self.window_mutex.clone();

        self.gdk.run_in_gdk_thread(Box::new(move |handle| {
            let mut guard = mutex.lock().unwrap();

            let window =
                if let Some(existing) = GdkWindowManager::get_existing_window(handle, &guard) {
                    existing
                } else {
                    GdkWindowManager::spawn_new_window(handle, &mut guard, &requested_properties)
                };

            window.show(|cairo| {
                let cairo_renderer = CairoRenderer::new(&cairo);
                let raw_size = window.size();
                let render_area_dimensions = Dimensions::new(raw_size.0, raw_size.1);
                callback(render_area_dimensions, &cairo_renderer);
            });
        }))
    }

    fn hide_window(&self) {
        let mutex = self.window_mutex.clone();
        self.gdk.run_in_gdk_thread(Box::new(move |handle| {
            let guard = mutex.lock().unwrap();
            if let Some(window) = GdkWindowManager::get_existing_window(handle, &guard) {
                window.hide();
            }
        }))
    }
}

// ----------------------------------------------------------------------------
// GraphicsLibRenderer
// ----------------------------------------------------------------------------

/// longcut-graphics-lib [Renderer] implementation, instantiated in [GuiWindowManager] show_window implementation.
#[derive(Debug)]
pub struct CairoRenderer<'a> {
    cairo_context: &'a cairo::Context,
}

impl<'a> CairoRenderer<'a> {
    pub fn new(cairo_context: &'a cairo::Context) -> Self {
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
        Dimensions::new(text_extents.width() as u32, font_extents.height() as u32)
    }
}
