use longcut_core::model::key::{Key, Modifier, Symbol};
use longcut_core::port::input::{Input, InputError, Keyboard};
use longcut_x11::{ActiveModifier, KeyboardGrab, X11Handle};

/// Adapts the X11 keyboard into the core [`Input`] port. Taking the keyboard is the X keyboard
/// grab, which lasts until the [`Keyboard`] is dropped. The X server releases every grab when the
/// connection closes, which happens on every form of process death, so a crash can never leave
/// the keyboard grabbed.
pub struct X11Input<'a> {
    x11: &'a X11Handle,
}

impl<'a> X11Input<'a> {
    #[must_use]
    pub fn new(x11: &'a X11Handle) -> Self {
        Self { x11 }
    }
}

impl Input for X11Input<'_> {
    fn take_keyboard(&self) -> Result<Box<dyn Keyboard + '_>, InputError> {
        match KeyboardGrab::take(self.x11) {
            Ok(grab) => Ok(Box::new(X11Keyboard { grab })),
            Err(error) => Err(InputError {
                reason: error.to_string(),
            }),
        }
    }
}

/// The grabbed X11 keyboard.
struct X11Keyboard<'a> {
    grab: KeyboardGrab<'a>,
}

impl Keyboard for X11Keyboard<'_> {
    /// Block until the next key press that stands for a symbol.
    fn capture_any(&self) -> Key {
        loop {
            let press = match self.grab.next_key_press() {
                Ok(press) => press,
                // A session cannot continue without input. Process death closes the connection,
                // which releases the grab.
                Err(error) => panic!("keyboard input is permanently unavailable: {error}"),
            };

            // A keycode the keymap leaves unbound stands for no symbol and is not a key.
            if let Some(symbol) = Symbol::from_keysym(press.keysym) {
                return to_core_key(symbol, &press.modifiers);
            }
        }
    }
}

fn to_core_key(symbol: Symbol, modifiers: &[ActiveModifier]) -> Key {
    let mut key = Key::new(symbol);
    for active in modifiers {
        key.add_modifier(to_core_modifier(*active));
    }
    key
}

fn to_core_modifier(modifier: ActiveModifier) -> Modifier {
    match modifier {
        ActiveModifier::Shift => Modifier::Shift,
        ActiveModifier::Control => Modifier::Control,
        ActiveModifier::Alt => Modifier::Alt,
        ActiveModifier::Logo => Modifier::Super,
    }
}

pub(crate) fn to_active_modifier(modifier: Modifier) -> ActiveModifier {
    match modifier {
        Modifier::Shift => ActiveModifier::Shift,
        Modifier::Control => ActiveModifier::Control,
        Modifier::Alt => ActiveModifier::Alt,
        Modifier::Super => ActiveModifier::Logo,
    }
}
