//! What a grinding mill needs for a grind, over plain values: water by the millibucket, sand by
//! the grind, raw ore that some loaded mod has, and room for it. A sand in its slot becomes
//! grinds only when a grind finishes and it has none left.

use super::{Grinder, Status};
use crate::rubble::grinding::{GRINDS_PER_SAND, WATER_PER_GRIND};

/// What a grinding mill has for its next grind, besides the sand it keeps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Supplies {
    /// Millibuckets of water.
    pub water: u32,
    /// Whether a sand waits in its sand slot.
    pub sand: bool,
    /// Whether a loaded mod has the raw ore its Ore Rubble grinds into.
    pub raw_ore: bool,
    /// Whether its output has room for the most that grind may give.
    pub room: bool,
}

impl Grinder {
    /// What it lacks for a grind with `supplies`; none when it may grind.
    pub fn lacks(&self, supplies: Supplies) -> Option<Status> {
        if !supplies.raw_ore {
            Some(Status::NoRawOre)
        } else if supplies.water < WATER_PER_GRIND {
            Some(Status::NoWater)
        } else if self.grit == 0 && !supplies.sand {
            Some(Status::NoSand)
        } else if !supplies.room {
            Some(Status::Full)
        } else {
            None
        }
    }

    /// Uses up a grind's sand and starts paying for the next. With no grinds left it takes the
    /// sand waiting in its slot, which `lacks` made sure of; whether it took it.
    pub fn ground(&mut self) -> bool {
        let takes = self.grit == 0;
        if takes {
            self.grit = GRINDS_PER_SAND;
        }
        self.grit -= 1;
        self.spent = 0;
        takes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const READY: Supplies = Supplies {
        water: 1_000,
        sand: true,
        raw_ore: true,
        room: true,
    };

    #[test]
    fn a_sand_lasts_four_grinds() {
        let mut grinder = Grinder::default();
        let took: Vec<bool> = (0..GRINDS_PER_SAND + 1)
            .map(|_| {
                assert_eq!(grinder.lacks(READY), None, "it grinds");
                grinder.ground()
            })
            .collect();
        assert_eq!(took, [true, false, false, false, true], "a sand every four");
    }

    #[test]
    fn sand_stays_in_its_slot_until_a_grind_finishes() {
        let grinder = Grinder::default();
        let dry = Supplies { water: 0, ..READY };
        assert_eq!(grinder.lacks(dry), Some(Status::NoWater), "a dry mill");
        let no_sand = Supplies {
            sand: false,
            ..READY
        };
        assert_eq!(grinder.lacks(no_sand), Some(Status::NoSand), "then sand");
        let worn = Grinder {
            grit: 1,
            ..Grinder::default()
        };
        assert_eq!(worn.lacks(no_sand), None, "a grind left of the last");
    }

    #[test]
    fn it_says_what_it_lacks_first() {
        let grinder = Grinder::default();
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
        assert!(!grinder.ground(), "no sand taken");
        assert_eq!((grinder.grit, grinder.spent), (1, 0), "a grind's sand used");
    }
}
