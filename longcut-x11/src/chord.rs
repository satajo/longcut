/// The keys the user held when the keyboard was taken, tracked until they are released.
///
/// The user takes the keyboard by pressing a launch key, typically a chord such as Super+w, and
/// the first keys of the session often arrive while that chord is still held. Its modifiers are
/// not part of what the user is typing, so they are taken out of every press until released, and
/// a held key repeating is not a press at all.
///
/// The held keys come from a snapshot of the keyboard taken right after the keyboard itself. A
/// press the server generated before the snapshot is a press regardless: the key went down after
/// the keyboard was taken and was merely still down when the snapshot was made.
///
/// A modifier counts as part of the chord while every key event since the keyboard was taken
/// has reported it held. The state a key event carries describes the modifiers before that event,
/// so a modifier released and pressed again stops being part of the chord at the first event
/// that sees it up.
#[derive(Debug)]
pub(crate) struct LaunchChord {
    /// One bit per keycode, set while the key has been down since before the snapshot.
    held: [u8; 32],
    /// The sequence number of the snapshot request.
    snapshot_sequence: u64,
    modifiers: u16,
}

/// The bits of a key event's state that are modifiers; the rest are pointer buttons and the
/// keyboard group.
const MODIFIER_BITS: u16 = 0xff;

impl LaunchChord {
    /// Starts the chord from the keys the snapshot found held, as the X server's key bit vector:
    /// bit `k % 8` of byte `k / 8` is key `k`.
    #[must_use]
    pub(crate) fn new(held: [u8; 32], snapshot_sequence: u64) -> Self {
        Self {
            held,
            snapshot_sequence,
            modifiers: MODIFIER_BITS,
        }
    }

    /// Accounts for a key press and returns the state to resolve it under, or `None` when the
    /// press is a key held since before the keyboard was taken repeating. `sequence` is the
    /// sequence number the server gave the event.
    #[must_use]
    pub(crate) fn press(&mut self, keycode: u8, state: u16, sequence: u64) -> Option<u16> {
        self.modifiers &= state & MODIFIER_BITS;
        if sequence < self.snapshot_sequence {
            self.clear(keycode);
        } else if self.is_held(keycode) {
            return None;
        }
        Some(state & !self.modifiers)
    }

    /// Accounts for a key release.
    pub(crate) fn release(&mut self, keycode: u8, state: u16) {
        self.modifiers &= state & MODIFIER_BITS;
        self.clear(keycode);
    }

    fn is_held(&self, keycode: u8) -> bool {
        #[expect(
            clippy::indexing_slicing,
            reason = "keycode / 8 is at most 31, the last index of the 32-byte key bit vector"
        )]
        let byte = self.held[usize::from(keycode / 8)];
        byte & (1 << (keycode % 8)) != 0
    }

    fn clear(&mut self, keycode: u8) {
        #[expect(
            clippy::indexing_slicing,
            reason = "keycode / 8 is at most 31, the last index of the 32-byte key bit vector"
        )]
        let byte = &mut self.held[usize::from(keycode / 8)];
        *byte &= !(1 << (keycode % 8));
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

    const SNAPSHOT: u64 = 10;
    const BEFORE_SNAPSHOT: u64 = 9;
    const AFTER_SNAPSHOT: u64 = 10;

    fn chord_with_held(keys: &[u8]) -> LaunchChord {
        let mut held = [0; 32];
        for key in keys {
            held[usize::from(key / 8)] |= 1 << (key % 8);
        }
        LaunchChord::new(held, SNAPSHOT)
    }

    #[test]
    fn a_modifier_held_since_the_launch_is_not_part_of_a_press() {
        let mut chord = chord_with_held(&[KEY_SUPER_L]);
        assert_eq!(chord.press(KEY_F, SUPER, AFTER_SNAPSHOT), Some(0));
    }

    #[test]
    fn a_modifier_pressed_after_the_launch_is_part_of_a_press() {
        let mut chord = chord_with_held(&[KEY_SUPER_L]);
        assert_eq!(chord.press(KEY_CONTROL_L, SUPER, AFTER_SNAPSHOT), Some(0));
        assert_eq!(
            chord.press(KEY_F, SUPER | CONTROL, AFTER_SNAPSHOT),
            Some(CONTROL)
        );
    }

    #[test]
    fn a_launch_modifier_pressed_again_after_its_release_is_part_of_a_press() {
        let mut chord = chord_with_held(&[KEY_SUPER_L]);
        chord.release(KEY_SUPER_L, SUPER);
        assert_eq!(chord.press(KEY_SUPER_R, 0, AFTER_SNAPSHOT), Some(0));
        assert_eq!(chord.press(KEY_F, SUPER, AFTER_SNAPSHOT), Some(SUPER));
    }

    #[test]
    fn a_key_held_since_the_launch_repeating_is_not_a_press() {
        let mut chord = chord_with_held(&[KEY_F12]);
        assert_eq!(chord.press(KEY_F12, 0, AFTER_SNAPSHOT), None);
        chord.release(KEY_F12, 0);
        assert_eq!(chord.press(KEY_F12, 0, AFTER_SNAPSHOT), Some(0));
    }

    #[test]
    fn a_key_pressed_before_the_snapshot_is_a_press_and_so_are_its_repeats() {
        let mut chord = chord_with_held(&[KEY_SUPER_L, KEY_F]);
        assert_eq!(chord.press(KEY_F, SUPER, BEFORE_SNAPSHOT), Some(0));
        assert_eq!(chord.press(KEY_F, SUPER, AFTER_SNAPSHOT), Some(0));
    }

    #[test]
    fn the_keyboard_group_of_a_press_is_kept() {
        let mut chord = chord_with_held(&[KEY_SUPER_L]);
        assert_eq!(
            chord.press(KEY_F, SUPER | GROUP_2, AFTER_SNAPSHOT),
            Some(GROUP_2)
        );
    }
}
