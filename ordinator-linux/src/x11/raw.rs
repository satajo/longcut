use ordinator_core::model::key::{Key, Modifier, Symbol};
use std::convert::TryFrom;
use std::ffi::{CStr, CString};
use std::ops::BitAnd;
use std::os::raw::{c_int, c_uint};
use std::ptr;
use x11::xlib::{
    ControlMask, CurrentTime, Display, GrabModeAsync, KeyPress, Mod1Mask, Mod4Mask, NoSymbol,
    ShiftMask, Window, XCloseDisplay, XDefaultRootWindow, XEvent, XGrabKey, XGrabKeyboard,
    XKeysymToKeycode, XKeysymToString, XNextEvent, XOpenDisplay, XStringToKeysym, XUngrabKey,
    XUngrabKeyboard, XkbKeycodeToKeysym,
};

pub struct X11Handle {
    display: *mut Display,
    window: Window,
}

impl X11Handle {
    pub fn new() -> Self {
        let display = unsafe { XOpenDisplay(ptr::null()) };
        let window = unsafe { XDefaultRootWindow(display) };
        Self { display, window }
    }

    fn read_next_event(&self) -> XEvent {
        let mut event = XEvent { pad: [0; 24] };
        unsafe {
            XNextEvent(self.display, &mut event);
        }
        event
    }

    pub fn read_next_keypress(&self) -> Key {
        // X keypress is represented as a key string name and a bitmask of the active modifiers.
        fn parse_keypress(key_name: &str, key_mods: c_uint) -> Result<Key, &'static str> {
            let symbol = Symbol::try_from(key_name)?;
            let mut press = Key::new(symbol);

            let is_mod_active = |mask| mask == key_mods.bitand(mask);
            if is_mod_active(ShiftMask) {
                press.add_modifier(Modifier::Shift);
            }

            if is_mod_active(ControlMask) {
                press.add_modifier(Modifier::Control);
            }

            if is_mod_active(Mod1Mask) {
                press.add_modifier(Modifier::Alt);
            }

            if is_mod_active(Mod4Mask) {
                press.add_modifier(Modifier::Super);
            }

            Ok(press)
        }

        loop {
            let event = self.read_next_event();
            if event.get_type() == KeyPress {
                let key_code = unsafe { event.key.keycode };
                let key_mods = unsafe { event.key.state };
                let key_name = self.keycode_to_string(key_code as u8);
                if let Ok(key) = parse_keypress(&key_name, key_mods) {
                    return key;
                }
            }
        }
    }

    pub fn grab_key(&self, key: &Key) {
        let key_string = Self::key_to_x11_keysym(key);
        if let Some(keycode) = self.string_to_keycode(&key_string) {
            unsafe {
                XGrabKey(
                    self.display,
                    keycode as c_int,
                    0,
                    self.window,
                    true as c_int,
                    GrabModeAsync,
                    GrabModeAsync,
                )
            };
        }
    }

    pub fn free_key(&self, key: &Key) {
        let key_string = Self::key_to_x11_keysym(key);
        if let Some(keycode) = self.string_to_keycode(&key_string) {
            unsafe { XUngrabKey(self.display, keycode as c_int, 0, self.window) };
        }
    }

    pub fn grab_keys<'a>(&self, keys: impl IntoIterator<Item = &'a Key>) {
        for key in keys {
            self.grab_key(key);
        }
    }

    pub fn free_keys<'a>(&self, keys: impl IntoIterator<Item = &'a Key>) {
        for key in keys {
            self.free_key(key);
        }
    }

    pub fn grab_keyboard(&self) {
        unsafe {
            XGrabKeyboard(
                self.display,
                self.window,
                true as c_int,
                GrabModeAsync,
                GrabModeAsync,
                CurrentTime,
            );
        }
    }

    pub fn free_keyboard(&self) {
        unsafe {
            XUngrabKeyboard(self.display, CurrentTime);
        }
    }

    pub fn string_to_keycode(&self, symbol: &str) -> Option<u8> {
        let c_str = CString::new(symbol).expect("Symbol must not be null-terminated");

        let symbol = unsafe { XStringToKeysym(c_str.as_ptr()) };
        if symbol as i32 == NoSymbol {
            return None;
        }

        let keycode = unsafe { XKeysymToKeycode(self.display, symbol) };
        if keycode == 0 {
            return None;
        }

        Some(keycode)
    }

    pub fn keycode_to_string(&self, code: u8) -> String {
        let slice = unsafe {
            let sym = XkbKeycodeToKeysym(self.display, code, 0, 0);
            let symbol = XKeysymToString(sym);
            CStr::from_ptr(symbol).to_str()
        };

        match slice {
            Ok(str) => str.to_string(),
            Err(_) => "".to_string(),
        }
    }

    fn key_to_x11_keysym(key: &Key) -> String {
        match &key.symbol {
            Symbol::Character(c) => c.to_string(),
            otherwise => format!("{:?}", otherwise),
        }
    }
}

impl Drop for X11Handle {
    fn drop(&mut self) {
        unsafe {
            XCloseDisplay(self.display);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_keycode() {
        let handle = X11Handle::new();
        let keycode = handle.string_to_keycode("Return").unwrap();
        assert_eq!(keycode, 36)
    }

    #[test]
    fn test_keycode_to_string() {
        let handle = X11Handle::new();
        let name = handle.keycode_to_string(36);
        assert_eq!(name, "Return".to_string())
    }
}
