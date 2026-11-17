//! How a blast furnace smelts, with no world: an iron ingot and a coke take 400 ticks of a
//! driven bellows' draft to become a steel bloom and slag. The ingot and coke stay in their
//! slots until the bloom and slag have room in the output; heat already put in stays when the
//! draft stops. Time missed while its area was away counts only when the bellows paid for it
//! and was blowing as the furnace caught up.

use super::{Blast, Status};

/// Ticks of draft one bloom takes.
pub const SMELT_TICKS: u32 = 400;

impl Blast {
    /// Smelts for `ticks`, `blown` saying whether a driven bellows blows into it and `charges`
    /// how many ingots it has coke for; `finish` puts each bloom away, false while its output has
    /// no room.
    pub fn smelt(
        &mut self,
        mut ticks: u32,
        blown: bool,
        mut charges: u32,
        mut finish: impl FnMut() -> bool,
    ) -> Status {
        loop {
            if charges == 0 {
                return Status::Idle;
            }
            if self.heat < SMELT_TICKS {
                if !blown {
                    return Status::NoDraft;
                }
                let step = ticks.min(SMELT_TICKS - self.heat);
                self.heat += step;
                ticks -= step;
                if self.heat < SMELT_TICKS {
                    return Status::Smelting;
                }
            }
            if !finish() {
                return Status::Full;
            }
            self.heat = 0;
            charges -= 1;
            if ticks == 0 || !blown {
                return Status::Smelting;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smelts with room for everything; the blooms made.
    fn smelted(blast: &mut Blast, ticks: u32, blown: bool, charges: u32) -> (Status, u32) {
        let mut blooms = 0;
        let status = blast.smelt(ticks, blown, charges, || {
            blooms += 1;
            true
        });
        (status, blooms)
    }

    #[test]
    fn a_bloom_takes_400_ticks_of_draft() {
        let mut blast = Blast::default();
        assert_eq!(
            smelted(&mut blast, SMELT_TICKS - 1, true, 1),
            (Status::Smelting, 0)
        );
        assert_eq!(
            smelted(&mut blast, 1, true, 1),
            (Status::Smelting, 1),
            "done"
        );
        assert_eq!(blast.heat, 0, "the next starts cold");
    }

    #[test]
    fn without_draft_it_waits_and_keeps_its_heat() {
        let mut blast = Blast::default();
        smelted(&mut blast, 100, true, 1);
        assert_eq!(
            smelted(&mut blast, 500, false, 1),
            (Status::NoDraft, 0),
            "no draft"
        );
        assert_eq!(blast.heat, 100, "kept");
    }

    #[test]
    fn missed_time_with_draft_smelts_bloom_after_bloom() {
        let mut blast = Blast::default();
        let (status, blooms) = smelted(&mut blast, 3 * SMELT_TICKS + 50, true, 5);
        assert_eq!(blooms, 3, "three in the time");
        assert_eq!(
            (status, blast.heat),
            (Status::Smelting, 50),
            "a fourth begun"
        );
        let (status, blooms) = smelted(&mut blast, 10 * SMELT_TICKS, true, 2);
        assert_eq!(
            (status, blooms),
            (Status::Idle, 2),
            "until the iron runs out"
        );
    }

    #[test]
    fn a_full_output_keeps_the_ingot_and_its_heat() {
        let mut blast = Blast::default();
        let status = blast.smelt(SMELT_TICKS, true, 1, || false);
        assert_eq!(status, Status::Full, "no room");
        assert_eq!(blast.heat, SMELT_TICKS, "smelted, waiting");
        assert_eq!(
            smelted(&mut blast, 1, false, 1).1,
            1,
            "out once there is room"
        );
    }

    #[test]
    fn nothing_to_smelt_is_idle() {
        let mut blast = Blast::default();
        assert_eq!(smelted(&mut blast, 100, true, 0), (Status::Idle, 0));
        assert_eq!(blast.heat, 0, "no heat for nothing");
    }
}
