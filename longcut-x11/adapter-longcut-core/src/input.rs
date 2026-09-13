use longcut_core::model::key::{Key, Modifier, Symbol};
use longcut_core::port::input::Input;
use longcut_x11::{ActiveModifier, Hotkey, HotkeyError, Hotkeys, X11Handle};

/// Adapts the X11 keyboard into the core [`Input`] port.
///
/// The keys of a `capture_one` are bound on the X server through passive grabs, so that a press
/// reaches this process no matter which window has the focus. A `capture_any_iter` grabs the
/// whole keyboard for as long as it is read. The X server releases every grab when the connection
/// closes, which happens on every form of process death, so a crash can never leave the keyboard
/// grabbed.
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
    fn capture_one(&self, keys: &[Key]) -> Key {
        let hotkeys = keys.iter().map(to_hotkey).collect();
        let hotkeys = match Hotkeys::bind(self.x11, hotkeys) {
            Ok(hotkeys) => hotkeys,
            Err(error) => panic!("keys cannot be bound: {}", describe(keys, &error)),
        };
        match hotkeys.wait_for_press() {
            Ok(index) => keys[index].clone(),
            // Input cannot continue without the X server. Process death closes the connection,
            // which releases every grab.
            Err(error) => panic!(
                "keyboard input is permanently unavailable: {}",
                describe(keys, &error)
            ),
        }
    }

    fn capture_any_iter(&self) -> Box<dyn Iterator<Item = Key> + '_> {
        Box::new(KeysIter::new(self.x11))
    }
}

/// Names the key an error is about, when it is about one.
fn describe(keys: &[Key], error: &HotkeyError) -> String {
    match error.hotkey() {
        Some(index) => format!("key {}: {error}", keys[index]),
        None => error.to_string(),
    }
}

/// Reads presses from the grabbed keyboard. The grab is held for as long as the iterator lives.
struct KeysIter<'a> {
    x11: &'a X11Handle,
}

impl<'a> KeysIter<'a> {
    fn new(x11: &'a X11Handle) -> Self {
        if let Err(error) = x11.grab_keyboard() {
            panic!("the keyboard cannot be grabbed: {error}");
        }
        Self { x11 }
    }
}

impl Iterator for KeysIter<'_> {
    type Item = Key;

    /// Block until the next key press that stands for a symbol.
    fn next(&mut self) -> Option<Key> {
        loop {
            let press = match self.x11.next_key_press() {
                Ok(press) => press,
                Err(error) => panic!("keyboard input is permanently unavailable: {error}"),
            };

            // A keycode the keymap leaves unbound stands for no symbol and is not a key.
            if let Some(symbol) = Symbol::from_keysym(press.keysym) {
                return Some(to_core_key(symbol, &press.modifiers));
            }
        }
    }
}

impl Drop for KeysIter<'_> {
    fn drop(&mut self) {
        // A failure here means the connection is gone, which the next read reports.
        if let Err(error) = self.x11.ungrab_keyboard() {
            eprintln!("could not release the keyboard: {error}");
        }
    }
}

fn to_hotkey(key: &Key) -> Hotkey {
    Hotkey {
        keysym: key.symbol.keysym(),
        modifiers: key
            .modifiers
            .iter()
            .copied()
            .map(to_active_modifier)
            .collect(),
    }
}

fn to_active_modifier(modifier: Modifier) -> ActiveModifier {
    match modifier {
        Modifier::Shift => ActiveModifier::Shift,
        Modifier::Control => ActiveModifier::Control,
        Modifier::Alt => ActiveModifier::Alt,
        Modifier::Super => ActiveModifier::Logo,
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
