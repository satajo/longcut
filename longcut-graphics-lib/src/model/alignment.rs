#[derive(Debug, Clone, Copy)]
pub enum Alignment {
    Beginning,
    Center,
    End,
}

#[derive(Debug, Clone, Copy)]
pub struct Alignment2d {
    pub horizontal: Alignment,
    pub vertical: Alignment,
}

impl Alignment2d {
    pub fn new(horizontal: Alignment, vertical: Alignment) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}
