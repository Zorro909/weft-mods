//! How a blast furnace smelts, with no world: an iron ingot and a coke take 400 ticks of a
//! driven bellows' draft to become a steel bloom and slag. The ingot and coke stay in their
//! slots until the bloom and slag have room in the output; heat already put in stays when the
//! draft stops. It smelts only the ticks it gets draft for, so time missed while its area was
//! away counts only as far as a bellows paid for it.

use super::{Blast, Status};

/// Ticks of draft one bloom takes.
pub const SMELT_TICKS: u32 = 400;

impl Blast {
    /// Smelts for up to `ticks`, with `charges` ingots it has coke for. `draft` hands over up to
    /// the ticks of draft asked for, asked only while there is something to heat; `finish` puts
    /// each bloom away, false while its output has no room.
    pub fn smelt(
        &mut self,
        mut ticks: u32,
        mut charges: u32,
        mut draft: impl FnMut(u32) -> u32,
        mut finish: impl FnMut() -> bool,
    ) -> Status {
        loop {
            if charges == 0 {
                return Status::Idle;
            }
            if self.heat < SMELT_TICKS {
                let blown = draft(ticks.min(SMELT_TICKS - self.heat));
                if blown == 0 {
                    return Status::NoDraft;
                }
                self.heat += blown;
                ticks -= blown;
                if self.heat < SMELT_TICKS {
                    return Status::Smelting;
                }
            }
            if !finish() {
                return Status::Full;
            }
            self.heat = 0;
            charges -= 1;
            if ticks == 0 {
                return Status::Smelting;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Smelts with room for everything and all the draft it asks for while `blown`; the blooms
    /// made.
    fn smelted(blast: &mut Blast, ticks: u32, blown: bool, charges: u32) -> (Status, u32) {
        let mut draft = if blown { u32::MAX } else { 0 };
        smelted_with(blast, ticks, &mut draft, charges)
    }

    /// Smelts with room for everything, taking what it asks for out of `draft`; the blooms
    /// made.
    fn smelted_with(blast: &mut Blast, ticks: u32, draft: &mut u32, charges: u32) -> (Status, u32) {
        let mut blooms = 0;
        let take = |wanted: u32| {
            let taken = wanted.min(*draft);
            *draft -= taken;
            taken
        };
        let status = blast.smelt(ticks, charges, take, || {
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
        let status = blast.smelt(SMELT_TICKS, 1, |wanted| wanted, || false);
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

    #[test]
    fn it_smelts_only_the_ticks_it_gets_draft_for() {
        let mut blast = Blast::default();
        let mut draft = 3;
        let (status, blooms) = smelted_with(&mut blast, 1_200, &mut draft, 5);
        assert_eq!(
            (status, blooms),
            (Status::Smelting, 0),
            "three ticks of 1,200"
        );
        assert_eq!((blast.heat, draft), (3, 0), "each used once");
    }

    #[test]
    fn it_takes_no_draft_it_cannot_use() {
        let mut draft = 4;
        smelted_with(&mut Blast::default(), 1, &mut draft, 0);
        assert_eq!(draft, 4, "nothing to smelt");
        let mut blast = Blast {
            heat: SMELT_TICKS,
            ..Blast::default()
        };
        let status = blast.smelt(1, 1, |_| panic!("asked for draft while full"), || false);
        assert_eq!(status, Status::Full, "waiting for room");
        let mut draft = 4;
        smelted_with(&mut Blast::default(), 2, &mut draft, 1);
        assert_eq!(draft, 2, "only the ticks of the call");
    }
}
