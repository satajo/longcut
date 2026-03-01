use crate::handle::{GtkHandle, GtkObjectHandle};
use crate::x11_platform;
use cairo::ImageSurface;
use gtk4::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Window {
    gtk_window: gtk4::Window,
    drawing_area: gtk4::DrawingArea,
    content: Rc<RefCell<Option<ImageSurface>>>,
    x11_info: x11_platform::X11WindowInfo,
}

impl Window {
    pub fn new(
        handle: &mut GtkHandle,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> (GtkObjectHandle, &mut Window) {
        let gtk_window = gtk4::Window::new();
        gtk_window.set_decorated(false);
        gtk_window.set_resizable(false);
        gtk_window.set_default_size(width as i32, height as i32);

        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.set_content_width(width as i32);
        drawing_area.set_content_height(height as i32);
        gtk_window.set_child(Some(&drawing_area));

        let content: Rc<RefCell<Option<ImageSurface>>> = Rc::new(RefCell::new(None));

        let content_for_draw = content.clone();
        drawing_area.set_draw_func(move |_area, cr, _w, _h| {
            if let Some(ref image) = *content_for_draw.borrow() {
                let _ = cr.set_source_surface(image, 0.0, 0.0);
                let _ = cr.paint();
            }
        });

        // Realize the window to create the native surface, then apply platform-specific configuration.
        gtk4::prelude::WidgetExt::realize(&gtk_window);
        let x11_info = x11_platform::configure_window(&gtk_window, x, y);

        let window = Self {
            gtk_window,
            drawing_area,
            content,
            x11_info,
        };
        handle.windows.insert(window)
    }

    pub fn show(&self, f: impl FnOnce(&cairo::Context, i32, i32) + 'static) {
        let (w, h) = self.size();

        // Pre-render content to an off-screen image surface.
        let image =
            ImageSurface::create(cairo::Format::ARgb32, w as i32, h as i32).expect("surface");
        {
            let cr = cairo::Context::new(&image).expect("context");
            f(&cr, w as i32, h as i32);
        }
        *self.content.borrow_mut() = Some(image);

        // Make the window visible and schedule a GTK redraw for future frame-clock repaints.
        self.drawing_area.queue_draw();
        self.gtk_window.set_visible(true);

        // Paint the pre-rendered content directly to the native window surface. GTK4 removed the
        // synchronous begin_draw_frame/end_draw_frame API from GDK3, so override-redirect windows
        // are mapped before the first frame-clock paint. This direct blit fills the gap.
        if let Some(ref image) = *self.content.borrow() {
            self.x11_info.blit_surface(image, w as i32, h as i32);
        }
    }

    pub fn hide(&self) {
        self.gtk_window.set_visible(false);
        *self.content.borrow_mut() = None;
    }

    pub fn size(&self) -> (u32, u32) {
        (
            self.drawing_area.content_width() as u32,
            self.drawing_area.content_height() as u32,
        )
    }
}
