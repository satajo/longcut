mod raw;

use crate::x11::raw::X11Handle;
use ordinator_core::model::key::KeyPress;
use ordinator_core::port::input::Input;

pub struct X11 {
    handle: X11Handle,
}

impl X11 {
    pub fn new() -> Self {
        X11 {
            handle: X11Handle::new(),
        }
    }
}

impl Input for X11 {
    fn capture_one(&self, keys: &[KeyPress]) -> KeyPress {
        self.handle.grab_keys(keys);
        let press = self.handle.read_next_keypress();
        self.handle.free_keys(keys);
        press
    }

    fn capture_any(&self) -> KeyPress {
        self.handle.grab_keyboard();
        let press = self.handle.read_next_keypress();
        self.handle.free_keyboard();
        press
    }
}
