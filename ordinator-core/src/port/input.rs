use crate::model::key::KeyPress;

pub trait Input {
    fn capture_one(&self, keys: &[KeyPress]) -> KeyPress;

    fn capture_any(&self) -> KeyPress;
}
