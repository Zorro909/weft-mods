//! A polymerizer's tick: hand on its pellets by its front, then, with a batch's gas, a catalyst
//! and room for the pellets, put energy into the batch until it is paid for. The gas leaves the
//! vessel, the catalyst wears and the pellets go into the output together, once the batch is
//! done. One step a call: missed ticks make nothing.

use weft_sdk::{Output, StackData, SystemBatch, Vec3, World};

use super::{POLYMERIZER, Polymerizing, Status};
use crate::lib::{Pos, TypedWorld};
use crate::liquid::HELD;
use crate::machine::{self, Payment};
use crate::plastics::{CATALYST, PLASTIC_PELLET};
use crate::refinery::products::Product;

/// Millibuckets of gas a batch takes, and the pellets it makes.
pub const GAS_A_BATCH: u32 = 250;
const PELLETS_A_BATCH: u32 = 4;
/// The most energy it puts into a batch a tick, and the energy a batch takes: 100 ticks.
const DRAW: i64 = 200;
const BATCH_COST: i64 = 20_000;
/// Ticks between bubbles, and between the sounds of it, while it works.
const BUBBLE_EVERY: u64 = 10;
const SOUND_EVERY: u64 = 40;
/// Where its vent lets out a wisp, blocks above its floor.
const VENT_UP: f64 = 1.05;

pub fn tick(batch: &mut SystemBatch<Polymerizing>) {
    let world = batch.world();
    for (id, state, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&POLYMERIZER) {
            continue;
        }
        machine::start_meter(&world, id);
        let front = machine::front(&world, pos);
        crate::store::eject_toward(&world, pos, &[front]);
        let status = work(&world, id, state);
        state.set_status(status);
        if status == Status::Working {
            show_working(&world, pos);
        }
    }
}

/// What keeps a polymerizer from working with `gas` millibuckets in it, whether it has a
/// `catalyst`, and whether a batch's pellets have `room`; none when it may work.
fn lacks(gas: u32, catalyst: bool, room: bool) -> Option<Status> {
    if gas < GAS_A_BATCH {
        Some(Status::NoGas)
    } else if !catalyst {
        Some(Status::NoCatalyst)
    } else if !room {
        Some(Status::Full)
    } else {
        None
    }
}

/// Works on the batch of the polymerizer `id` once it has what that takes, and finishes it once
/// it is paid for.
fn work(world: &World, id: u64, state: &mut Polymerizing) -> Status {
    let inventory = world.inventory(id);
    let catalysts = inventory.group("catalyst");
    let catalyst = (0..catalysts.len()).find(|&slot| catalysts.get(slot).item == CATALYST.id());
    let pellets = PLASTIC_PELLET.stack(PELLETS_A_BATCH);
    let room = crate::store::insert_into(inventory.group("output"), pellets, true).is_empty();
    if let Some(lack) = lacks(gas(world, id), catalyst.is_some(), room) {
        return lack;
    }
    match machine::pay(world, id, &mut state.spent, BATCH_COST, DRAW) {
        Payment::Paid => {}
        Payment::Paying => return Status::Working,
        Payment::NoPower => return Status::NoPower,
    }
    crate::store::insert_into(inventory.group("output"), pellets, false);
    if let Some(held) = world.component_mut(id, &HELD) {
        held.remove(GAS_A_BATCH);
    }
    if let Some(slot) = catalyst {
        catalysts.set(slot, catalysts.get(slot).add_wear(1));
    }
    state.spent = 0;
    Status::Working
}

/// Millibuckets of gas the polymerizer `id` holds.
fn gas(world: &World, id: u64) -> u32 {
    let held = world.component(id, &HELD).unwrap_or_default();
    if held.liquid == Product::Gas.key() {
        held.amount
    } else {
        0
    }
}

/// A wisp from the vent of the polymerizer at `pos` now and then, and its bubbling.
fn show_working(world: &World, pos: Pos) {
    let now = world.game_time();
    if now.is_multiple_of(BUBBLE_EVERY) {
        let vent = pos.floor_middle() + Vec3::new(0.0, VENT_UP, 0.0);
        crate::steam::puff(world, vent, 1);
    }
    if now.is_multiple_of(SOUND_EVERY) {
        world.play_sound(&crate::sounds::POLYMERIZER, pos.center(), 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_batch_takes_a_hundred_ticks_at_full_power() {
        assert_eq!(BATCH_COST / DRAW, 100, "200 a tick for 100 ticks");
    }

    #[test]
    fn it_says_what_it_lacks_first() {
        assert_eq!(lacks(0, false, false), Some(Status::NoGas), "gas first");
        assert_eq!(
            lacks(GAS_A_BATCH - 1, true, true),
            Some(Status::NoGas),
            "short of a batch"
        );
        assert_eq!(
            lacks(GAS_A_BATCH, false, true),
            Some(Status::NoCatalyst),
            "then a catalyst"
        );
        assert_eq!(lacks(GAS_A_BATCH, true, false), Some(Status::Full), "room");
        assert_eq!(lacks(GAS_A_BATCH, true, true), None, "a batch's worth");
    }
}
