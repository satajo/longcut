use crate::handle::{GdkHandle, GdkObjectHandle};
use gdk::cairo;
use longcut_gui::model::dimensions::Dimensions;
use longcut_gui::model::position::Position;

pub struct Window {
    gdk_window: gdk::Window,
    dimensions: Dimensions,
}

impl Window {
    pub fn new(
        handle: &mut GdkHandle,
        dimensions: Dimensions,
        position: Position,
    ) -> (GdkObjectHandle, &mut Window) {
        let gdk_window = gdk::Window::new(
            None,
            &gdk::WindowAttr {
                x: Some(position.horizontal as i32),
                y: Some(position.vertical as i32),
                width: dimensions.width as i32,
                height: dimensions.height as i32,
                override_redirect: true,
                type_hint: Some(gdk::WindowTypeHint::Dock),
                ..gdk::WindowAttr::default()
            },
        );

        gdk_window.set_keep_above(true);

        let window = Self {
            gdk_window,
            dimensions,
        };
        handle.windows.insert(window)
    }

    pub fn show(&self, f: impl FnOnce(cairo::Context)) {
        self.gdk_window.show();

        let region = self.gdk_window.visible_region().unwrap();
        let drawing_context = self.gdk_window.begin_draw_frame(&region).unwrap();
        let cairo_context = drawing_context.cairo_context().unwrap();

        // Screen is blanked before beginning draw.
        cairo_context.set_source_rgb(0.0, 0.0, 0.0);
        cairo_context.paint().expect("Cairo Context error");

        f(cairo_context);

        self.gdk_window.end_draw_frame(&drawing_context);
        gdk::flush();
    }

    pub fn hide(&self) {
        if self.gdk_window.is_visible() {
            self.gdk_window.hide();
            gdk::flush();
        }
    }

    pub fn size(&self) -> Dimensions {
        self.dimensions
    }
}
