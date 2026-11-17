//! An acid tank's batch, with no world: what one batch takes and makes, and what keeps it from
//! starting. A batch goes ahead only with its sulfur and water there and room for all the acid
//! it makes, so nothing it takes is lost and nothing it makes is spilt.

use super::Status;

/// Millibuckets of water one batch takes, and of acid it makes, from one sulfur.
pub const WATER_A_BATCH: u32 = 1_000;
pub const ACID_A_BATCH: u32 = 1_000;
/// Millibuckets of water and of acid the tank keeps: four batches of each.
pub const WATER_HELD: u32 = 4_000;
pub const ACID_HELD: u32 = 4_000;

/// What the tank holds of each liquid, in millibuckets.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Tanks {
    pub water: u32,
    pub acid: u32,
}

impl Tanks {
    /// What keeps it from brewing, `sulfur` saying whether a sulfur waits in its input; none when
    /// it may.
    pub fn lacks(&self, sulfur: bool) -> Option<Status> {
        if !sulfur {
            Some(Status::NoSulfur)
        } else if self.water < WATER_A_BATCH {
            Some(Status::NoWater)
        } else if self.acid + ACID_A_BATCH > ACID_HELD {
            Some(Status::Full)
        } else {
            None
        }
    }

    /// The tanks after a batch, which `lacks` allowed.
    pub fn brewed(self) -> Tanks {
        Tanks {
            water: self.water - WATER_A_BATCH,
            acid: self.acid + ACID_A_BATCH,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready() -> Tanks {
        Tanks {
            water: WATER_A_BATCH,
            acid: ACID_HELD - ACID_A_BATCH,
        }
    }

    #[test]
    fn it_says_what_it_lacks_first() {
        let empty = Tanks::default();
        assert_eq!(empty.lacks(false), Some(Status::NoSulfur), "sulfur first");
        assert_eq!(empty.lacks(true), Some(Status::NoWater), "then water");
        let full = Tanks {
            acid: ACID_HELD - ACID_A_BATCH + 1,
            ..ready()
        };
        assert_eq!(full.lacks(true), Some(Status::Full), "then room for acid");
        assert_eq!(ready().lacks(true), None, "just enough of each");
    }

    #[test]
    fn a_batch_turns_a_bucket_of_water_into_a_bucket_of_acid() {
        let after = ready().brewed();
        assert_eq!(
            after,
            Tanks {
                water: 0,
                acid: ACID_HELD
            },
            "to the millibucket"
        );
        assert_eq!(after.lacks(true), Some(Status::NoWater), "and stops");
    }
}
