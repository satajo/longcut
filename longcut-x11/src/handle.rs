use crate::keymap::{Keymap, KeymapError};
use std::cell::{Cell, Ref, RefCell, RefMut};
use x11rb::connection::Connection;
use x11rb::errors::{ConnectError, ReplyError};
use x11rb::protocol::xkb;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};
use x11rb::xcb_ffi::XCBConnection;

/// A connection to the X server, its root window, and the server's keymap.
pub struct X11Handle {
    connection: XCBConnection,
    root_window: Window,
    keymap: RefCell<Keymap>,
    /// Set when the keymap has been refreshed since the hotkeys were last bound.
    keymap_changed: Cell<bool>,
    /// The sequence number of the request that last released the keyboard. Key events queued
    /// before it were typed at the keyboard while it was held.
    keyboard_release_sequence: Cell<u64>,
}

#[derive(Debug)]
pub enum X11Error {
    /// No connection to the X server could be established.
    Connect(ConnectError),
    /// The server's keymap could not be fetched.
    Keymap(KeymapError),
    /// The server refused to report autorepeat as repeated presses.
    DetectableAutoRepeat(ReplyError),
}

impl std::fmt::Display for X11Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X11Error::Connect(error) => write!(f, "could not connect to the X server: {error}"),
            X11Error::Keymap(error) => write!(f, "{error}"),
            X11Error::DetectableAutoRepeat(error) => {
                write!(f, "could not enable detectable autorepeat: {error}")
            }
        }
    }
}

impl std::error::Error for X11Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            X11Error::Connect(error) => Some(error),
            X11Error::Keymap(error) => Some(error),
            X11Error::DetectableAutoRepeat(error) => Some(error),
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
        enable_detectable_autorepeat(&connection).map_err(X11Error::DetectableAutoRepeat)?;
        Ok(Self {
            connection,
            root_window,
            keymap: RefCell::new(keymap),
            keymap_changed: Cell::new(false),
            keyboard_release_sequence: Cell::new(0),
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

    pub(crate) fn keymap_mut(&self) -> RefMut<'_, Keymap> {
        self.keymap.borrow_mut()
    }

    /// Whether the keymap has been refreshed since this was last asked.
    pub(crate) fn take_keymap_changed(&self) -> bool {
        self.keymap_changed.replace(false)
    }

    pub(crate) fn keyboard_release_sequence(&self) -> u64 {
        self.keyboard_release_sequence.get()
    }

    pub(crate) fn set_keyboard_release_sequence(&self, sequence: u64) {
        self.keyboard_release_sequence.set(sequence);
    }

    /// Fetches the keymap from the server again, for instance after a `setxkbmap`. Presses from
    /// here on resolve against the layout every other program now sees. A keymap that cannot be
    /// fetched leaves the previous one in place.
    pub(crate) fn refresh_keymap(&self) {
        match Keymap::from_server(&self.connection) {
            Ok(keymap) => {
                *self.keymap.borrow_mut() = keymap;
                self.keymap_changed.set(true);
            }
            Err(error) => eprintln!("keeping the previous keymap: {error}"),
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

/// Asks the server to report a held key's autorepeat as repeated presses without releases in
/// between, so that a key held since before the keyboard was taken can be told from one pressed
/// again.
fn enable_detectable_autorepeat(connection: &XCBConnection) -> Result<(), ReplyError> {
    let flag = xkb::PerClientFlag::DETECTABLE_AUTO_REPEAT;
    let no_controls = xkb::BoolCtrl::default();
    xkb::per_client_flags(
        connection,
        u16::from(xkb::ID::USE_CORE_KBD),
        flag,
        flag,
        no_controls,
        no_controls,
        no_controls,
    )?
    .reply()?;
    Ok(())
}
