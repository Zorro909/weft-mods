//! A shaker screen's tick: hand on what it sifted out by its front, then put energy into the
//! first Rubble or slag of its input until that is paid for and sifted, the tray shaking
//! meanwhile. What a paid-for load turns up is rolled once and kept in `Shaker::sifted` until it
//! all fits in the output, so a full output never rolls it away.

use weft_sdk::{StackData, SystemBatch, World};

use super::{SHAKER_SCREEN, Shaker, Status, tray};
use crate::lib::{Pos, TypedWorld};
use crate::machine::{self, Payment};
use crate::rubble::sifting::{self, Pile, Sifted};

/// The most energy it puts into sifting a tick, and the ticks one Rubble takes at that.
const DRAW: i64 = 20;
const SIFT_TICKS: i64 = 40;
/// Energy sifting one Rubble takes.
const SIFT_COST: i64 = DRAW * SIFT_TICKS;
/// The bits of `Shaker::sifted` set once its load is rolled, and when that load is slag; the
/// others are a `Sifted`.
const ROLLED: u32 = 1 << 31;
const SLAG_ROLLED: u32 = 1 << 30;

pub fn tick(batch: &mut SystemBatch<Shaker>) {
    let world = batch.world();
    for (id, shaker, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&SHAKER_SCREEN) {
            continue;
        }
        machine::start_meter(&world, id);
        let front = machine::front(&world, pos);
        crate::store::eject_toward(&world, pos, &[front]);
        let status = sift(&world, id, shaker);
        shaker.set_status(status);
        tray::show(&world, id, pos, front, shaker);
    }
}

/// Puts this tick's energy into the first load in the input of the shaker screen `id`, and
/// sifts it into the output once it is paid for and what it gives has room there.
fn sift(world: &World, id: u64, shaker: &mut Shaker) -> Status {
    let input = world.inventory(id).group("input");
    let piles = (0..input.len()).map(|slot| Pile::of(input.get(slot).item));
    let Some((slot, pile)) = shaker.next_load(piles) else {
        return Status::Idle;
    };
    let payment = machine::pay(world, id, &mut shaker.spent, SIFT_COST, DRAW);
    if let Some(status) = still_paying(payment) {
        return status;
    }
    let finds = shaker.finds(pile, || sifting::roll(world, pile));
    if !crate::store::insert_all(
        world.inventory(id).group("output"),
        &finds.stacks(pile.table()),
    ) {
        // Paid for and rolled already: both wait until there is room.
        return Status::Full;
    }
    input.set(slot, input.get(slot).shrink(1));
    shaker.sifted_out();
    Status::Sifting
}

impl Shaker {
    /// The slot and pile of the load it sifts next, of the `piles` in its input slots: the one it
    /// rolled for already if that is still there, else the first. With none it has nothing to
    /// sift.
    fn next_load(&self, piles: impl Iterator<Item = Option<Pile>>) -> Option<(u32, Pile)> {
        let loads: Vec<(u32, Pile)> = (0..)
            .zip(piles)
            .filter_map(|(slot, pile)| Some((slot, pile?)))
            .collect();
        let rolled = self.rolled();
        let pending = loads.iter().find(|&&(_, pile)| Some(pile) == rolled);
        pending.or(loads.first()).copied()
    }

    /// The pile its kept roll is for, if it has one.
    fn rolled(&self) -> Option<Pile> {
        if self.sifted & ROLLED == 0 {
            None
        } else if self.sifted & SLAG_ROLLED == 0 {
            Some(Pile::Rubble)
        } else {
            Some(Pile::Slag)
        }
    }

    /// What its paid-for load of `pile` turns up: rolled by `roll` the first time, then the same
    /// until it is `sifted_out`. A roll kept for another pile, whose load was taken out, is
    /// dropped.
    fn finds(&mut self, pile: Pile, roll: impl FnOnce() -> Sifted) -> Sifted {
        if self.rolled() != Some(pile) {
            let kind = if pile == Pile::Slag { SLAG_ROLLED } else { 0 };
            self.sifted = ROLLED | kind | roll().0;
        }
        Sifted(self.sifted & !(ROLLED | SLAG_ROLLED))
    }

    /// Its load sifted into the output: the next starts unpaid and unrolled.
    fn sifted_out(&mut self) {
        (self.spent, self.sifted) = (0, 0);
    }
}

/// What a shaker screen shows while its Rubble is not paid for yet; none once it is.
fn still_paying(payment: Payment) -> Option<Status> {
    match payment {
        Payment::Paid => None,
        Payment::Paying => Some(Status::Sifting),
        Payment::NoPower => Some(Status::NoPower),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rubble::sifting::Find;

    #[test]
    fn a_rubble_takes_two_seconds_at_full_power() {
        assert_eq!(SIFT_COST, 800, "800 energy");
        assert_eq!(SIFT_COST / DRAW, 40, "40 ticks of 20");
    }

    #[test]
    fn a_find_waits_for_room_instead_of_being_rolled_again() {
        let diamond = Sifted(1 << 8);
        let rubble = Pile::Rubble.table();
        assert_eq!(
            diamond.finds(rubble),
            [Find::Item("base:diamond")],
            "the last row"
        );
        let mut shaker = Shaker::default();
        assert_eq!(
            shaker.finds(Pile::Rubble, || diamond),
            diamond,
            "the first roll"
        );
        // The output was full, so it wasn't sifted out; the next tick rolls nothing.
        let again = shaker.finds(Pile::Rubble, Sifted::default);
        assert_eq!(again, diamond, "the diamond kept");
        shaker.sifted_out();
        let empty = Sifted::default();
        assert_eq!(
            shaker.finds(Pile::Rubble, || empty),
            empty,
            "the next rolls anew"
        );
        assert_eq!(
            shaker.finds(Pile::Rubble, || diamond),
            empty,
            "and keeps its roll too"
        );
    }

    #[test]
    fn a_kept_roll_is_sifted_from_its_own_pile() {
        let mut shaker = Shaker::default();
        let iron = Sifted(1 << 1);
        shaker.finds(Pile::Slag, || iron);
        let piles = [Some(Pile::Rubble), None, Some(Pile::Slag)];
        let next = shaker.next_load(piles.into_iter());
        assert_eq!(next, Some((2, Pile::Slag)), "the slag it rolled for");
        let gone = shaker.next_load([Some(Pile::Rubble)].into_iter());
        assert_eq!(gone, Some((0, Pile::Rubble)), "else the first load");
        let rolled = shaker.finds(Pile::Rubble, Sifted::default);
        assert_eq!(rolled, Sifted::default(), "the slag's roll dropped");
    }

    #[test]
    fn an_empty_input_has_nothing_to_sift() {
        let shaker = Shaker::default();
        assert_eq!(shaker.next_load([None, None].into_iter()), None);
    }

    #[test]
    fn it_sifts_once_paid_and_shakes_while_paying() {
        assert_eq!(still_paying(Payment::Paid), None, "paid: sift it");
        assert_eq!(
            still_paying(Payment::Paying),
            Some(Status::Sifting),
            "shaking"
        );
        assert_eq!(
            still_paying(Payment::NoPower),
            Some(Status::NoPower),
            "no energy"
        );
    }
}
