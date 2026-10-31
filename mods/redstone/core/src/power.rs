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
    /// No clamping at all: `FULL` stays `FULL`. For describing outputs independent of a world.
    pub const UNCLAMPED: Range = Range { max: u8::MAX };
    pub fn max(self) -> u8 {
        self.max
    }
    pub fn clamp(self, s: u8) -> u8 {
        s.min(self.max)
    }
    /// The brightness a wire of strength `s` shows with `levels` levels: 0 only when unpowered,
    /// the rest spread evenly over the range.
    pub fn level(self, s: u8, levels: u8) -> u8 {
        let s = self.clamp(s);
        if s == 0 || levels < 2 {
            return 0;
        }
        1 + ((s as u32 - 1) * (levels as u32 - 1) / self.max as u32) as u8
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

    #[test]
    fn levels_spread_over_range() {
        for i in 0..3 {
            let r = Range::from_index(i);
            let levels: Vec<u8> = (0..=r.max()).map(|s| r.level(s, 4)).collect();
            assert_eq!(levels[0], 0);
            assert_eq!(levels[1], 1);
            assert_eq!(*levels.last().unwrap(), 3);
            assert!(levels.windows(2).all(|w| w[1] >= w[0] && w[1] - w[0] <= 1));
            for l in 1..=3 {
                let n = levels.iter().filter(|&&x| x == l).count();
                assert!(n * 3 >= r.max() as usize - 2 && n * 3 <= r.max() as usize + 3, "range {} level {l}: {n}", r.max());
            }
        }
        assert_eq!(Range::from_index(0).level(15, 1), 0);
    }
}
