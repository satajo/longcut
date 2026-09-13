use longcut_core::model::key::{Key, Modifier, Symbol};
use longcut_core::port::input::Input;
use longcut_x11::{ActiveModifier, X11Handle};

/// Adapts the grabbed X11 keyboard into the core [`Input`] port.
///
/// This only reads. The keyboard is grabbed by the [`X11Launcher`](crate::X11Launcher) when a
/// launch key begins the session, and released when the session ends. The X server releases every
/// grab when the connection closes, which happens on every form of process death, so a crash can
/// never leave the keyboard grabbed.
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
    /// Block until the next key press that stands for a symbol.
    fn capture_any(&self) -> Key {
        loop {
            let press = match self.x11.next_key_press() {
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
