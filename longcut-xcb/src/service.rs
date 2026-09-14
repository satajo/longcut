use crate::window::{Window, WindowGeometry};
use x11rb::connection::Connection;
use x11rb::errors::ConnectError;
use x11rb::protocol::xproto::Screen;
use x11rb::xcb_ffi::XCBConnection;

#[derive(Debug)]
pub struct XcbService {
    connection: XCBConnection,
    /// The screen the display name selects.
    screen: Screen,
}

#[derive(Debug)]
pub enum XcbError {
    /// No connection to the X server could be established.
    Connect(ConnectError),
    /// The display name selects a screen the X server does not have.
    NoSuchScreen(usize),
}

impl std::fmt::Display for XcbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XcbError::Connect(error) => write!(f, "could not connect to the X server: {error}"),
            XcbError::NoSuchScreen(screen) => write!(f, "the X server has no screen {screen}"),
        }
    }
}

impl std::error::Error for XcbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            XcbError::Connect(error) => Some(error),
            XcbError::NoSuchScreen(_) => None,
        }
    }
}

impl XcbService {
    /// Connects to the X server named by `DISPLAY`.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails or the display names a screen the server does
    /// not have.
    pub fn new() -> Result<Self, XcbError> {
        let (connection, screen_num) = XCBConnection::connect(None).map_err(XcbError::Connect)?;
        let screen = connection
            .setup()
            .roots
            .get(screen_num)
            .ok_or(XcbError::NoSuchScreen(screen_num))?
            .clone();
        Ok(XcbService { connection, screen })
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
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

    pub fn create_window(&self, geometry: WindowGeometry) -> Window<'_> {
        Window::new(&self.connection, self.screen(), geometry)
    }
}
