use crate::model::key::Key;

pub trait Input {
    /// Only capture input from the specified keys, returning the first Key encountered.
    fn capture_one(&self, keys: &[Key]) -> Key;

    /// Grab the entire input device and return an iterator that yields one Key per press.
    /// The grab is held for the iterator's lifetime and released on drop.
    fn capture_any_iter(&self) -> Box<dyn Iterator<Item = Key> + '_>;

    /// Capture the entire input device to read a single Key.
    fn capture_any(&self) -> Key {
        self.capture_any_iter().next().unwrap()
    }
}
