#[derive(Clone, Debug, Ord, Eq, PartialOrd, PartialEq)]
pub struct KeyPress {
    pub code: u32,
}

impl KeyPress {
    pub fn from_keycode(keycode: u32) -> Self {
        Self { code: keycode }
    }
}
