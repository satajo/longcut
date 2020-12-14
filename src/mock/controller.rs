use crate::core::{Controller, ControllerEvent};
use rand;
use rand::Rng;
use std::cell::RefCell;
use std::thread;
use std::time::Duration;

pub struct MockController {
    rng: RefCell<rand::rngs::ThreadRng>,
}

impl MockController {
    pub fn new() -> MockController {
        return MockController {
            rng: RefCell::new(rand::thread_rng()),
        };
    }
}

impl Controller for MockController {
    fn read_event(&self) -> ControllerEvent {
        thread::sleep(Duration::from_secs(1));
        return match self.rng.borrow_mut().gen_range(0, 2) {
            0 => ControllerEvent::Begin,
            _ => ControllerEvent::End,
        };
    }
}
