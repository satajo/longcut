use crate::keymap::{Keymap, KeymapError, X11KeyPress};
use std::cell::{Cell, Ref, RefCell};
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::errors::{ConnectError, ConnectionError, ReplyError};
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, GrabMode, GrabStatus, Window};
use x11rb::xcb_ffi::XCBConnection;

/// A connection to the X server, its root window, and the server's keymap.
pub struct X11Handle {
    connection: XCBConnection,
    root_window: Window,
    keymap: RefCell<Keymap>,
    /// The sequence number of the request that last released the keyboard grab. Key events
    /// queued before it were typed into that grab.
    ungrab_sequence: Cell<u64>,
}

#[derive(Debug)]
pub enum X11Error {
    /// No connection to the X server could be established.
    Connect(ConnectError),
    /// The server's keymap could not be fetched.
    Keymap(KeymapError),
}

impl std::fmt::Display for X11Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X11Error::Connect(error) => write!(f, "could not connect to the X server: {error}"),
            X11Error::Keymap(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for X11Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            X11Error::Connect(error) => Some(error),
            X11Error::Keymap(error) => Some(error),
        }
    }
}

/// Why the keyboard grab did not happen.
#[derive(Debug)]
pub enum GrabError {
    /// Another client holds the keyboard grab.
    AlreadyGrabbed,
    /// The root window is not viewable.
    NotViewable,
    /// The keyboard is frozen by another client's grab.
    Frozen,
    /// The X server rejected the grab time.
    InvalidTime,
    /// The X server reported a status this crate does not know.
    Unknown(u8),
    /// The request could not be exchanged with the X server.
    Connection(ReplyError),
}

impl std::fmt::Display for GrabError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GrabError::AlreadyGrabbed => write!(f, "another client holds the keyboard grab"),
            GrabError::NotViewable => write!(f, "the root window is not viewable"),
            GrabError::Frozen => write!(f, "the keyboard is frozen by another client's grab"),
            GrabError::InvalidTime => write!(f, "the X server rejected the grab time"),
            GrabError::Unknown(status) => {
                write!(
                    f,
                    "the X server refused the keyboard grab with status {status}"
                )
            }
            GrabError::Connection(error) => {
                write!(f, "could not request the keyboard grab: {error}")
            }
        }
    }
}

impl std::error::Error for GrabError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GrabError::Connection(error) => Some(error),
            _ => None,
        }
    }
}

impl X11Handle {
    /// Connects to the X server named by `DISPLAY` and fetches its keymap.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection fails or the keymap cannot be fetched.
    pub fn connect() -> Result<Self, X11Error> {
        let (connection, screen) = XCBConnection::connect(None).map_err(X11Error::Connect)?;
        let root_window = connection.setup().roots[screen].root;
        let keymap = Keymap::from_server(&connection).map_err(X11Error::Keymap)?;
        Ok(Self {
            connection,
            root_window,
            keymap: RefCell::new(keymap),
            ungrab_sequence: Cell::new(0),
        })
    }

    pub(crate) fn connection(&self) -> &XCBConnection {
        &self.connection
    }

    pub(crate) fn root_window(&self) -> Window {
        self.root_window
    }

    pub(crate) fn keymap(&self) -> Ref<'_, Keymap> {
        self.keymap.borrow()
    }

    pub(crate) fn ungrab_sequence(&self) -> u64 {
        self.ungrab_sequence.get()
    }

    /// Fetches the keymap from the server again, for instance after a `setxkbmap`. Presses from
    /// here on resolve against the layout every other program now sees. A keymap that cannot be
    /// fetched leaves the previous one in place.
    pub(crate) fn refresh_keymap(&self) {
        match Keymap::from_server(&self.connection) {
            Ok(keymap) => *self.keymap.borrow_mut() = keymap,
            Err(error) => eprintln!("keeping the previous keymap: {error}"),
        }
    }

    /// Takes the keyboard grab on the root window. The X server holds it until it is released or
    /// the connection closes.
    ///
    /// # Errors
    ///
    /// Returns an error when the X server refuses the grab or cannot be reached.
    pub fn grab_keyboard(&self) -> Result<(), GrabError> {
        let reply = self
            .connection
            .grab_keyboard(
                true,
                self.root_window,
                CURRENT_TIME,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )
            .map_err(ReplyError::from)
            .and_then(x11rb::cookie::Cookie::reply)
            .map_err(GrabError::Connection)?;
        match reply.status {
            GrabStatus::SUCCESS => Ok(()),
            GrabStatus::ALREADY_GRABBED => Err(GrabError::AlreadyGrabbed),
            GrabStatus::NOT_VIEWABLE => Err(GrabError::NotViewable),
            GrabStatus::FROZEN => Err(GrabError::Frozen),
            GrabStatus::INVALID_TIME => Err(GrabError::InvalidTime),
            status => Err(GrabError::Unknown(u8::from(status))),
        }
    }

    /// Releases the keyboard grab.
    ///
    /// # Errors
    ///
    /// Returns an error when the X server cannot be reached.
    pub fn ungrab_keyboard(&self) -> Result<(), ConnectionError> {
        let cookie = self.connection.ungrab_keyboard(CURRENT_TIME)?;
        self.ungrab_sequence.set(cookie.sequence_number());
        self.connection.flush()
    }

    /// Blocks until the next key press and resolves it against the server's keymap.
    ///
    /// # Errors
    ///
    /// Returns an error when the connection to the X server is lost.
    pub fn next_key_press(&self) -> Result<X11KeyPress, ConnectionError> {
        loop {
            match self.connection.wait_for_event()? {
                Event::KeyPress(event) => {
                    return Ok(self
                        .keymap
                        .borrow_mut()
                        .resolve(event.detail, u16::from(event.state)));
                }
                Event::MappingNotify(_) => self.refresh_keymap(),
                _ => {}
            }
        }
    }

    /// Returns the currently focused window via `_NET_ACTIVE_WINDOW`, or `None` if the property is
    /// unavailable (e.g. no EWMH-compliant window manager is running).
    #[must_use]
    pub fn get_active_window(&self) -> Option<Window> {
        let atom = self
            .connection
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")
            .ok()?
            .reply()
            .ok()?
            .atom;
        let reply = self
            .connection
            .get_property(false, self.root_window, atom, AtomEnum::WINDOW, 0, 1)
            .ok()?
            .reply()
            .ok()?;
        reply.value32()?.next().filter(|&window| window != 0)
    }

    /// Returns the `WM_CLASS` property of the window as `(instance_name, class_name)`, or `None`
    /// if the property is absent. The two values are the null-separated parts of the raw property.
    #[must_use]
    pub fn get_window_class(&self, window: Window) -> Option<(String, String)> {
        let reply = self
            .connection
            .get_property(false, window, AtomEnum::WM_CLASS, AtomEnum::STRING, 0, 1024)
            .ok()?
            .reply()
            .ok()?;
        let mut parts = reply.value.split(|&byte| byte == 0);
        let instance = String::from_utf8(parts.next()?.to_vec()).ok()?;
        let class = String::from_utf8(parts.next()?.to_vec()).ok()?;
        Some((instance, class))
    }
}
