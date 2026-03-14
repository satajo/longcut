#[derive(Clone, Debug)]
pub struct Font {
    pub family: String,
    pub size: u8,
}

impl Font {
    #[must_use]
    pub fn new(family: String, size: u8) -> Self {
        Self { family, size }
    }
}
