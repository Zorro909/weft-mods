//! A shaker screen's tick: hand on what it sifted out by its front, then put energy into the
//! first Rubble of its input until that is paid for and sifted, the tray shaking meanwhile.
//! What a paid-for Rubble turns up is rolled once and kept in `Shaker::sifted` until it all
//! fits in the output, so a full output never rolls it away.

use weft_sdk::{StackData, SystemBatch, World};

use super::{RUBBLE, SHAKER_SCREEN, Shaker, Status, tray};
use crate::lib::{Pos, TypedWorld};
use crate::machine::{self, Payment};
use crate::rubble::sifting::{self, Sifted};

/// The most energy it puts into sifting a tick, and the ticks one Rubble takes at that.
const DRAW: i64 = 20;
const SIFT_TICKS: i64 = 40;
/// Energy sifting one Rubble takes.
const SIFT_COST: i64 = DRAW * SIFT_TICKS;
/// The bit of `Shaker::sifted` set once its Rubble is rolled; the others are a `Sifted`.
const ROLLED: u32 = 1 << 31;

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

/// Puts this tick's energy into the first Rubble in the input of the shaker screen `id`, and
/// sifts it into the output once it is paid for and what it gives has room there.
fn sift(world: &World, id: u64, shaker: &mut Shaker) -> Status {
    let input = world.inventory(id).group("input");
    let Some(slot) = (0..input.len()).find(|&i| input.get(i).item == RUBBLE.id()) else {
        return Status::Idle;
    };
    let payment = machine::pay(world, id, &mut shaker.spent, SIFT_COST, DRAW);
    if let Some(status) = still_paying(payment) {
        return status;
    }
    let finds = shaker.finds(|| sifting::roll_rubble(world)).stacks();
    if !crate::store::insert_all(world.inventory(id).group("output"), &finds) {
        // Paid for and rolled already: both wait until there is room.
        return Status::Full;
    }
    input.set(slot, input.get(slot).shrink(1));
    shaker.sifted_out();
    Status::Sifting
}

impl Shaker {
    /// What its paid-for Rubble turns up: rolled by `roll` the first time, then the same until
    /// it is `sifted_out`.
    fn finds(&mut self, roll: impl FnOnce() -> Sifted) -> Sifted {
        if self.sifted & ROLLED == 0 {
            self.sifted = ROLLED | roll().0;
        }
        Sifted(self.sifted & !ROLLED)
    }

    /// Its Rubble sifted into the output: the next starts unpaid and unrolled.
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
        assert_eq!(
            diamond.finds(),
            [Find::Item("base:diamond")],
            "the last row"
        );
        let mut shaker = Shaker::default();
        assert_eq!(shaker.finds(|| diamond), diamond, "the first roll");
        // The output was full, so it wasn't sifted out; the next tick rolls nothing.
        assert_eq!(shaker.finds(Sifted::default), diamond, "the diamond kept");
        shaker.sifted_out();
        let empty = Sifted::default();
        assert_eq!(shaker.finds(|| empty), empty, "the next Rubble rolls anew");
        assert_eq!(shaker.finds(|| diamond), empty, "and keeps its roll too");
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
