mod raw;

use crate::core::model::key::Key;
use crate::core::port::controller::{Controller, ControllerEvent};
use crate::x11::raw::X11Handle;

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
                for code in codes {
                    self.handle.grab_key(*code);
                }
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
                for code in codes {
                    self.handle.grab_key(*code);
                }
            }
            (CaptureMode::Some(codes), CaptureMode::None()) => {
                println!("Releasing codes {:?}!", codes);
                for code in codes {
                    self.handle.free_key(*code);
                }
            }
            (CaptureMode::Some(current), CaptureMode::Some(desired)) => {
                println!(
                    "Releasing codes {:?} and capturing codes {:?}!",
                    current, desired
                );
                for code in current {
                    self.handle.free_key(*code);
                }
                for code in desired {
                    self.handle.grab_key(*code);
                }
            }
            (CaptureMode::Some(codes), CaptureMode::All()) => {
                println!("Releasing codes {:?}, grabbing keyboard!", codes);
                for code in codes {
                    self.handle.free_key(*code);
                }
                self.handle.grab_keyboard();
            }
            _ => {}
        }

        self.mode = mode;
    }
}

impl Controller for X11 {
    fn capture_one_of(&mut self, keys: &Vec<Key>) -> Key {
        let codes = keys.iter().map(|key| key.code).collect();
        self.set_capture_mode(CaptureMode::Some(codes));

        let keycode = self.handle.read_next_keypress();
        Key::from_keycode(keycode)
    }

    fn capture_all(&mut self) -> Key {
        self.set_capture_mode(CaptureMode::All());

        let keycode = self.handle.read_next_keypress();
        Key::from_keycode(keycode)
    }
}
