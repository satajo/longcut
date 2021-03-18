use crate::core::model::key::Key;
use crate::core::port::controller::{Controller, ControllerEvent};
use rand;
use rand::Rng;
use std::cell::RefCell;
use std::thread;
use std::time::Duration;

pub struct MockController {
    rng: rand::rngs::ThreadRng,
}

impl MockController {
    pub fn new() -> MockController {
        return MockController {
            rng: rand::thread_rng(),
        };
    }
}

impl Controller for MockController {
    fn capture_one_of(&mut self, keys: &Vec<Key>) -> Key {
        Key::from_keycode(keys.first().unwrap().code)
    }

    fn capture_all(&mut self) -> Key {
        thread::sleep(Duration::from_secs(1));
        let random_keycode: u32 = self.rng.gen_range(1, 100);
        Key::from_keycode(random_keycode)
    }
}
