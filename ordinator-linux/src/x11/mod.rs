mod raw;

use crate::x11::raw::X11Handle;
use ordinator_core::model::key::KeyPress;
use ordinator_core::port::input::Input;

#[derive(Debug, PartialEq)]
enum CaptureMode {
    None(),
    Some(Vec<u32>),
    All(),
}

pub struct X11 {
    handle: X11Handle,
    mode: CaptureMode,
}

impl X11 {
    pub fn new() -> Self {
        X11 {
            handle: X11Handle::new(),
            mode: CaptureMode::None(),
        }
    }

    fn set_capture_mode(&mut self, mode: CaptureMode) {
        match (&self.mode, &mode) {
            (CaptureMode::None(), CaptureMode::Some(codes)) => {
                println!("Grabbing codes {:?}!", codes);
                self.handle.grab_keys(codes);
            }
            (CaptureMode::None(), CaptureMode::All()) => {
                println!("Grabbing keyboard!");
                self.handle.grab_keyboard();
            }
            (CaptureMode::All(), CaptureMode::None()) => {
                println!("Releasing keyboard!");
                self.handle.free_keyboard();
            }
            (CaptureMode::All(), CaptureMode::Some(codes)) => {
                println!("Releasing keyboard, grabbing codes {:?}!", codes);
                self.handle.free_keyboard();
                self.handle.grab_keys(codes);
            }
            (CaptureMode::Some(codes), CaptureMode::None()) => {
                println!("Releasing codes {:?}!", codes);
                self.handle.free_keys(codes);
            }
            (CaptureMode::Some(current), CaptureMode::Some(desired)) => {
                println!(
                    "Releasing codes {:?} and capturing codes {:?}!",
                    current, desired
                );
                self.handle.free_keys(current);
                self.handle.grab_keys(desired);
            }
            (CaptureMode::Some(codes), CaptureMode::All()) => {
                println!("Releasing codes {:?}, grabbing keyboard!", codes);
                self.handle.free_keys(codes);
                self.handle.grab_keyboard();
            }
            _ => {}
        }

        self.mode = mode;
    }
}

impl Input for X11 {
    fn capture_one(&mut self, keys: &Vec<KeyPress>) -> KeyPress {
        let codes = keys.iter().map(|key| key.code).collect();
        self.set_capture_mode(CaptureMode::Some(codes));

        let keycode = self.handle.read_next_keypress();
        KeyPress::from_keycode(keycode)
    }

    fn capture_any(&mut self) -> KeyPress {
        self.set_capture_mode(CaptureMode::All());

        let keycode = self.handle.read_next_keypress();
        KeyPress::from_keycode(keycode)
    }
}
