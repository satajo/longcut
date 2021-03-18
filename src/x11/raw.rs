use std::os::raw::c_int;
use std::ptr;
use x11::xlib::{
    CurrentTime, Display, GrabModeAsync, KeyPress, KeySym, Window, XDefaultRootWindow, XEvent,
    XGrabKey, XGrabKeyboard, XKeyEvent, XKeysymToKeycode, XNextEvent, XOpenDisplay, XUngrabKey,
    XUngrabKeyboard,
};

pub struct X11Handle {
    display: *mut Display,
    window: Window,
}

impl X11Handle {
    pub fn new() -> Self {
        let display = unsafe { XOpenDisplay(ptr::null()) };
        let window = unsafe { XDefaultRootWindow(display) };
        return Self { display, window };
    }

    pub fn read_next_keypress(&self) -> u32 {
        loop {
            let mut event = XEvent { pad: [0; 24] };
            unsafe {
                XNextEvent(self.display, &mut event);
            }
            if event.get_type() == KeyPress {
                return unsafe { event.key.keycode };
            }
        }
    }

    pub fn grab_key(&self, keycode: u32) {
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

    pub fn free_key(&self, keycode: u32) {
        unsafe { XUngrabKey(self.display, keycode as c_int, 0, self.window) };
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
}
