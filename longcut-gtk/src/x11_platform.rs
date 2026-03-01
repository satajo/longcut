use cairo::ImageSurface;
use gdk4_x11::prelude::*;
use gtk4::prelude::*;
use std::ffi::CString;
use x11::xlib;

// Extern declaration for cairo's Xlib surface constructor. This symbol is provided by libcairo,
// which is always linked with Xlib support on X11 (required by GTK4's GDK X11 backend).
unsafe extern "C" {
    fn cairo_xlib_surface_create(
        dpy: *mut xlib::Display,
        drawable: xlib::Drawable,
        visual: *mut xlib::Visual,
        width: i32,
        height: i32,
    ) -> *mut cairo::ffi::cairo_surface_t;
}

/// Handle for painting directly to an X11 window surface, bypassing GTK4's asynchronous
/// rendering pipeline.
///
/// On X11, override-redirect windows are mapped immediately by the X server, before GTK4's
/// frame clock has a chance to render. This handle allows callers to blit pre-rendered content
/// to the window surface right after mapping, replicating the synchronous drawing that GDK3
/// provided via `begin_draw_frame`/`end_draw_frame`.
pub struct X11WindowInfo {
    display: *mut xlib::Display,
    xid: xlib::Window,
    visual: *mut xlib::Visual,
}

impl X11WindowInfo {
    /// Paints the given image surface directly to the X11 window.
    pub fn blit_surface(&self, image: &ImageSurface, width: i32, height: i32) {
        unsafe {
            let ptr = cairo_xlib_surface_create(self.display, self.xid, self.visual, width, height);
            let surface = cairo::Surface::from_raw_full(ptr).expect("X11 surface");
            let cr = cairo::Context::new(&surface).expect("X11 cairo context");
            cr.set_source_surface(image, 0.0, 0.0).expect("source");
            cr.paint().expect("paint");
            drop(cr);
            drop(surface);
            xlib::XFlush(self.display);
        }
    }
}

/// Configures X11-specific window properties after the GTK window has been realized.
///
/// This sets override-redirect, positions the window, marks it as a dock-type window above
/// all others, and disables focus. Returns a handle for direct surface rendering.
pub fn configure_window(gtk_window: &gtk4::Window, x: u32, y: u32) -> X11WindowInfo {
    let surface = gtk_window
        .surface()
        .expect("Window has no surface after realize");
    let x11_surface = surface
        .downcast::<gdk4_x11::X11Surface>()
        .expect("Surface is not X11");
    let xid = x11_surface.xid() as xlib::Window;

    let display = gdk4::Display::default().expect("No default display");
    let x11_display = display
        .downcast::<gdk4_x11::X11Display>()
        .expect("Display is not X11");
    let x_display = unsafe { x11_display.xdisplay() };

    let visual = unsafe {
        let mut attrs: xlib::XWindowAttributes = std::mem::zeroed();
        xlib::XGetWindowAttributes(x_display, xid, &mut attrs);
        attrs.visual
    };

    unsafe {
        // Bypass the window manager entirely (equivalent to override_redirect in GDK3).
        let mut attrs: xlib::XSetWindowAttributes = std::mem::zeroed();
        attrs.override_redirect = 1;
        xlib::XChangeWindowAttributes(x_display, xid, xlib::CWOverrideRedirect, &mut attrs);

        // Position the window (GTK4 removed programmatic window positioning).
        xlib::XMoveWindow(x_display, xid, x as i32, y as i32);

        // Set _NET_WM_WINDOW_TYPE to DOCK.
        let type_name = CString::new("_NET_WM_WINDOW_TYPE").unwrap();
        let wm_type = xlib::XInternAtom(x_display, type_name.as_ptr(), 0);
        let dock_name = CString::new("_NET_WM_WINDOW_TYPE_DOCK").unwrap();
        let dock_type = xlib::XInternAtom(x_display, dock_name.as_ptr(), 0);
        xlib::XChangeProperty(
            x_display,
            xid,
            wm_type,
            xlib::XA_ATOM,
            32,
            xlib::PropModeReplace,
            &dock_type as *const xlib::Atom as *const u8,
            1,
        );

        // Set _NET_WM_STATE to ABOVE.
        let state_name = CString::new("_NET_WM_STATE").unwrap();
        let wm_state = xlib::XInternAtom(x_display, state_name.as_ptr(), 0);
        let above_name = CString::new("_NET_WM_STATE_ABOVE").unwrap();
        let above_state = xlib::XInternAtom(x_display, above_name.as_ptr(), 0);
        xlib::XChangeProperty(
            x_display,
            xid,
            wm_state,
            xlib::XA_ATOM,
            32,
            xlib::PropModeReplace,
            &above_state as *const xlib::Atom as *const u8,
            1,
        );

        // Don't accept focus.
        let mut hints: xlib::XWMHints = std::mem::zeroed();
        hints.flags = xlib::InputHint;
        hints.input = 0;
        xlib::XSetWMHints(x_display, xid, &mut hints);

        xlib::XFlush(x_display);
    }

    X11WindowInfo {
        display: x_display,
        xid,
        visual,
    }
}
