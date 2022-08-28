use std::ops::Add;

#[derive(Copy, Clone, Debug, Default)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            width: u32::min(self.width, other.width),
            height: u32::min(self.height, other.height),
        }
    }
}

impl Add for Dimensions {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            width: self.width + rhs.width,
            height: self.height + rhs.height,
        }
    }
}
