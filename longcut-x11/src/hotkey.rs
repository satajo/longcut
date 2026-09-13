use crate::handle::{GrabError, X11Handle};
use crate::keymap::ActiveModifier;
use std::cell::RefCell;
use x11rb::connection::Connection;
use x11rb::errors::{ConnectionError, ReplyError};
use x11rb::protocol::xproto::{ConnectionExt, Grab, GrabMode, ModMask};
use x11rb::protocol::{ErrorKind, Event};
use xkbcommon::xkb::Keysym;

/// A key the X server watches for on the whole keyboard, named the way the user names it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Hotkey {
    pub keysym: Keysym,
    pub modifiers: Vec<ActiveModifier>,
}

/// One passive grab: a keycode with the exact modifiers that must be held with it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct KeyGrab {
    pub keycode: u8,
    pub modifiers: u16,
}

/// Why the hotkeys could not be bound or waited for. The variants about one hotkey carry its
/// index in the bound list.
#[derive(Debug)]
pub enum HotkeyError {
    /// No key in the current keyboard layout produces the hotkey's keysym.
    Unmapped { hotkey: usize },
    /// Another client already holds a passive grab on the hotkey.
    AlreadyBound { hotkey: usize },
    /// The keyboard could not be taken for the session a hotkey began.
    Grab(GrabError),
    /// The X server refused a request or could not be reached.
    Connection(ReplyError),
}

impl HotkeyError {
    /// The index of the hotkey the error is about, when it is about one.
    #[must_use]
    pub fn hotkey(&self) -> Option<usize> {
        match self {
            HotkeyError::Unmapped { hotkey } | HotkeyError::AlreadyBound { hotkey } => {
                Some(*hotkey)
            }
            HotkeyError::Grab(_) | HotkeyError::Connection(_) => None,
        }
    }
}

impl std::fmt::Display for HotkeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HotkeyError::Unmapped { .. } => {
                write!(f, "no key in the current keyboard layout produces it")
            }
            HotkeyError::AlreadyBound { .. } => write!(
                f,
                "another client already binds it; does the window manager still bind it?"
            ),
            HotkeyError::Grab(error) => write!(f, "{error}"),
            HotkeyError::Connection(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for HotkeyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HotkeyError::Grab(error) => Some(error),
            HotkeyError::Connection(error) => Some(error),
            HotkeyError::Unmapped { .. } | HotkeyError::AlreadyBound { .. } => None,
        }
    }
}

impl From<ConnectionError> for HotkeyError {
    fn from(error: ConnectionError) -> Self {
        HotkeyError::Connection(ReplyError::ConnectionError(error))
    }
}

/// A set of hotkeys bound on the X server through passive grabs on the root window.
///
/// The X server delivers a bound hotkey's press to this connection no matter which window has
/// the focus, and from that press on it treats the keyboard as grabbed by this connection, so
/// no key pressed after the hotkey can reach any other client. The grabs are rebound whenever
/// the server's keymap changes, since the keycodes producing a keysym change with it.
pub struct Hotkeys<'a> {
    x11: &'a X11Handle,
    bound: Vec<Hotkey>,
    /// Every passive grab, with the index of the hotkey it belongs to.
    grabs: RefCell<Vec<(KeyGrab, usize)>>,
}

impl<'a> Hotkeys<'a> {
    /// Binds the hotkeys.
    ///
    /// # Errors
    ///
    /// Returns an error if a hotkey cannot be bound.
    pub fn bind(x11: &'a X11Handle, hotkeys: Vec<Hotkey>) -> Result<Self, HotkeyError> {
        let bound = Self {
            x11,
            bound: hotkeys,
            grabs: RefCell::new(Vec::new()),
        };
        bound.rebind()?;
        Ok(bound)
    }

    /// Blocks until a hotkey is pressed, takes the keyboard for the session it begins, and
    /// returns the hotkey's index.
    ///
    /// # Errors
    ///
    /// Returns an error if the hotkeys cannot be rebound after a keymap change, the keyboard
    /// cannot be taken, or the X server cannot be reached.
    pub fn wait_for_press(&self) -> Result<usize, HotkeyError> {
        if self.x11.take_keymap_changed() {
            self.rebind()?;
        }
        loop {
            let (event, sequence) = self.x11.connection().wait_for_event_with_sequence()?;
            match event {
                // A press queued while the previous session still held the keyboard was typed
                // into that session, not at a hotkey.
                Event::KeyPress(event) if sequence >= self.x11.session_end_sequence() => {
                    let state = u16::from(event.state) & 0xff & !self.x11.keymap().lock_bits();
                    let pressed = self
                        .grabs
                        .borrow()
                        .iter()
                        .find(|(grab, _)| grab.keycode == event.detail && grab.modifiers == state)
                        .map(|(_, hotkey)| *hotkey);
                    if let Some(hotkey) = pressed {
                        self.x11
                            .begin_session(event.detail)
                            .map_err(HotkeyError::Grab)?;
                        return Ok(hotkey);
                    }
                }
                Event::MappingNotify(_) => {
                    self.x11.refresh_keymap();
                    self.rebind()?;
                }
                _ => {}
            }
        }
    }

    /// Replaces every passive grab with the grabs the current keymap calls for.
    fn rebind(&self) -> Result<(), HotkeyError> {
        let connection = self.x11.connection();
        let root = self.x11.root_window();
        let keymap = self.x11.keymap();
        let lock_bits = keymap.lock_bits();

        connection
            .ungrab_key(Grab::ANY, root, ModMask::ANY)
            .map_err(ReplyError::from)
            .and_then(x11rb::cookie::VoidCookie::check)
            .map_err(HotkeyError::Connection)?;

        let mut grabs = Vec::new();
        for (index, hotkey) in self.bound.iter().enumerate() {
            let key_grabs = keymap.key_grabs(hotkey);
            if key_grabs.is_empty() {
                return Err(HotkeyError::Unmapped { hotkey: index });
            }
            for grab in key_grabs {
                // The user leaves Caps Lock and Num Lock on for long stretches, so the hotkey
                // is grabbed under every combination of them.
                for locks in lock_variants(lock_bits) {
                    let modifiers = ModMask::from(grab.modifiers | locks);
                    connection
                        .grab_key(
                            false,
                            root,
                            modifiers,
                            grab.keycode,
                            GrabMode::ASYNC,
                            GrabMode::ASYNC,
                        )
                        .map_err(ReplyError::from)
                        .and_then(x11rb::cookie::VoidCookie::check)
                        .map_err(|error| to_bind_error(error, index))?;
                }
                grabs.push((grab, index));
            }
        }
        *self.grabs.borrow_mut() = grabs;
        Ok(())
    }
}

/// Every subset of the lock bits, as the modifier combinations a hotkey is grabbed under.
fn lock_variants(lock_bits: u16) -> impl Iterator<Item = u16> {
    // Iterating over all submasks of a mask: the next submask below `current` is
    // (current - 1) & mask, and 0 is the last one.
    let mut next = Some(lock_bits);
    std::iter::from_fn(move || {
        let current = next?;
        next = (current != 0).then(|| (current - 1) & lock_bits);
        Some(current)
    })
}

fn to_bind_error(error: ReplyError, hotkey: usize) -> HotkeyError {
    match error {
        ReplyError::X11Error(x11_error) if x11_error.error_kind == ErrorKind::Access => {
            HotkeyError::AlreadyBound { hotkey }
        }
        error => HotkeyError::Connection(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_variants_cover_every_combination_of_the_lock_bits() {
        let mut variants: Vec<u16> = lock_variants(0b10010).collect();
        variants.sort_unstable();
        assert_eq!(variants, vec![0b00000, 0b00010, 0b10000, 0b10010]);
    }

    #[test]
    fn no_lock_bits_means_one_plain_variant() {
        assert_eq!(lock_variants(0).collect::<Vec<u16>>(), vec![0]);
    }
}
