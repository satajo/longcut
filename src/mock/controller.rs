use crate::core::{Controller, ControllerEvent};
use std::thread;
use std::time::Duration;

pub struct MockController {}

impl MockController {
    pub fn new() -> MockController {
        return MockController {};
    }
}

impl Controller for MockController {
    fn read_event(&self) -> ControllerEvent {
        thread::sleep(Duration::from_secs(1));
        println!("Emitting fake event!");
        return ControllerEvent::End;
    }
}
