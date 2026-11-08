//! Randomness for a plan: the same numbers every time for the same seed, structure and region,
//! so every cube and every client plans alike.

use weft_sdk::noise;

use crate::turn::Turn;

/// SplitMix64's step between two states.
const GOLDEN_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;

pub struct Random {
    state: u64,
}

impl Random {
    pub(crate) fn new(seed: u64) -> Random {
        Random { state: seed }
    }

    /// The next 64 random bits (SplitMix64).
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> u64 {
        // `mix` steps once more itself before mixing.
        let bits = noise::mix(self.state);
        self.state = self.state.wrapping_add(GOLDEN_GAMMA);
        bits
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
    fn the_numbers_never_change() {
        // Plans of worlds already generated come from these: changing them moves structures.
        let mut random = Random::new(42);
        assert_eq!(random.next(), 0xbdd7_3226_2feb_6e95, "the first");
        assert_eq!(random.next(), 0x28ef_e333_b266_f103, "the second");
        assert_eq!(random.between(-5, 5), -2, "and what is made of them");
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
