use crate::model::key::Key;

pub trait Input {
    fn capture_one(&self, keys: &[Key]) -> Key;

    fn capture_any(&self) -> Key;
}
