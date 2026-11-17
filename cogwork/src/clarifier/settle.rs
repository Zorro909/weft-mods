//! How a clarifier settles muddy water, with no world: a lot of 10 mB every 2 ticks gives back
//! 9 mB of clear water, and each bucket settled has a one in ten chance of a clay ball. A lot
//! settles only with room for its water, and the lot that ends a bucket only with room for the
//! clay it may give, so no roll is ever thrown away. Time it waits is not saved up.

use super::{Settle, Status};
use crate::chance;

/// Millibuckets of muddy water a lot is, and of clear water it gives back.
pub const LOT: u32 = 10;
pub const CLEAR_PER_LOT: u32 = 9;
/// Ticks one lot takes to settle: 5 mB a tick.
pub const LOT_TICKS: u32 = 2;
/// Millibuckets settled between chances of clay, and that chance.
pub const BUCKET: u32 = 1_000;
pub const CLAY_CHANCE: f64 = 0.1;
/// Millibuckets of clear water it keeps until they leave by its front.
pub const CLEAR_HELD: u32 = 1_000;

/// What the clarifier holds: millibuckets of muddy water and of clear water.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Basin {
    pub muddy: u32,
    pub clear: u32,
}

impl Settle {
    /// Settles `basin` for `ticks`, ending with the bucket it finishes, if it does: `clay_room`
    /// says whether a clay ball fits in the output, and `roll` (`chance::roll`) decides whether
    /// the bucket gives one. The clay balls it gave, and what it is doing now.
    pub fn run(
        &mut self,
        ticks: u32,
        basin: &mut Basin,
        clay_room: bool,
        mut roll: impl FnMut() -> f64,
    ) -> (u32, Status) {
        self.elapsed = self.elapsed.saturating_add(ticks);
        loop {
            if let Some(lack) = self.lacks(basin, clay_room) {
                self.elapsed = 0;
                return (0, lack);
            }
            if self.elapsed < LOT_TICKS {
                return (0, Status::Settling);
            }
            self.elapsed -= LOT_TICKS;
            basin.muddy -= LOT;
            basin.clear += CLEAR_PER_LOT;
            self.settled += LOT;
            if self.settled >= BUCKET {
                self.settled = 0;
                self.elapsed = 0;
                let clay = u32::from(chance::happens(CLAY_CHANCE, roll()));
                return (clay, Status::Settling);
            }
        }
    }

    /// What keeps it from settling a lot of `basin`; none when it may.
    fn lacks(&self, basin: &Basin, clay_room: bool) -> Option<Status> {
        if basin.muddy < LOT {
            Some(Status::Idle)
        } else if basin.clear + CLEAR_PER_LOT > CLEAR_HELD {
            Some(Status::Full)
        } else if self.settled + LOT >= BUCKET && !clay_room {
            Some(Status::NoRoom)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A basin with `muddy` millibuckets of muddy water and no clear water.
    fn muddy(muddy: u32) -> Basin {
        Basin { muddy, clear: 0 }
    }

    #[test]
    fn a_lot_settles_every_two_ticks_into_nine_tenths_clear_water() {
        let (mut settle, mut basin) = (Settle::default(), muddy(100));
        let (clay, status) = settle.run(1, &mut basin, true, || 0.5);
        assert_eq!((clay, status), (0, Status::Settling), "half a lot's time");
        assert_eq!(basin, muddy(100), "nothing settled yet");
        settle.run(1, &mut basin, true, || 0.5);
        assert_eq!(
            basin,
            Basin {
                muddy: 90,
                clear: 9
            },
            "one lot"
        );
    }

    #[test]
    fn missed_time_settles_what_there_is_and_no_more() {
        let (mut settle, mut basin) = (Settle::default(), muddy(30));
        let (_, status) = settle.run(100, &mut basin, true, || 0.5);
        assert_eq!(
            basin,
            Basin {
                muddy: 0,
                clear: 27
            },
            "three lots"
        );
        assert_eq!(status, Status::Idle, "then it waits");
        assert_eq!(settle.elapsed, 0, "the rest of the time is not saved up");
    }

    #[test]
    fn a_bucket_rolls_for_clay_once_and_only_with_room() {
        let (mut settle, mut basin) = (Settle::default(), muddy(2_000));
        settle.settled = BUCKET - LOT;
        let (clay, status) = settle.run(2, &mut basin, false, || unreachable!());
        assert_eq!((clay, status), (0, Status::NoRoom), "waits for room");
        assert_eq!(basin.muddy, 2_000, "and keeps the lot");
        let (clay, _) = settle.run(2, &mut basin, true, || 0.05);
        assert_eq!(clay, 1, "a lucky bucket");
        assert_eq!(settle.settled, 0, "the next bucket starts");
        settle.settled = BUCKET - LOT;
        let (clay, _) = settle.run(2, &mut basin, true, || 0.5);
        assert_eq!(clay, 0, "an unlucky one");
    }

    #[test]
    fn a_long_gap_finishes_one_bucket_at_most() {
        let (mut settle, mut basin) = (Settle::default(), muddy(4_000));
        let mut rolls = 0;
        settle.run(10_000, &mut basin, true, || {
            rolls += 1;
            0.5
        });
        assert_eq!(rolls, 1, "one roll, with the room it had");
        assert_eq!(basin.muddy, 3_000, "one bucket");
    }

    #[test]
    fn it_waits_while_its_clear_water_has_nowhere_to_go() {
        let mut settle = Settle::default();
        let mut basin = Basin {
            muddy: 100,
            clear: CLEAR_HELD - CLEAR_PER_LOT + 1,
        };
        let (_, status) = settle.run(2, &mut basin, true, || 0.5);
        assert_eq!(status, Status::Full, "no room for a lot's water");
        assert_eq!(basin.muddy, 100, "none settled");
    }

    #[test]
    fn a_bucket_gives_back_nine_tenths() {
        assert_eq!(BUCKET / LOT * CLEAR_PER_LOT, 900, "900 mB of 1,000");
        assert_eq!(BUCKET / LOT * LOT_TICKS, 200, "in ten seconds");
    }
}
