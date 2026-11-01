//! Randomness for a plan: the same numbers every time for the same seed, structure and region,
//! so every cube and every client plans alike.

use crate::turn::Turn;

pub struct Random {
    state: u64,
}

impl Random {
    pub(crate) fn new(seed: u64) -> Random {
        Random { state: seed }
    }

    /// The next 64 random bits.
    pub fn next(&mut self) -> u64 {
        // SplitMix64.
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// 0 up to `n - 1`; 0 when `n` is 0.
    pub fn below(&mut self, n: u64) -> u64 {
        ((self.next() as u128 * n as u128) >> 64) as u64
    }

    /// `from` up to `to`, both included; `from` when `to` is smaller.
    pub fn between(&mut self, from: i32, to: i32) -> i32 {
        let span = (to as i64 - from as i64 + 1).max(1) as u64;
        (from as i64 + self.below(span) as i64) as i32
    }

    /// True with probability `chance`, 0 to 1.
    pub fn chance(&mut self, chance: f32) -> bool {
        let threshold = (chance.clamp(0.0, 1.0) * (1u32 << 24) as f32) as u64;
        self.next() >> 40 < threshold
    }

    pub fn turn(&mut self) -> Turn {
        Turn::ALL[self.below(4) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_gives_the_same_numbers() {
        let (mut a, mut b) = (Random::new(42), Random::new(42));
        assert_eq!(
            [a.next(), a.next()],
            [b.next(), b.next()],
            "pure in the seed"
        );
    }

    #[test]
    fn between_stays_inside() {
        let mut random = Random::new(7);
        let values: Vec<i32> = (0..1000).map(|_| random.between(-3, 3)).collect();
        assert!(values.iter().all(|v| (-3..=3).contains(v)), "in range");
        assert!(
            values.contains(&-3) && values.contains(&3),
            "both ends reached"
        );
    }

    #[test]
    fn chance_holds_on_average() {
        let mut random = Random::new(9);
        let hits = (0..10_000).filter(|_| random.chance(0.25)).count();
        assert!((2300..2700).contains(&hits), "about a quarter, got {hits}");
    }
}
