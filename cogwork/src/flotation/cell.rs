//! A flotation cell's job, with no world: what one float of an Ore Rubble takes and leaves, what
//! keeps it from floating, and what grade it gives. A float goes ahead only with all of it there
//! and room for all it leaves, so nothing it takes is lost and nothing it makes is spilt.

use super::Status;
use crate::rubble::Grade;

/// Millibuckets of water and of creosote one float takes, and of muddy water it leaves.
pub const WATER_PER_FLOAT: u32 = 500;
pub const CREOSOTE_PER_FLOAT: u32 = 50;
pub const MUDDY_PER_FLOAT: u32 = 250;
/// Millibuckets of creosote and of muddy water the cell keeps beside its water.
pub const CREOSOTE_HELD: u32 = 500;
pub const MUDDY_HELD: u32 = 1_000;

/// What the cell holds of each liquid, in millibuckets.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Tanks {
    pub water: u32,
    pub creosote: u32,
    pub muddy: u32,
}

impl Tanks {
    /// What keeps it from floating, `room` saying whether the Rich Ore Rubble fits in the
    /// output; none when it may float.
    pub fn lacks(&self, room: bool) -> Option<Status> {
        if self.water < WATER_PER_FLOAT {
            Some(Status::NoWater)
        } else if self.creosote < CREOSOTE_PER_FLOAT {
            Some(Status::NoCreosote)
        } else if self.muddy + MUDDY_PER_FLOAT > MUDDY_HELD {
            Some(Status::Muddy)
        } else if !room {
            Some(Status::Full)
        } else {
            None
        }
    }

    /// The tanks after a float, which `lacks` allowed.
    pub fn floated(self) -> Tanks {
        Tanks {
            water: self.water - WATER_PER_FLOAT,
            creosote: self.creosote - CREOSOTE_PER_FLOAT,
            muddy: self.muddy + MUDDY_PER_FLOAT,
        }
    }
}

/// The grade floating Ore Rubble of `grade` gives: Common floats into Rich, and any other
/// passes through, Poor to be washed first and Rich or Pure as floated already.
pub fn floated(grade: Grade) -> Option<Grade> {
    (grade == Grade::Common).then_some(Grade::Rich)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tanks with enough of everything for one float.
    fn ready() -> Tanks {
        Tanks {
            water: WATER_PER_FLOAT,
            creosote: CREOSOTE_PER_FLOAT,
            muddy: MUDDY_HELD - MUDDY_PER_FLOAT,
        }
    }

    #[test]
    fn only_common_ore_rubble_is_floated() {
        assert_eq!(floated(Grade::Common), Some(Grade::Rich), "common to rich");
        for grade in [Grade::Poor, Grade::Rich, Grade::Pure] {
            assert_eq!(floated(grade), None, "{grade:?} passes through");
        }
    }

    #[test]
    fn it_says_what_it_lacks_first() {
        let empty = Tanks::default();
        assert_eq!(empty.lacks(false), Some(Status::NoWater), "water first");
        let no_creosote = Tanks {
            creosote: CREOSOTE_PER_FLOAT - 1,
            ..ready()
        };
        assert_eq!(no_creosote.lacks(true), Some(Status::NoCreosote));
        let muddy = Tanks {
            muddy: MUDDY_HELD - MUDDY_PER_FLOAT + 1,
            ..ready()
        };
        assert_eq!(muddy.lacks(true), Some(Status::Muddy), "nowhere for it");
        assert_eq!(ready().lacks(false), Some(Status::Full), "then room");
        assert_eq!(ready().lacks(true), None, "just enough of each");
    }

    #[test]
    fn a_float_takes_and_leaves_exactly_its_share() {
        let after = ready().floated();
        assert_eq!(
            after,
            Tanks {
                water: 0,
                creosote: 0,
                muddy: MUDDY_HELD
            },
            "to the millibucket"
        );
        assert_eq!(
            after.lacks(true),
            Some(Status::NoWater),
            "and can't go again"
        );
    }

    #[test]
    fn it_holds_four_floats_of_water_and_ten_of_creosote() {
        assert_eq!(
            super::super::WATER_CAPACITY / WATER_PER_FLOAT,
            4,
            "two buckets"
        );
        assert_eq!(CREOSOTE_HELD / CREOSOTE_PER_FLOAT, 10, "half a bucket");
        assert_eq!(MUDDY_HELD / MUDDY_PER_FLOAT, 4, "as many as its water");
    }
}
