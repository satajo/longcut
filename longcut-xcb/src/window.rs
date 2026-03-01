use crate::visual::{CXcbVisualtype, find_argb_visual, find_root_visual};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ColormapAlloc, ConnectionExt, CreateWindowAux, PropMode, Screen, Visualtype,
    WindowClass,
};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;
use x11rb::xcb_ffi::XCBConnection;

/// An X11 window managed via XCB.
///
/// Owns the X11 window and colormap resources, which are freed on drop.
pub struct Window<'a> {
    conn: &'a XCBConnection,
    id: u32,
    colormap: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    visual: Visualtype,
}

impl<'a> Window<'a> {
    pub fn new(
        conn: &'a XCBConnection,
        screen: &Screen,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> Self {
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
            x,
            y,
            width,
            height,
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
            x: x as u32,
            y: y as u32,
            width: width as u32,
            height: height as u32,
            visual,
        }
    }

    pub fn show(&self, render_fn: impl FnOnce(&cairo::Context, u32, u32)) {
        let w = self.width as i32;
        let h = self.height as i32;

        // Render to an off-screen ImageSurface.
        let image = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h).expect("ImageSurface");
        {
            let cr = cairo::Context::new(&image).expect("cairo context");
            render_fn(&cr, self.width, self.height);
        }

        // Map the window first so the compositor redirects it and allocates its buffer.
        self.conn.map_window(self.id).expect("Failed to map window");
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

    pub fn hide(&self) {
        self.conn
            .unmap_window(self.id)
            .expect("Failed to unmap window");
        self.conn.flush().expect("Failed to flush");
    }

    pub fn position(&self) -> (u32, u32) {
        (self.x, self.y)
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Creates a cairo XCB surface targeting this window's drawable.
    ///
    /// All unsafe bridging between x11rb and cairo is confined here: the `CXcbVisualtype` lives
    /// on the stack and is guaranteed to outlive the `XCBVisualType` pointer wrapper derived
    /// from it.
    fn create_xcb_surface(&self, width: i32, height: i32) -> cairo::XCBSurface {
        let raw_conn = self.conn.get_raw_xcb_connection();
        // SAFETY: x11rb's XCBConnection wraps the same libxcb xcb_connection_t that cairo
        // expects. The connection is owned by XcbService and outlives this surface usage.
        let xcb_conn = unsafe { cairo::XCBConnection::from_raw_none(raw_conn as *mut _) };
        let xcb_drawable = cairo::XCBDrawable(self.id);

        // SAFETY: CXcbVisualtype is #[repr(C)] and matches the layout of xcb_visualtype_t.
        // c_visual is stack-local and outlives xcb_visual and the surface creation below.
        let mut c_visual = CXcbVisualtype::from_x11rb(&self.visual);
        let xcb_visual = unsafe {
            cairo::XCBVisualType::from_raw_none(
                &mut c_visual as *mut CXcbVisualtype as *mut cairo::ffi::xcb_visualtype_t,
            )
        };

        cairo::XCBSurface::create(&xcb_conn, &xcb_drawable, &xcb_visual, width, height)
            .expect("XCB surface")
    }
}

impl Drop for Window<'_> {
    fn drop(&mut self) {
        let _ = self.conn.destroy_window(self.id);
        let _ = self.conn.free_colormap(self.colormap);
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
