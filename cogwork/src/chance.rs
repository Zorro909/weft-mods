//! Chances, for what comes out of a machine: a fraction of an item is that chance of one more,
//! so 1.5 raw ore is one, and a second every other time.

use weft_sdk::World;

/// A number from 0 up to 1, never 1, from the world's random numbers.
pub fn roll(world: &World) -> f64 {
    // The top 53 bits: as many as an `f64` holds exactly.
    (world.random() >> 11) as f64 / (1u64 << 53) as f64
}

/// `amount` as a whole count, with `roll` from 0 to 1: its whole part, and one more when `roll`
/// falls inside its fraction.
pub fn count(amount: f64, roll: f64) -> u32 {
    let whole = amount.floor();
    whole as u32 + u32::from(roll < amount - whole)
}

/// Whether something with `chance` (0 to 1) happens, with `roll` from 0 to 1.
pub fn happens(chance: f64, roll: f64) -> bool {
    roll < chance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fraction_is_a_chance_of_one_more() {
        assert_eq!(count(1.5, 0.2), 2, "inside the half");
        assert_eq!(count(1.5, 0.7), 1, "outside it");
        assert_eq!(count(2.0, 0.0), 2, "a whole number is always that");
        assert_eq!(count(1.25, 0.9999), 1, "a high roll misses a quarter");
    }

    #[test]
    fn a_chance_happens_below_its_share() {
        assert!(happens(0.05, 0.04), "inside the chance");
        assert!(!happens(0.05, 0.05), "on its edge, not");
        assert!(!happens(0.0, 0.0), "never with none");
    }
}
