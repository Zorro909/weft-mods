//! What a grinding mill needs for a grind, over plain values: water by the millibucket, sand by
//! the grind, raw ore that some loaded mod has, and room for it.

use super::{Grinder, Status};
use crate::rubble::grinding::{GRINDS_PER_SAND, WATER_PER_GRIND};

/// What a grinding mill has for its next grind, besides the sand it keeps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Supplies {
    /// Millibuckets of water.
    pub water: u32,
    /// Whether a loaded mod has the raw ore its Ore Rubble grinds into.
    pub raw_ore: bool,
    /// Whether its output has room for the most that grind may give.
    pub room: bool,
}

impl Grinder {
    /// Takes a block of sand's grinds when it has none left and `sand` waits in its slot;
    /// whether it took it.
    pub fn top_up(&mut self, sand: bool) -> bool {
        let takes = self.grit == 0 && sand;
        if takes {
            self.grit = GRINDS_PER_SAND;
        }
        takes
    }

    /// What it lacks for a grind with `supplies`; none when it may grind.
    pub fn lacks(&self, supplies: Supplies) -> Option<Status> {
        if !supplies.raw_ore {
            Some(Status::NoRawOre)
        } else if supplies.water < WATER_PER_GRIND {
            Some(Status::NoWater)
        } else if self.grit == 0 {
            Some(Status::NoSand)
        } else if !supplies.room {
            Some(Status::Full)
        } else {
            None
        }
    }

    /// Uses up a grind's sand and starts paying for the next.
    pub fn ground(&mut self) {
        self.grit = self.grit.saturating_sub(1);
        self.spent = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const READY: Supplies = Supplies {
        water: 1_000,
        raw_ore: true,
        room: true,
    };

    #[test]
    fn a_sand_lasts_four_grinds() {
        let mut grinder = Grinder::default();
        assert!(grinder.top_up(true), "the first sand");
        assert!(!grinder.top_up(true), "not while it has grinds left");
        for _ in 0..GRINDS_PER_SAND {
            assert_eq!(grinder.lacks(READY), None, "it grinds");
            grinder.ground();
        }
        assert_eq!(grinder.lacks(READY), Some(Status::NoSand), "then sand");
        assert!(!grinder.top_up(false), "none waiting");
        assert!(grinder.top_up(true), "the next sand");
    }

    #[test]
    fn it_says_what_it_lacks_first() {
        let grinder = Grinder {
            grit: 1,
            ..Grinder::default()
        };
        let no_raw_ore = Supplies {
            raw_ore: false,
            water: 0,
            ..READY
        };
        assert_eq!(
            grinder.lacks(no_raw_ore),
            Some(Status::NoRawOre),
            "a metal no mod has"
        );
        let dry = Supplies {
            water: WATER_PER_GRIND - 1,
            ..READY
        };
        assert_eq!(
            grinder.lacks(dry),
            Some(Status::NoWater),
            "short of a grind"
        );
        let full = Supplies {
            room: false,
            ..READY
        };
        assert_eq!(grinder.lacks(full), Some(Status::Full), "no room");
        let one_grind = Supplies {
            water: WATER_PER_GRIND,
            ..READY
        };
        assert_eq!(
            grinder.lacks(one_grind),
            None,
            "one grind's water is enough"
        );
    }

    #[test]
    fn a_grind_starts_the_next_one_over() {
        let mut grinder = Grinder {
            grit: 2,
            spent: 800,
            ..Grinder::default()
        };
        grinder.ground();
        assert_eq!((grinder.grit, grinder.spent), (1, 0), "a grind's sand used");
    }
}
