#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Position {
    pub horizontal: u32,
    pub vertical: u32,
}

impl Position {
    #[must_use]
    pub fn new(horizontal: u32, vertical: u32) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    #[must_use]
    pub fn zero() -> Self {
        Self::new(0, 0)
    }
}

impl std::ops::Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Position {
            horizontal: self.horizontal + rhs.horizontal,
            vertical: self.vertical + rhs.vertical,
        }
    }
}
