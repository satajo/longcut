use crate::visual::{CXcbVisualtype, find_argb_visual, find_root_visual};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ColormapAlloc, ConfigureWindowAux, ConnectionExt, CreateWindowAux, PropMode, Screen,
    StackMode, Visualtype, WindowClass,
};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;
use x11rb::xcb_ffi::XCBConnection;

/// The placement of a window in the integer types the X protocol carries it in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowGeometry {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

/// An X11 window managed via XCB.
///
/// Owns the X11 window and colormap resources, which are freed on drop.
#[derive(Debug)]
pub struct Window<'a> {
    conn: &'a XCBConnection,
    id: u32,
    colormap: u32,
    geometry: WindowGeometry,
    visual: Visualtype,
}

impl<'a> Window<'a> {
    /// # Panics
    ///
    /// Panics if window or colormap creation fails.
    pub fn new(conn: &'a XCBConnection, screen: &Screen, geometry: WindowGeometry) -> Self {
        let (visual, depth) = if let Some(v) = find_argb_visual(screen) {
            (v, 32u8)
        } else {
            (
                find_root_visual(screen).expect("No root visual found"),
                screen.root_depth,
            )
        };

        let colormap = conn.generate_id().expect("Failed to generate colormap ID");
        conn.create_colormap(ColormapAlloc::NONE, colormap, screen.root, visual.visual_id)
            .expect("Failed to create colormap");

        let window_id = conn.generate_id().expect("Failed to generate window ID");
        let win_aux = CreateWindowAux::new()
            .border_pixel(0)
            .override_redirect(1)
            .colormap(colormap);

        conn.create_window(
            depth,
            window_id,
            screen.root,
            geometry.x,
            geometry.y,
            geometry.width,
            geometry.height,
            0,
            WindowClass::INPUT_OUTPUT,
            visual.visual_id,
            &win_aux,
        )
        .expect("Failed to create window");

        // Set _NET_WM_WINDOW_TYPE = _NET_WM_WINDOW_TYPE_DOCK
        let wm_type = intern_atom(conn, b"_NET_WM_WINDOW_TYPE");
        let dock_type = intern_atom(conn, b"_NET_WM_WINDOW_TYPE_DOCK");
        conn.change_property32(
            PropMode::REPLACE,
            window_id,
            wm_type,
            AtomEnum::ATOM,
            &[dock_type],
        )
        .expect("Failed to set window type");

        // Set _NET_WM_STATE = _NET_WM_STATE_ABOVE
        let wm_state = intern_atom(conn, b"_NET_WM_STATE");
        let above_state = intern_atom(conn, b"_NET_WM_STATE_ABOVE");
        conn.change_property32(
            PropMode::REPLACE,
            window_id,
            wm_state,
            AtomEnum::ATOM,
            &[above_state],
        )
        .expect("Failed to set window state");

        // Set WM_HINTS with input = false (no-focus).
        // WM_HINTS format: flags(u32), input(u32), initial_state(u32), ...
        // InputHint flag = bit 0 (value 1), input = 0 (don't take focus).
        // Per ICCCM, WM_HINTS is a self-typed property (type atom = property atom).
        let wm_hints = intern_atom(conn, b"WM_HINTS");
        let hints_data: [u32; 9] = [1, 0, 0, 0, 0, 0, 0, 0, 0];
        conn.change_property32(
            PropMode::REPLACE,
            window_id,
            wm_hints,
            wm_hints,
            &hints_data,
        )
        .expect("Failed to set WM hints");

        conn.flush().expect("Failed to flush connection");

        Window {
            conn,
            id: window_id,
            colormap,
            geometry,
            visual,
        }
    }

    /// # Panics
    ///
    /// Panics if the rendering surface or context cannot be created.
    pub fn show(&self, render_fn: impl FnOnce(&cairo::Context, u32, u32)) {
        let w = i32::from(self.geometry.width);
        let h = i32::from(self.geometry.height);

        // Render to an off-screen ImageSurface.
        let image = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h).expect("ImageSurface");
        {
            let cr = cairo::Context::new(&image).expect("cairo context");
            render_fn(
                &cr,
                u32::from(self.geometry.width),
                u32::from(self.geometry.height),
            );
        }

        // Map the window first so the compositor redirects it and allocates its buffer.
        self.conn.map_window(self.id).expect("Failed to map window");

        // Raise the window to the top of the stacking order. Override-redirect windows are not
        // managed by the window manager, so EWMH hints like _NET_WM_STATE_ABOVE have no effect.
        // We must explicitly raise the window each time it is shown.
        self.conn
            .configure_window(
                self.id,
                &ConfigureWindowAux::new().stack_mode(StackMode::ABOVE),
            )
            .expect("Failed to raise window");

        self.conn.flush().expect("Failed to flush");

        // Blit the pre-rendered content to the now-mapped window via a cairo XCB surface.
        let surface = self.create_xcb_surface(w, h);
        let cr = cairo::Context::new(&surface).expect("blit context");
        cr.set_source_surface(&image, 0.0, 0.0).expect("source");
        cr.paint().expect("paint");
        drop(cr);
        surface.flush();
        drop(surface);
        self.conn.flush().expect("Failed to flush");
    }

    /// # Panics
    ///
    /// Panics if the window cannot be unmapped.
    pub fn hide(&self) {
        self.conn
            .unmap_window(self.id)
            .expect("Failed to unmap window");
        self.conn.flush().expect("Failed to flush");
    }

    #[must_use]
    pub fn geometry(&self) -> WindowGeometry {
        self.geometry
    }

    /// Creates a cairo XCB surface targeting this window's drawable.
    ///
    /// All unsafe bridging between x11rb and cairo is confined here: the `CXcbVisualtype` lives
    /// on the stack and is guaranteed to outlive the `XCBVisualType` pointer wrapper derived
    /// from it.
    fn create_xcb_surface(&self, width: i32, height: i32) -> cairo::XCBSurface {
        let raw_conn = self.conn.get_raw_xcb_connection();
        #[expect(
            unsafe_code,
            reason = "cairo takes the libxcb connection as a raw pointer"
        )]
        // SAFETY: x11rb's XCBConnection wraps the same libxcb xcb_connection_t that cairo
        // expects. The connection is owned by XcbService and outlives this surface usage.
        let xcb_conn = unsafe { cairo::XCBConnection::from_raw_none(raw_conn.cast()) };
        let xcb_drawable = cairo::XCBDrawable(self.id);

        let mut c_visual = CXcbVisualtype::from_x11rb(&self.visual);
        #[expect(unsafe_code, reason = "cairo takes the visual type as a raw pointer")]
        // SAFETY: CXcbVisualtype is #[repr(C)] and matches the layout of xcb_visualtype_t.
        // c_visual is stack-local and outlives xcb_visual and the surface creation below.
        let xcb_visual = unsafe {
            cairo::XCBVisualType::from_raw_none(
                (&raw mut c_visual).cast::<cairo::ffi::xcb_visualtype_t>(),
            )
        };

        cairo::XCBSurface::create(&xcb_conn, &xcb_drawable, &xcb_visual, width, height)
            .expect("XCB surface")
    }
}

impl Drop for Window<'_> {
    fn drop(&mut self) {
        #[expect(
            clippy::let_underscore_must_use,
            reason = "the X server destroys the window itself when the connection closes"
        )]
        let _ = self.conn.destroy_window(self.id);
        #[expect(
            clippy::let_underscore_must_use,
            reason = "the X server frees the colormap itself when the connection closes"
        )]
        let _ = self.conn.free_colormap(self.colormap);
        #[expect(
            clippy::let_underscore_must_use,
            reason = "a flush fails only once the connection is gone, and then nothing is left to release"
        )]
        let _ = self.conn.flush();
    }
}

fn intern_atom(conn: &XCBConnection, name: &[u8]) -> u32 {
    conn.intern_atom(false, name)
        .expect("Failed to send intern atom request")
        .reply()
        .expect("Failed to intern atom")
        .atom
}
