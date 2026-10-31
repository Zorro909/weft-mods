/// Power a device gives to one side.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Power {
    pub strength: u8,
    pub strong: bool,
}

/// What arrives at one side of a receiver. `direct`: the neighbor itself is the source (wire or
/// device), not a block it powers.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct SidePower {
    pub strength: u8,
    pub strong: bool,
    pub direct: bool,
}

/// "Full strength", whatever the world's range is; `Range::clamp` turns it into the maximum.
pub const FULL: u8 = u8::MAX;

/// The world's signal range: strengths go from 0 to `max()`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Range {
    max: u8,
}

impl Range {
    /// From the `signal_range` setting's index (16, 32 or 64 levels).
    pub fn from_index(i: u32) -> Range {
        Range::from_index_const(i)
    }
    pub const fn from_index_const(i: u32) -> Range {
        Range { max: match i { 1 => 31, 2 => 63, _ => 15 } }
    }
    pub fn max(self) -> u8 {
        self.max
    }
    pub fn clamp(self, s: u8) -> u8 {
        s.min(self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_values() {
        assert_eq!([0, 1, 2, 7].map(|i| Range::from_index(i).max()), [15, 31, 63, 15]);
        assert_eq!(Range::from_index(0).clamp(FULL), 15);
        assert_eq!(Range::from_index(1).clamp(20), 20);
    }
}
