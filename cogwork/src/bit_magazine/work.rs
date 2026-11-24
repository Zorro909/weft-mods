//! Every Bit Magazine's tick: find the quarry its feed points into, swap a bit if that quarry
//! needs one (`swap`), and show it: the drum turning a quarter and a clack at each swap.

use weft_sdk::{ItemStack, Output, SlotGroup, SystemBatch, World};

use super::swap::{Seen, Swap, plan};
use super::{BIT_MAGAZINE, DRUM, MAGAZINE, Magazine, Status, uses_left};
use crate::excavator::bit;
use crate::excavator::kind::Kind;
use crate::lib::{Pos, TypedWorld};

/// The drum's clip, a quarter turn.
const SWAP: &str = "swap";

pub fn tick(batch: &mut SystemBatch<Magazine>) {
    let world = batch.world();
    for (id, magazine, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&BIT_MAGAZINE) {
            continue;
        }
        let front = crate::machine::front(&world, pos);
        let (status, swapped) = serve(&world, id, pos.toward(front));
        magazine.set_status(status);
        let yaw = crate::owned::yaw(front);
        magazine.drum =
            crate::owned::place(&world, id, magazine.drum, &DRUM, pos.floor_middle(), yaw);
        if swapped {
            world.action_play(magazine.drum, SWAP, false);
            world.play_sound(&crate::sounds::BIT_SWAP, pos.center(), 1.0);
        }
    }
}

pub fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &MAGAZINE)
        .is_some_and(|magazine| magazine.drum == id)
}

/// Swaps a bit of the magazine `id` into the quarry at `at` if it needs one; the status, and
/// whether it swapped.
fn serve(world: &World, id: u64, at: Pos) -> (Status, bool) {
    let Some(kind) = Kind::of(world.block_typed(at)) else {
        return (Status::NoQuarry, false);
    };
    let quarry = world.block_entity(at);
    let Some(slot) = world.inventory(quarry).find_group(bit::GROUP) else {
        return (Status::NoQuarry, false);
    };
    let inventory = world.inventory(id);
    let bits = inventory.group("bits");
    let output = || inventory.group("output");
    let fitted = slot.get(0);
    let seen = |stack: ItemStack| seen(kind, stack);
    let fresh: Vec<Option<Seen>> = crate::store::stacks(&bits).into_iter().map(seen).collect();
    let room = crate::store::insert_into(output(), fitted, true).is_empty();
    let (status, swap) = plan(seen(fitted), &fresh, room);
    let Some(swap) = swap else {
        return (status, false);
    };
    let from = match swap {
        Swap::Fit(from) => from,
        Swap::Replace(from) => {
            crate::store::insert_into(output(), fitted, false);
            from
        }
    };
    fit(&slot, &bits, from);
    (status, true)
}

/// Moves the bit in `from` of the magazine's `bits` into the quarry's bit `slot`.
fn fit(slot: &SlotGroup, bits: &SlotGroup, from: u32) {
    slot.set(0, bits.get(from));
    bits.set(from, ItemStack::EMPTY);
}

/// `stack` as the magazine sees it beside a quarry of `kind`: none for an empty slot; anything
/// but a bit the quarry takes doesn't fit.
fn seen(kind: &Kind, stack: ItemStack) -> Option<Seen> {
    if stack.is_empty() {
        return None;
    }
    let Some(bit) = bit::of(stack) else {
        return Some(Seen {
            fits: false,
            left: 0,
        });
    };
    Some(Seen {
        fits: kind.takes_bit(bit),
        left: uses_left(bit, stack),
    })
}
