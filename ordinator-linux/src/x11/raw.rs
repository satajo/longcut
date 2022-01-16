use ordinator_core::model::key::{Key, Modifier, Symbol};
use std::convert::TryFrom;
use std::ffi::{c_void, CStr, CString};
use std::ops::BitAnd;
use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_ulong};
use std::ptr;
use x11::xlib::{
    ControlMask, CurrentTime, Display, GrabModeAsync, KeyPress, Mod1Mask, Mod4Mask, NoSymbol,
    ShiftMask, Window, XCloseDisplay, XCreateIC, XDefaultRootWindow, XEvent, XGrabKey,
    XGrabKeyboard, XIMPreeditNothing, XIMStatusNothing, XKeyEvent, XKeysymToKeycode,
    XKeysymToString, XNClientWindow, XNInputStyle, XNextEvent, XOpenDisplay, XOpenIM,
    XStringToKeysym, XUngrabKey, XUngrabKeyboard, XkbKeycodeToKeysym, Xutf8LookupString, XIC, XIM,
};

pub struct X11Handle {
    display: *mut Display,
    window: Window,
    input_context: XIC,
}

impl X11Handle {
    pub fn new() -> Self {
        let display = unsafe { XOpenDisplay(ptr::null()) };
        let window = unsafe { XDefaultRootWindow(display) };
        let input_method = Self::load_input_method(display).expect("Failed to load input method");
        let input_context =
            Self::load_input_context(&window, &input_method).expect("Failed to load input context");

        Self {
            display,
            input_context,
            window,
        }
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
            let x_event = self.read_next_event();
            if x_event.get_type() == KeyPress {
                let event = XKeyEvent::from(x_event);
                let modifiers = event.state;

                // TODO: Simplify this if and when Key model distinguishes 'control' and 'normal' characters.
                // The key symbol is parsed based on both what key was pressed and what character
                // that press ended up generating. The idea is that if a "control character" such as
                // backspace, F5, or Arrow Left was pressed, that character is mapped into a distinct
                // enum value. If that is not the case, then the unicode value is used.
                let key_name = self.parse_key_name(&event);
                let typed_character = self.parse_typed_character(&event);

                println!("'{:?}', {:?}", key_name, typed_character);

                let symbol = match (key_name, typed_character) {
                    (None, None) => None,
                    (Some(key), None) => Symbol::try_from(key.as_str()).ok(),
                    (None, Some(character)) => Symbol::try_from(character.as_str()).ok(),
                    (Some(key), Some(character)) => {
                        if let Ok(symbol) = Symbol::try_from(key.as_str()) {
                            if let Symbol::Character(_) = symbol {
                                Symbol::try_from(character.as_str()).ok()
                            } else {
                                Some(symbol)
                            }
                        } else {
                            Symbol::try_from(character.as_str()).ok()
                        }
                    }
                };

                // Assuming we managed to parse a valid symbol from the input, the active modifiers
                // information is appended to it.
                if let Some(symbol) = symbol {
                    let mut press = Key::new(symbol);

                    let is_mod_active = |mask| mask == modifiers.bitand(mask);
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

                    return press;
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

    fn parse_key_name(&self, event: &XKeyEvent) -> Option<String> {
        unsafe {
            let sym = XkbKeycodeToKeysym(self.display, event.keycode as c_uchar, 0, 0);
            let symbol = XKeysymToString(sym);

            // Null is returned when the specified Keysym is not defined.
            if symbol.is_null() {
                return None;
            }

            Some(CStr::from_ptr(symbol).to_string_lossy().into_owned())
        }
    }

    fn parse_typed_character(&self, event: &XKeyEvent) -> Option<String> {
        const BUFFER_LENGTH: usize = 4;
        let mut char_buffer: [c_char; BUFFER_LENGTH] = [0; BUFFER_LENGTH];
        let char_buffer_ptr = char_buffer.as_mut_ptr();

        let bytes_returned = unsafe {
            let mut keysym_return = 0;
            let status_return = ptr::null_mut();
            Xutf8LookupString(
                self.input_context,
                &mut event.clone(),
                char_buffer_ptr,
                BUFFER_LENGTH as c_int,
                &mut keysym_return,
                status_return,
            )
        };

        // The input has no valid character representation. This for example occurs on presses
        // of modifier and navigation keys.
        if bytes_returned == 0 {
            return None;
        }

        // Converting the potentially returned symbol into a character again.
        let char_str = unsafe { CStr::from_ptr(char_buffer_ptr) };
        Some(char_str.to_string_lossy().into_owned())
    }

    fn key_to_x11_keysym(key: &Key) -> String {
        match &key.symbol {
            Symbol::Character(c) => c.to_string(),
            Symbol::AltL => "Alt_L".to_string(),
            Symbol::AltR => "Alt_R".to_string(),
            Symbol::ShiftL => "Shift_L".to_string(),
            Symbol::ShiftR => "Shift_R".to_string(),
            Symbol::SuperL => "Super_L".to_string(),
            Symbol::SuperR => "Super_R".to_string(),
            otherwise => format!("{:?}", otherwise),
        }
    }

    fn load_input_method(display: *mut Display) -> Option<XIM> {
        let xim = unsafe { XOpenIM(display, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) };
        if xim.is_null() {
            None
        } else {
            Some(xim)
        }
    }

    fn load_input_context(window: &Window, method: &XIM) -> Option<XIC> {
        let xic = unsafe {
            let xn_input_style = CString::new(XNInputStyle).unwrap();
            let xn_client_window = CString::new(XNClientWindow).unwrap();

            XCreateIC(
                *method,
                xn_input_style.as_ptr(),
                XIMPreeditNothing | XIMStatusNothing,
                xn_client_window.as_ptr(),
                *window,
                ptr::null_mut::<c_void>(),
            )
        };

        if xic.is_null() {
            None
        } else {
            Some(xic)
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
        let name = handle.parse_key_name(36).unwrap();
        assert_eq!(name, "Return".to_string())
    }
}
