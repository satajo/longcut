use crate::hotkey::{Hotkey, KeyGrab};
use x11rb::xcb_ffi::XCBConnection;
use xkbcommon::xkb;
use xkbcommon::xkb::x11 as xkb_x11;

/// A modifier named in xkb terms: one held during a key press, or one a hotkey requires.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ActiveModifier {
    Shift,
    Control,
    Alt,
    Logo,
}

/// A single key press, resolved against the X server's keymap.
///
/// `keysym` is resolved under the modifiers the X server reports held at the moment of the press,
/// so it carries Shift and Caps Lock (`a` becomes `A`) but not Control, which xkb applies only to
/// the produced text. `modifiers` are the held modifiers.
#[derive(Clone, Debug)]
pub struct X11KeyPress {
    pub keysym: xkb::Keysym,
    pub modifiers: Vec<ActiveModifier>,
}

/// The X server's keymap, with a modifier state fed from each key event.
pub struct Keymap {
    description: xkb::Keymap,
    state: xkb::State,
    /// The modifiers a press can report, resolved to their indices in the keymap.
    modifiers: [(xkb::ModIndex, ActiveModifier); 4],
    /// The Lock modifier's bit in a key event's state.
    lock_mask: xkb::ModMask,
    /// The Num Lock modifier's bit in a key event's state.
    num_lock_mask: xkb::ModMask,
}

#[derive(Debug)]
pub enum KeymapError {
    /// The X server does not provide the XKB extension version libxkbcommon needs.
    XkbExtensionUnavailable,
    /// The X server has no core keyboard device.
    NoCoreKeyboard,
    /// libxkbcommon could not build a keymap from the X server's keyboard description.
    KeymapConstruction,
}

impl std::fmt::Display for KeymapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeymapError::XkbExtensionUnavailable => {
                write!(f, "the X server does not provide the XKB extension")
            }
            KeymapError::NoCoreKeyboard => write!(f, "the X server has no core keyboard device"),
            KeymapError::KeymapConstruction => {
                write!(
                    f,
                    "could not build a keymap from the X server's keyboard description"
                )
            }
        }
    }
}

impl std::error::Error for KeymapError {}

/// How many modifier combinations one level of a key can be reached with; a key type has at most
/// a handful of entries.
const MAX_MASKS_PER_LEVEL: usize = 16;

/// A keymap modifier mask as the X server reports it in a key event's state. The keymap comes
/// from the X server, so its real modifiers are the eight X modifiers in protocol order and the
/// masks it produces fit the state's low byte.
fn to_state_bits(mask: xkb::ModMask) -> u16 {
    u16::try_from(mask).expect("real modifier masks of an X keymap fit in 8 bits")
}

impl Keymap {
    /// Fetches the core keyboard's keymap from the X server.
    ///
    /// # Errors
    ///
    /// Returns an error if the X server lacks the XKB extension or a core keyboard, or if the
    /// keymap cannot be built from its description.
    pub fn from_server(connection: &XCBConnection) -> Result<Self, KeymapError> {
        let (mut major, mut minor, mut base_event, mut base_error) = (0, 0, 0, 0);
        let xkb_available = xkb_x11::setup_xkb_extension(
            connection,
            xkb_x11::MIN_MAJOR_XKB_VERSION,
            xkb_x11::MIN_MINOR_XKB_VERSION,
            xkb_x11::SetupXkbExtensionFlags::NoFlags,
            &mut major,
            &mut minor,
            &mut base_event,
            &mut base_error,
        );
        if !xkb_available {
            return Err(KeymapError::XkbExtensionUnavailable);
        }

        let device = xkb_x11::get_core_keyboard_device_id(connection);
        if device == -1 {
            return Err(KeymapError::NoCoreKeyboard);
        }

        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb_x11::keymap_new_from_device(
            &context,
            connection,
            device,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );
        if keymap.get_raw_ptr().is_null() {
            return Err(KeymapError::KeymapConstruction);
        }
        Ok(Self::new(&keymap))
    }

    fn new(keymap: &xkb::Keymap) -> Self {
        let modifiers = [
            (xkb::MOD_NAME_SHIFT, ActiveModifier::Shift),
            (xkb::MOD_NAME_CTRL, ActiveModifier::Control),
            (xkb::MOD_NAME_ALT, ActiveModifier::Alt),
            (xkb::MOD_NAME_LOGO, ActiveModifier::Logo),
        ]
        .map(|(name, modifier)| (keymap.mod_get_index(name), modifier));
        Self {
            description: keymap.clone(),
            state: xkb::State::new(keymap),
            modifiers,
            lock_mask: 1 << keymap.mod_get_index(xkb::MOD_NAME_CAPS),
            num_lock_mask: 1 << keymap.mod_get_index(xkb::MOD_NAME_NUM),
        }
    }

    /// The bits of a key event's state that a hotkey ignores: Caps Lock and Num Lock, which the
    /// user leaves on for long stretches without meaning them as part of a hotkey.
    #[must_use]
    pub fn lock_bits(&self) -> u16 {
        to_state_bits(self.lock_mask | self.num_lock_mask)
    }

    /// The passive grabs that catch every way of pressing the hotkey: one per key and level
    /// producing its keysym, with the modifiers that level needs on top of the hotkey's own.
    /// An empty result means no key in the keymap produces the keysym.
    #[must_use]
    pub fn key_grabs(&self, hotkey: &Hotkey) -> Vec<KeyGrab> {
        let required: xkb::ModMask = hotkey
            .modifiers
            .iter()
            .map(|&modifier| self.real_mask(modifier))
            .fold(0, |mask, bit| mask | bit);

        let mut grabs = Vec::new();
        self.description.key_for_each(|keymap, keycode| {
            let Ok(x_keycode) = u8::try_from(keycode.raw()) else {
                return;
            };
            for layout in 0..keymap.num_layouts_for_key(keycode) {
                for level in 0..keymap.num_levels_for_key(keycode, layout) {
                    if !keymap
                        .key_get_syms_by_level(keycode, layout, level)
                        .contains(&hotkey.keysym)
                    {
                        continue;
                    }
                    let mut level_masks = [0; MAX_MASKS_PER_LEVEL];
                    let count =
                        keymap.key_get_mods_for_level(keycode, layout, level, &mut level_masks);
                    for level_mask in &level_masks[..count] {
                        grabs.push(KeyGrab {
                            keycode: x_keycode,
                            modifiers: to_state_bits(level_mask | required),
                        });
                    }
                }
            }
        });
        grabs.sort_unstable();
        grabs.dedup();
        grabs
    }

    fn real_mask(&self, modifier: ActiveModifier) -> xkb::ModMask {
        let (index, _) = self
            .modifiers
            .iter()
            .find(|(_, candidate)| *candidate == modifier)
            .expect("every ActiveModifier has an index");
        1 << index
    }

    /// Resolves a key press from its X keycode and the state the X server reports with the event.
    ///
    /// The low byte of the state holds the effective modifiers and bits 13 and 14 the keyboard
    /// group, which is what the keymap needs to pick the key's level.
    pub fn resolve(&mut self, keycode: u8, state: u16) -> X11KeyPress {
        let modifiers = xkb::ModMask::from(state & 0xff);
        let group = xkb::LayoutIndex::from((state >> 13) & 0x3);
        self.state.update_mask(modifiers, 0, 0, 0, 0, group);

        let keycode = xkb::Keycode::from(u32::from(keycode));
        let keysym = self.state.key_get_one_sym(keycode);
        let modifiers = self
            .modifiers
            .iter()
            .filter(|(index, _)| {
                self.state
                    .mod_index_is_active(*index, xkb::STATE_MODS_EFFECTIVE)
            })
            .map(|(_, modifier)| *modifier)
            .collect();
        X11KeyPress { keysym, modifiers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xkb::Keysym;

    /// X keycodes of the keys the tests press: the Linux keycode plus the X11 minimum of 8.
    const KEY_1: u8 = 10;
    const KEY_A: u8 = 38;
    const KEY_KP1: u8 = 87;
    const KEY_ESCAPE: u8 = 9;

    /// Modifier bits as the X server reports them in a key event's state.
    const SHIFT: u16 = 1 << 0;
    const LOCK: u16 = 1 << 1;
    const CONTROL: u16 = 1 << 2;
    const NUM_LOCK: u16 = 1 << 4;
    const LOGO: u16 = 1 << 6;

    fn us_keymap() -> Keymap {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let keymap = xkb::Keymap::new_from_names(
            &context,
            "",
            "",
            "us",
            "",
            None,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .expect("the us layout compiles");
        Keymap::new(&keymap)
    }

    #[test]
    fn a_plain_key_is_its_keysym_without_modifiers() {
        let press = us_keymap().resolve(KEY_A, 0);
        assert_eq!(press.keysym, Keysym::a);
        assert!(press.modifiers.is_empty());
    }

    #[test]
    fn shift_selects_the_shifted_level_and_is_reported() {
        let mut keymap = us_keymap();
        let press = keymap.resolve(KEY_1, SHIFT);
        assert_eq!(press.keysym, Keysym::exclam);
        assert_eq!(press.modifiers, vec![ActiveModifier::Shift]);

        let press = keymap.resolve(KEY_A, SHIFT);
        assert_eq!(press.keysym, Keysym::A);
        assert_eq!(press.modifiers, vec![ActiveModifier::Shift]);
    }

    #[test]
    fn control_leaves_the_keysym_alone_and_is_reported() {
        let press = us_keymap().resolve(KEY_A, CONTROL);
        assert_eq!(press.keysym, Keysym::a);
        assert_eq!(press.modifiers, vec![ActiveModifier::Control]);
    }

    #[test]
    fn num_lock_selects_the_keypad_digits() {
        let mut keymap = us_keymap();
        assert_eq!(keymap.resolve(KEY_KP1, 0).keysym, Keysym::KP_End);
        assert_eq!(keymap.resolve(KEY_KP1, NUM_LOCK).keysym, Keysym::KP_1);
    }

    fn hotkey(keysym: Keysym, modifiers: &[ActiveModifier]) -> Hotkey {
        Hotkey {
            keysym,
            modifiers: modifiers.to_vec(),
        }
    }

    fn grab(keycode: u8, modifiers: u16) -> KeyGrab {
        KeyGrab { keycode, modifiers }
    }

    #[test]
    fn a_hotkey_is_grabbed_on_the_key_producing_its_keysym() {
        let keymap = us_keymap();
        assert_eq!(
            keymap.key_grabs(&hotkey(Keysym::a, &[])),
            vec![grab(KEY_A, 0)]
        );
        assert_eq!(
            keymap.key_grabs(&hotkey(Keysym::Escape, &[])),
            vec![grab(KEY_ESCAPE, 0)]
        );
    }

    #[test]
    fn a_hotkey_requires_its_own_modifiers() {
        let keymap = us_keymap();
        assert_eq!(
            keymap.key_grabs(&hotkey(
                Keysym::a,
                &[ActiveModifier::Logo, ActiveModifier::Shift]
            )),
            vec![grab(KEY_A, SHIFT | LOGO)]
        );
    }

    #[test]
    fn a_shifted_keysym_is_grabbed_with_the_modifiers_reaching_its_level() {
        let keymap = us_keymap();
        assert_eq!(
            keymap.key_grabs(&hotkey(Keysym::exclam, &[])),
            vec![grab(KEY_1, SHIFT)]
        );
        assert_eq!(
            keymap.key_grabs(&hotkey(Keysym::A, &[])),
            vec![grab(KEY_A, SHIFT), grab(KEY_A, LOCK)]
        );
    }

    #[test]
    fn a_keysym_no_key_produces_has_no_grabs() {
        assert!(
            us_keymap()
                .key_grabs(&hotkey(Keysym::adiaeresis, &[]))
                .is_empty()
        );
    }

    #[test]
    fn the_lock_bits_are_caps_lock_and_num_lock() {
        assert_eq!(us_keymap().lock_bits(), LOCK | NUM_LOCK);
    }
}
