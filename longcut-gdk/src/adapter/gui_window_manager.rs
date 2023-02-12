use crate::adapter::graphics_lib_renderer::GraphicsLibRenderer;
use crate::handle::{GdkHandle, GdkObjectHandle};
use crate::window::Window;
use crate::GdkModule;
use longcut_graphics_lib::model::alignment::Alignment;
use longcut_graphics_lib::model::dimensions::Dimensions;
use longcut_graphics_lib::model::position::Position;
use longcut_gui::port::window_manager::{RenderPassFn, WindowManager};
use longcut_gui::window_properties::WindowProperties;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct GuiWindowManager<'a> {
    gdk: &'a GdkModule,
    window_mutex: Arc<Mutex<Option<GdkObjectHandle>>>,
}

impl<'a> GuiWindowManager<'a> {
    pub fn new(gdk: &'a GdkModule) -> Self {
        Self {
            gdk,
            window_mutex: Arc::new(Mutex::new(None)),
        }
    }
}

impl GuiWindowManager<'_> {
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
            GuiWindowManager::calculate_window_geometry(handle, requested_properties);
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

impl<'a> WindowManager for GuiWindowManager<'a> {
    fn show_window(&self, requested_properties: WindowProperties, callback: RenderPassFn) {
        let mutex = self.window_mutex.clone();

        self.gdk.run_in_gdk_thread(Box::new(move |handle| {
            let mut guard = mutex.lock().unwrap();

            let window =
                if let Some(existing) = GuiWindowManager::get_existing_window(handle, &guard) {
                    existing
                } else {
                    GuiWindowManager::spawn_new_window(handle, &mut guard, &requested_properties)
                };

            window.show(|cairo| {
                let cairo_renderer = GraphicsLibRenderer::new(&cairo);
                callback(window.size(), &cairo_renderer);
            });
        }))
    }

    fn hide_window(&self) {
        let mutex = self.window_mutex.clone();
        self.gdk.run_in_gdk_thread(Box::new(move |handle| {
            let guard = mutex.lock().unwrap();
            if let Some(window) = GuiWindowManager::get_existing_window(handle, &guard) {
                window.hide();
            }
        }))
    }
}
