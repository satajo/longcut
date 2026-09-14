#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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

    /// A sum past `u32::MAX` is `u32::MAX`.
    #[must_use]
    pub fn saturating_add(self, other: Self) -> Self {
        Self {
            horizontal: self.horizontal.saturating_add(other.horizontal),
            vertical: self.vertical.saturating_add(other.vertical),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition_sums_each_axis() {
        assert_eq!(
            Position::new(1, 2).saturating_add(Position::new(10, 20)),
            Position::new(11, 22)
        );
    }

    #[test]
    fn addition_saturates_at_u32_max() {
        assert_eq!(
            Position::new(u32::MAX, u32::MAX - 1).saturating_add(Position::new(1, 2)),
            Position::new(u32::MAX, u32::MAX)
        );
    }
}
