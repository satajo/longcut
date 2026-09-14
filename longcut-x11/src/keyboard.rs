use crate::chord::LaunchChord;
use crate::handle::X11Handle;
use crate::keymap::X11KeyPress;
use std::cell::RefCell;
use x11rb::CURRENT_TIME;
use x11rb::connection::Connection;
use x11rb::errors::{ConnectionError, ReplyError};
use x11rb::protocol::Event;
use x11rb::protocol::xproto::{ConnectionExt, GrabMode, GrabStatus};

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

/// The keyboard, held by this connection alone. From the grab until the value is dropped every
/// key event reaches this connection and no other client. The X server releases the grab itself
/// when the connection closes, so a crash cannot keep the keyboard.
pub struct KeyboardGrab<'a> {
    x11: &'a X11Handle,
    chord: RefCell<LaunchChord>,
}

impl<'a> KeyboardGrab<'a> {
    /// Takes the keyboard grab on the root window. The keys held at that moment are the chord
    /// that took it, and they are left out of the presses that follow until released.
    ///
    /// # Errors
    ///
    /// Returns an error if the X server refuses the grab or cannot be reached.
    pub fn take(x11: &'a X11Handle) -> Result<Self, GrabError> {
        let connection = x11.connection();
        let grab = connection
            .grab_keyboard(
                true,
                x11.root_window(),
                CURRENT_TIME,
                GrabMode::ASYNC,
                GrabMode::ASYNC,
            )
            .map_err(|error| GrabError::Connection(error.into()))?;
        // Requested right behind the grab, so that the server takes the snapshot as soon as the
        // grab is in place.
        let snapshot = connection
            .query_keymap()
            .map_err(|error| GrabError::Connection(error.into()))?;
        match grab.reply().map_err(GrabError::Connection)?.status {
            GrabStatus::SUCCESS => {}
            GrabStatus::ALREADY_GRABBED => return Err(GrabError::AlreadyGrabbed),
            GrabStatus::NOT_VIEWABLE => return Err(GrabError::NotViewable),
            GrabStatus::FROZEN => return Err(GrabError::Frozen),
            GrabStatus::INVALID_TIME => return Err(GrabError::InvalidTime),
            status => return Err(GrabError::Unknown(u8::from(status))),
        }
        let snapshot_sequence = snapshot.sequence_number();
        let held = snapshot.reply().map_err(GrabError::Connection)?.keys;
        Ok(Self {
            x11,
            chord: RefCell::new(LaunchChord::new(held, snapshot_sequence)),
        })
    }

    /// Blocks until the next key press and resolves it against the server's keymap, with the
    /// chord that took the keyboard left out of the press.
    ///
    /// # Errors
    ///
    /// Returns an error when the connection to the X server is lost.
    pub fn next_key_press(&self) -> Result<X11KeyPress, ConnectionError> {
        loop {
            let (event, sequence) = self.x11.connection().wait_for_event_with_sequence()?;
            match event {
                Event::KeyPress(event) => {
                    let state = u16::from(event.state);
                    let Some(state) = self.chord.borrow_mut().press(event.detail, state, sequence)
                    else {
                        continue;
                    };
                    return Ok(self.x11.keymap_mut().resolve(event.detail, state));
                }
                Event::KeyRelease(event) => {
                    self.chord
                        .borrow_mut()
                        .release(event.detail, u16::from(event.state));
                }
                Event::MappingNotify(_) => self.x11.refresh_keymap(),
                _ => {}
            }
        }
    }
}

impl Drop for KeyboardGrab<'_> {
    /// Releases the keyboard. A failure means the connection is gone, which the next request
    /// on it reports; the X server has released the grab by then.
    fn drop(&mut self) {
        let connection = self.x11.connection();
        let released = connection.ungrab_keyboard(CURRENT_TIME).and_then(|cookie| {
            self.x11
                .set_keyboard_release_sequence(cookie.sequence_number());
            connection.flush()
        });
        if let Err(error) = released {
            eprintln!("could not release the keyboard: {error}");
        }
    }
}
