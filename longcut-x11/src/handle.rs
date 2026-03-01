use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_ulong, c_void};
use std::ops::BitAnd;
use std::ptr;
use x11::xlib::{
    Atom, CurrentTime, Display, GrabModeAsync, KeyPress, NoSymbol, XA_STRING, XA_WINDOW,
    XCloseDisplay, XCreateIC, XDefaultRootWindow, XEvent, XFree, XGetWindowProperty, XGrabKey,
    XGrabKeyboard, XIC, XID, XIM, XIMPreeditNothing, XIMStatusNothing, XInternAtom, XKeyEvent,
    XKeysymToKeycode, XKeysymToString, XNClientWindow, XNInputStyle, XNextEvent, XOpenDisplay,
    XOpenIM, XStringToKeysym, XSync, XUngrabKey, XUngrabKeyboard, XkbKeycodeToKeysym,
    Xutf8LookupString,
};

pub struct X11Handle {
    display: *mut Display,
    input_context: XIC,
    root_window: XID,
}

#[derive(Debug)]
pub struct X11KeyPress {
    event: XKeyEvent,
    pub modmask: u32,
    pub keycode: u8,
}

impl X11KeyPress {
    pub fn is_mod_active(&self, mask: c_uint) -> bool {
        mask == self.modmask.bitand(mask)
    }
}

impl X11Handle {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let display = unsafe { XOpenDisplay(ptr::null()) };
        let root_window = unsafe { XDefaultRootWindow(display) };
        let input_context = Self::load_input_context(display, root_window)
            .expect("Failed to load X11 input context");

        Self {
            display,
            input_context,
            root_window,
        }
    }

    pub fn grab_key(&self, keycode: u8) {
        unsafe {
            XGrabKey(
                self.display,
                keycode as c_int,
                0,
                self.root_window,
                true as c_int,
                GrabModeAsync,
                GrabModeAsync,
            )
        };
    }

    pub fn grab_keys(&self, keys: impl IntoIterator<Item = u8>) {
        for key in keys {
            self.grab_key(key);
        }
    }

    pub fn free_key(&self, keycode: u8) {
        unsafe { XUngrabKey(self.display, keycode as c_int, 0, self.root_window) };
    }

    pub fn free_keys(&self, keys: impl IntoIterator<Item = u8>) {
        for key in keys {
            self.free_key(key);
        }
    }

    pub fn grab_keyboard(&self) {
        unsafe {
            XGrabKeyboard(
                self.display,
                self.root_window,
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
            // Discard any key events that were queued during the grab period, so they
            // don't bleed into the next input-capture call.
            XSync(self.display, 1);
        }
    }

    /// Blocks on the next XEvent of KeyPress type to happen, and returns the keycode and mod mask
    /// tuple of the key.
    pub fn read_next_keypress(&self) -> X11KeyPress {
        loop {
            let x_event = self.read_next_event();
            if x_event.get_type() == KeyPress {
                let event = XKeyEvent::from(x_event);
                return X11KeyPress {
                    event,
                    modmask: event.state,
                    keycode: event.keycode as u8,
                };
            }
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

    /// Returns the character corresponding to the X11KeyPress.
    ///
    /// Can return both simple ASCII characters a, b, c, etc. or whole Unicode graphemes, depending
    /// on the input.
    ///
    /// Control characters such as the arrow or modifier keys do not have a character representation,
    /// and for them None is returned.
    pub fn keypress_to_grapheme(&self, press: &X11KeyPress) -> Option<String> {
        const BUFFER_LENGTH: usize = 4;
        let mut char_buffer: [c_char; BUFFER_LENGTH] = [0; BUFFER_LENGTH];
        let char_buffer_ptr = char_buffer.as_mut_ptr();

        let bytes_returned = unsafe {
            let mut keysym_return = 0;
            let status_return = ptr::null_mut();
            Xutf8LookupString(
                self.input_context,
                &mut press.event.clone(),
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

        // Converting the returned symbol into a character again.
        let char_str = unsafe { CStr::from_ptr(char_buffer_ptr) };
        Some(char_str.to_string_lossy().into_owned())
    }

    /// Returns the key symbol name corresponding to the X11KeyPress.
    ///
    /// The conversion is performed by looking up the key name based of the key code. This means all
    /// modifier information is lost, and the returned symbol might not correspond to the one printed
    /// onto the physical keycap.
    ///
    /// For control characters a string representation of the key name is returned.
    pub fn keypress_to_key_name(&self, press: &X11KeyPress) -> Option<String> {
        unsafe {
            let sym = XkbKeycodeToKeysym(self.display, press.keycode, 0, 0);
            let symbol = XKeysymToString(sym);

            // Null is returned when the specified Keysym is not defined.
            if symbol.is_null() {
                return None;
            }

            Some(CStr::from_ptr(symbol).to_string_lossy().into_owned())
        }
    }

    /// Returns the XID of the currently focused window via `_NET_ACTIVE_WINDOW`, or `None` if
    /// the property is unavailable (e.g. no EWMH-compliant compositor is running).
    pub fn get_active_window(&self) -> Option<XID> {
        let atom_name = CString::new("_NET_ACTIVE_WINDOW").unwrap();
        let atom = unsafe { XInternAtom(self.display, atom_name.as_ptr(), 0) };
        unsafe {
            let mut prop: *mut u8 = ptr::null_mut();
            let mut actual_type: Atom = 0;
            let mut actual_format: c_int = 0;
            let mut nitems: c_ulong = 0;
            let mut bytes_after: c_ulong = 0;

            let status = XGetWindowProperty(
                self.display,
                self.root_window,
                atom,
                0,
                1,
                0,
                XA_WINDOW,
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut prop,
            );

            if status != 0 || prop.is_null() || nitems == 0 {
                if !prop.is_null() {
                    XFree(prop as *mut c_void);
                }
                return None;
            }

            let window = *(prop as *mut XID);
            XFree(prop as *mut c_void);
            Some(window)
        }
    }

    /// Returns the `WM_CLASS` property of the window as `(instance_name, class_name)`, or `None`
    /// if the property is absent. The two values are the null-separated parts of the raw property.
    pub fn get_window_class(&self, window: XID) -> Option<(String, String)> {
        let prop_name = CString::new("WM_CLASS").unwrap();
        let prop_atom = unsafe { XInternAtom(self.display, prop_name.as_ptr(), 0) };

        unsafe {
            let mut prop: *mut u8 = ptr::null_mut();
            let mut actual_type: Atom = 0;
            let mut actual_format: c_int = 0;
            let mut nitems: c_ulong = 0;
            let mut bytes_after: c_ulong = 0;

            let status = XGetWindowProperty(
                self.display,
                window,
                prop_atom,
                0,
                1024,
                0,
                XA_STRING,
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut prop,
            );

            if status != 0 || prop.is_null() || nitems == 0 {
                if !prop.is_null() {
                    XFree(prop as *mut c_void);
                }
                return None;
            }

            // WM_CLASS is two null-terminated strings concatenated: instance\0class\0
            let data = std::slice::from_raw_parts(prop, nitems as usize);
            let mut parts = data.splitn(2, |&b| b == 0);
            let instance = parts
                .next()
                .and_then(|s| std::str::from_utf8(s).ok())
                .map(str::to_owned);
            let class = parts
                .next()
                .map(|s| s.split(|&b| b == 0).next().unwrap_or(s))
                .and_then(|s| std::str::from_utf8(s).ok())
                .map(str::to_owned);

            XFree(prop as *mut c_void);
            instance.zip(class)
        }
    }

    fn read_next_event(&self) -> XEvent {
        let mut event = XEvent { pad: [0; 24] };
        unsafe {
            XNextEvent(self.display, &mut event);
        }
        event
    }

    fn load_input_context(display: *mut Display, window: XID) -> Option<XIC> {
        let xim = Self::load_input_method(display)?;
        let xic = unsafe {
            let xn_input_style = CString::new(XNInputStyle).unwrap();
            let xn_client_window = CString::new(XNClientWindow).unwrap();

            XCreateIC(
                xim,
                xn_input_style.as_ptr(),
                XIMPreeditNothing | XIMStatusNothing,
                xn_client_window.as_ptr(),
                window,
                ptr::null_mut::<c_void>(),
            )
        };
        xic.into()
    }

    fn load_input_method(display: *mut Display) -> Option<XIM> {
        let xim = unsafe { XOpenIM(display, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()) };
        xim.into()
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
#[cfg(feature = "x11-tests")]
mod tests {
    use super::X11Handle;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_string_to_keycode() {
        let x11 = X11Handle::new();
        let keycode = x11.string_to_keycode("Return").unwrap();
        assert_eq!(keycode, 36)
    }
}
