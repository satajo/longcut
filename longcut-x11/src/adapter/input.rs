use crate::X11Module;
use longcut_core::model::key::Key;
use longcut_core::port::input::Input;

pub struct X11Input<'a> {
    x11: &'a X11Module,
}

impl<'a> X11Input<'a> {
    pub fn new(x11: &'a X11Module) -> Self {
        Self { x11 }
    }
}

impl<'a> Input for X11Input<'a> {
    fn capture_one(&self, keys: &[Key]) -> Key {
        self.x11.grab_keys(keys);
        let press = self.x11.read_next_keypress();
        self.x11.free_keys(keys);
        press
    }

    fn capture_any(&self) -> Key {
        self.x11.grab_keyboard();
        let press = self.x11.read_next_keypress();
        self.x11.free_keyboard();
        press
    }
}
