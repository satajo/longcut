use crate::handle::{GdkHandle, GdkObjectHandle};
use crate::window::Window;
use crate::GdkModule;
use gdk::cairo;
use longcut_gui::model::alignment::Alignment;
use longcut_gui::model::color::Color;
use longcut_gui::model::dimensions::Dimensions;
use longcut_gui::model::position::Position;
use longcut_gui::port::renderer::{Graphics, RenderPassFn, Renderer, WindowProperties};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct GuiRenderer<'a> {
    gdk: &'a GdkModule,
    window_mutex: Arc<Mutex<Option<GdkObjectHandle>>>,
}

impl<'a> GuiRenderer<'a> {
    pub fn new(gdk: &'a GdkModule) -> Self {
        Self {
            gdk,
            window_mutex: Arc::new(Mutex::new(None)),
        }
    }
}

impl GuiRenderer<'_> {
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
            GuiRenderer::calculate_window_geometry(handle, &requested_properties);
        let (window_handle, window) = Window::new(handle, dimensions, position);
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

        let screen_size = handle.get_screen_dimensions();
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

impl<'a> Graphics for GuiRenderer<'a> {
    fn show_gui(&self, requested_properties: WindowProperties, callback: RenderPassFn) {
        let mutex = self.window_mutex.clone();

        self.gdk.run_in_gdk_thread(Box::new(move |handle| {
            let mut guard = mutex.lock().unwrap();

            let window = if let Some(existing) = GuiRenderer::get_existing_window(handle, &guard) {
                existing
            } else {
                GuiRenderer::spawn_new_window(handle, &mut guard, &requested_properties)
            };

            window.show(|cairo| {
                let cairo_renderer = CairoRenderer::new(&cairo).with_font_size(20);
                callback(window.size(), &cairo_renderer);
            });
        }))
    }

    fn hide_gui(&self) {
        let mutex = self.window_mutex.clone();
        self.gdk.run_in_gdk_thread(Box::new(move |handle| {
            let guard = mutex.lock().unwrap();
            if let Some(window) = GuiRenderer::get_existing_window(handle, &guard) {
                window.hide();
            }
        }))
    }
}

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
