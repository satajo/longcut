use crate::model::key::KeyPress;

pub trait Input {
    fn capture_one(&self, keys: &Vec<KeyPress>) -> KeyPress;

    fn capture_any(&self) -> KeyPress;
}
