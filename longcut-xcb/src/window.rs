use crate::visual::{CXcbVisualtype, find_argb_visual, find_root_visual};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ColormapAlloc, ConnectionExt, CreateWindowAux, EventMask, PropMode, Screen,
    Visualtype, WindowClass,
};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;
use x11rb::xcb_ffi::XCBConnection;

pub struct Window {
    id: u32,
    width: u32,
    height: u32,
    visual: Visualtype,
    depth: u8,
}

impl Window {
    pub fn new(
        conn: &XCBConnection,
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
            .colormap(colormap)
            .event_mask(EventMask::EXPOSURE);

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

        // Set WM_HINTS with input = false (no-focus)
        let wm_hints = intern_atom(conn, b"WM_HINTS");
        // WM_HINTS format: flags(u32), input(u32), initial_state(u32), ...
        // InputHint flag = bit 0 (value 1), input = 0 (don't take focus)
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
            id: window_id,
            width: width as u32,
            height: height as u32,
            visual,
            depth,
        }
    }

    pub fn show(&self, conn: &XCBConnection, render_fn: impl FnOnce(&cairo::Context, u32, u32)) {
        let w = self.width as i32;
        let h = self.height as i32;

        // Render to an off-screen ImageSurface.
        let image = cairo::ImageSurface::create(cairo::Format::ARgb32, w, h).expect("ImageSurface");
        {
            let cr = cairo::Context::new(&image).expect("cairo context");
            render_fn(&cr, self.width, self.height);
        }

        // Map the window first so the compositor redirects it and allocates its buffer.
        conn.map_window(self.id).expect("Failed to map window");
        conn.flush().expect("Failed to flush");

        // Blit the pre-rendered content to the now-mapped window via a cairo XCB surface.
        let raw_conn = conn.get_raw_xcb_connection();
        let xcb_conn = unsafe { cairo::XCBConnection::from_raw_none(raw_conn as *mut _) };
        let xcb_drawable = cairo::XCBDrawable(self.id);
        let mut c_visual = CXcbVisualtype::from_x11rb(&self.visual);
        let xcb_visual = c_visual.as_cairo();

        let surface = cairo::XCBSurface::create(&xcb_conn, &xcb_drawable, &xcb_visual, w, h)
            .expect("XCB surface");

        let cr = cairo::Context::new(&surface).expect("blit context");
        cr.set_source_surface(&image, 0.0, 0.0).expect("source");
        cr.paint().expect("paint");
        drop(cr);
        surface.flush();
        drop(surface);
        conn.flush().expect("Failed to flush");
    }

    pub fn hide(&self, conn: &XCBConnection) {
        conn.unmap_window(self.id).expect("Failed to unmap window");
        conn.flush().expect("Failed to flush");
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }
}

fn intern_atom(conn: &XCBConnection, name: &[u8]) -> u32 {
    conn.intern_atom(false, name)
        .expect("Failed to send intern atom request")
        .reply()
        .expect("Failed to intern atom")
        .atom
}
