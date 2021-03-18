use crate::core::model::key::Key;

pub enum ControllerEvent {
    Begin,
    End,
}

pub trait Controller {
    fn capture_one_of(&mut self, keys: &Vec<Key>) -> Key;

    fn capture_all(&mut self) -> Key;
}
