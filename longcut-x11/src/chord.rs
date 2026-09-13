/// The keys the user held to launch a session, tracked until they are released.
///
/// The user launches a session by pressing a hotkey, typically a chord such as Super+w, and the
/// first keys of the session often arrive while that chord is still held. Its modifiers are not
/// part of what the user is typing, so they are taken out of every press until released, and the
/// hotkey repeating while it is held is not a press at all.
///
/// A modifier counts as part of the chord while every key event since the launch has
/// reported it held. The state a key event carries describes the modifiers before that event,
/// so a modifier released and pressed again stops being part of the chord at the first event
/// that sees it up.
pub struct LaunchChord {
    hotkey: u8,
    hotkey_held: bool,
    modifiers: u16,
}

/// The bits of a key event's state that are modifiers; the rest are pointer buttons and the
/// keyboard group.
const MODIFIER_BITS: u16 = 0xff;

impl LaunchChord {
    #[must_use]
    pub fn new(hotkey: u8) -> Self {
        Self {
            hotkey,
            hotkey_held: true,
            modifiers: MODIFIER_BITS,
        }
    }

    /// Accounts for a key press and returns the state to resolve it under, or `None` when the
    /// press is the held hotkey repeating.
    #[must_use]
    pub fn press(&mut self, keycode: u8, state: u16) -> Option<u16> {
        self.modifiers &= state & MODIFIER_BITS;
        if keycode == self.hotkey && self.hotkey_held {
            return None;
        }
        Some(state & !self.modifiers)
    }

    /// Accounts for a key release.
    pub fn release(&mut self, keycode: u8, state: u16) {
        self.modifiers &= state & MODIFIER_BITS;
        if keycode == self.hotkey {
            self.hotkey_held = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY_F: u8 = 41;
    const KEY_F12: u8 = 96;
    const KEY_CONTROL_L: u8 = 37;
    const KEY_SUPER_L: u8 = 133;
    const KEY_SUPER_R: u8 = 134;

    const CONTROL: u16 = 1 << 2;
    const SUPER: u16 = 1 << 6;
    const GROUP_2: u16 = 1 << 13;

    #[test]
    fn a_modifier_held_since_the_launch_is_not_part_of_a_press() {
        let mut chord = LaunchChord::new(KEY_SUPER_L);
        assert_eq!(chord.press(KEY_F, SUPER), Some(0));
    }

    #[test]
    fn a_modifier_pressed_after_the_launch_is_part_of_a_press() {
        let mut chord = LaunchChord::new(KEY_SUPER_L);
        assert_eq!(chord.press(KEY_CONTROL_L, SUPER), Some(0));
        assert_eq!(chord.press(KEY_F, SUPER | CONTROL), Some(CONTROL));
    }

    #[test]
    fn a_launch_modifier_pressed_again_after_its_release_is_part_of_a_press() {
        let mut chord = LaunchChord::new(KEY_SUPER_L);
        chord.release(KEY_SUPER_L, SUPER);
        assert_eq!(chord.press(KEY_SUPER_R, 0), Some(0));
        assert_eq!(chord.press(KEY_F, SUPER), Some(SUPER));
    }

    #[test]
    fn the_hotkey_repeating_while_held_is_not_a_press() {
        let mut chord = LaunchChord::new(KEY_F12);
        assert_eq!(chord.press(KEY_F12, 0), None);
        chord.release(KEY_F12, 0);
        assert_eq!(chord.press(KEY_F12, 0), Some(0));
    }

    #[test]
    fn the_keyboard_group_of_a_press_is_kept() {
        let mut chord = LaunchChord::new(KEY_SUPER_L);
        assert_eq!(chord.press(KEY_F, SUPER | GROUP_2), Some(GROUP_2));
    }
}
