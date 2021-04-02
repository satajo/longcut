use crate::model::key::KeyPress;

pub trait Input {
    fn capture_one(&mut self, keys: &Vec<KeyPress>) -> KeyPress;

    fn capture_any(&mut self) -> KeyPress;
}
