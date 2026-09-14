#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub fn intersect(self, other: Self) -> Self {
        Self {
            width: u32::min(self.width, other.width),
            height: u32::min(self.height, other.height),
        }
    }

    /// A sum past `u32::MAX` is `u32::MAX`.
    #[must_use]
    pub fn saturating_add(self, other: Self) -> Self {
        Self {
            width: self.width.saturating_add(other.width),
            height: self.height.saturating_add(other.height),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition_sums_each_side() {
        assert_eq!(
            Dimensions::new(1, 2).saturating_add(Dimensions::new(10, 20)),
            Dimensions::new(11, 22)
        );
    }

    #[test]
    fn addition_saturates_at_u32_max() {
        assert_eq!(
            Dimensions::new(u32::MAX, u32::MAX - 1).saturating_add(Dimensions::new(1, 2)),
            Dimensions::new(u32::MAX, u32::MAX)
        );
    }
}
