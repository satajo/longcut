use crate::window::Window;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::Screen;
use x11rb::xcb_ffi::XCBConnection;

pub struct XcbService {
    connection: XCBConnection,
    screen_num: usize,
}

impl XcbService {
    /// # Panics
    ///
    /// Panics if the connection to the X server cannot be established.
    #[expect(
        clippy::new_without_default,
        reason = "connects to the X server on construction; Default would hide this side effect"
    )]
    #[must_use]
    pub fn new() -> Self {
        let (connection, screen_num) =
            XCBConnection::connect(None).expect("Failed to connect to X server");
        XcbService {
            connection,
            screen_num,
        }
    }

    pub fn screen(&self) -> &Screen {
        &self.connection.setup().roots[self.screen_num]
    }

    pub fn get_screen_dimensions(&self) -> (u32, u32) {
        // Try RandR get_monitors to find the primary monitor.
        if let Ok(cookie) =
            x11rb::protocol::randr::get_monitors(&self.connection, self.screen().root, true)
            && let Ok(reply) = cookie.reply()
        {
            // Find the monitor at origin (0, 0), matching the previous GTK behavior.
            if let Some(monitor) = reply
                .monitors
                .iter()
                .find(|m| m.x == 0 && m.y == 0)
                .or_else(|| reply.monitors.first())
            {
                return (u32::from(monitor.width), u32::from(monitor.height));
            }
        }

        // Fallback: use root window geometry.
        let screen = self.screen();
        (
            u32::from(screen.width_in_pixels),
            u32::from(screen.height_in_pixels),
        )
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "values are clamped with .min() before casting, so truncation cannot occur"
    )]
    pub fn create_window(&self, x: u32, y: u32, width: u32, height: u32) -> Window<'_> {
        Window::new(
            &self.connection,
            self.screen(),
            x.min(i16::MAX as u32) as i16,
            y.min(i16::MAX as u32) as i16,
            width.min(u32::from(u16::MAX)) as u16,
            height.min(u32::from(u16::MAX)) as u16,
        )
    }
}
