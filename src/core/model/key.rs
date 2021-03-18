#[derive(Clone, Debug)]
pub struct Key {
    pub code: u32,
}

impl Key {
    pub fn from_keycode(keycode: u32) -> Self {
        Self { code: keycode }
    }
}
