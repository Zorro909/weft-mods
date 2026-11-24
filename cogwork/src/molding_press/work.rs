//! Every molding press's tick: hand on its parts by its front, then, with a mold, what the mold
//! presses and room for its part, put energy into the press until it is paid for. The input is
//! taken, the mold wears and the part goes into the output together, as the ram comes down. One
//! step a call: missed ticks press nothing.

use weft_sdk::{ItemStack, StackData, SystemBatch, World};

use super::molds::{Shape, shape_of};
use super::{MOLDING_PRESS, Pressing, Status, ram};
use crate::lib::{Pos, TypedWorld};
use crate::machine::{self, Payment};

/// The most energy it puts into a press a tick, and the energy a press takes: 10 ticks.
const DRAW: i64 = 40;
const PRESS_COST: i64 = 400;

pub fn tick(batch: &mut SystemBatch<Pressing>) {
    let world = batch.world();
    for (id, state, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&MOLDING_PRESS) {
            continue;
        }
        machine::start_meter(&world, id);
        let front = machine::front(&world, pos);
        crate::store::eject_toward(&world, pos, &[front]);
        let (status, pressed) = work(&world, id, state);
        state.set_status(status);
        ram::show(&world, id, pos, front, state, pressed);
    }
}

/// Works on a press of the molding press `id`; its status, and whether it pressed a part now.
fn work(world: &World, id: u64, state: &mut Pressing) -> (Status, bool) {
    let inventory = world.inventory(id);
    let mold = inventory.group("mold");
    let Some(shape) = shape_of(mold.get(0)) else {
        return (Status::NoMold, false);
    };
    let input = inventory.group("input");
    let stacks = crate::store::stacks(&input);
    let Some(left) = pressed_from(&stacks, shape) else {
        return (Status::Idle, false);
    };
    let part = shape.makes.stack(1);
    if !crate::store::insert_into(inventory.group("output"), part, true).is_empty() {
        return (Status::Full, false);
    }
    match machine::pay(world, id, &mut state.spent, PRESS_COST, DRAW) {
        Payment::Paid => {}
        Payment::Paying => return (Status::Pressing, false),
        Payment::NoPower => return (Status::NoPower, false),
    }
    crate::store::insert_into(inventory.group("output"), part, false);
    for (slot, (&before, &after)) in (0..).zip(stacks.iter().zip(&left)) {
        if before != after {
            input.set(slot, after);
        }
    }
    mold.set(0, mold.get(0).add_wear(1));
    state.spent = 0;
    (Status::Pressing, true)
}

/// The input `stacks` once a press of `shape` has taken what it takes; none when it isn't all
/// there.
fn pressed_from(stacks: &[ItemStack], shape: &Shape) -> Option<Vec<ItemStack>> {
    crate::excavator::taken(stacks, shape.takes, |cost, item| cost.material.is(item))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_takes_ten_ticks_at_full_power() {
        assert_eq!(PRESS_COST / DRAW, 10, "400 at 40 a tick");
    }
}
