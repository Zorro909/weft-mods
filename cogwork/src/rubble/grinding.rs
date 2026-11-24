//! What grinding one Ore Rubble takes and gives, the same in the millstone and the grinding
//! mill: a grind's water and sand, and its metal's raw ore by its grade.

use super::Grade;
use crate::chance;

/// Millibuckets of water one grind takes.
pub const WATER_PER_GRIND: u32 = 125;
/// Grinds one block of sand lasts.
pub const GRINDS_PER_SAND: u32 = 4;

/// Raw ore one Ore Rubble of `grade` grinds into, `roll` (0 to 1) deciding its fraction.
pub fn raw_ore(grade: Grade, roll: f64) -> u32 {
    chance::count(grade.raw_ore(), roll)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bucket_lasts_eight_grinds() {
        assert_eq!(1_000 / WATER_PER_GRIND, 8, "eight");
    }

    #[test]
    fn a_grind_gives_its_grades_raw_ore() {
        assert_eq!(raw_ore(Grade::Poor, 0.99), 1, "poor: always one");
        assert_eq!(raw_ore(Grade::Common, 0.2), 2, "common: the half came up");
        assert_eq!(raw_ore(Grade::Common, 0.7), 1, "and here it didn't");
        assert_eq!(raw_ore(Grade::Pure, 0.2), 3, "pure: two and a half");
    }
}
