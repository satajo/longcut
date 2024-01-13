use crate::handle::{GdkHandle, GdkObjectHandle};
use gdk::cairo;

pub struct Window {
    gdk_window: gdk::Window,
}

impl Window {
    pub fn new(
        handle: &mut GdkHandle,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> (GdkObjectHandle, &mut Window) {
        let gdk_window = gdk::Window::new(
            None,
            &gdk::WindowAttr {
                x: Some(x as i32),
                y: Some(y as i32),
                width: width as i32,
                height: height as i32,
                override_redirect: true,
                type_hint: Some(gdk::WindowTypeHint::Dock),
                ..gdk::WindowAttr::default()
            },
        );

        gdk_window.set_keep_above(true);

        let window = Self { gdk_window };
        handle.windows.insert(window)
    }

    pub fn show(&self, f: impl FnOnce(cairo::Context)) {
        self.gdk_window.show();

        let region = self.gdk_window.visible_region().unwrap();
        let drawing_context = self.gdk_window.begin_draw_frame(&region).unwrap();

        let cairo_context = drawing_context.cairo_context().unwrap();
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

    pub fn size(&self) -> (u32, u32) {
        (
            self.gdk_window.width() as u32,
            self.gdk_window.height() as u32,
        )
    }
}
