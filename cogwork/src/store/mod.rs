//! Putting items into and taking them out of any mod's machines, by the names of their slot
//! groups: `fuel` and `input` take, `output` gives, a chest's `chest` does both. An inventory
//! without any of these names is one big chest. An assembly bench takes what its pattern asks
//! for. Liquid groups are left alone both ways: these are item tubes.

mod fill;
mod span;

use fill::{fill_all, fill_slots};
use span::{Span, gives, takes};
use weft_sdk::*;

use crate::lib::Pos;

/// Puts `stack` into the machine `owner` (its entity, or its block) and returns what didn't
/// fit. With `simulate` nothing changes.
pub fn insert(
    world: &World,
    owner: impl Into<Holder>,
    stack: ItemStack,
    simulate: bool,
) -> ItemStack {
    let owner = owner.into();
    if let Some(left) = crate::bench::insert(world, owner, stack, simulate) {
        return left;
    }
    let Some(span) = takes(world.inventory(owner), stack) else {
        return stack;
    };
    fill_span(&span, stack, simulate)
}

fn fill_span(span: &Span, stack: ItemStack, simulate: bool) -> ItemStack {
    let mut slots = span.read();
    let left = fill_slots(&mut slots, stack, max_stack);
    if !simulate {
        span.write(&slots);
    }
    left
}

/// The stacks in the slots of `group`, in order.
pub fn stacks(group: &SlotGroup) -> Vec<ItemStack> {
    (0..group.len()).map(|i| group.get(i)).collect()
}

/// Puts `stack` into the slots of `group` and returns what didn't fit; with `simulate` nothing
/// changes.
pub fn insert_into(group: SlotGroup<'static>, stack: ItemStack, simulate: bool) -> ItemStack {
    fill_span(&Span::Group(group), stack, simulate)
}

/// Puts every one of `stacks` into `group`, or none of them when they don't all fit together;
/// returns whether they went in.
pub fn insert_all(group: SlotGroup<'static>, stacks: &[ItemStack]) -> bool {
    let span = Span::Group(group);
    let mut slots = span.read();
    let all_fit = fill_all(&mut slots, stacks, max_stack);
    if all_fit {
        span.write(&slots);
    }
    all_fit
}

/// Whether any of `stack` would go into `owner`.
pub fn accepts(world: &World, owner: impl Into<Holder>, stack: ItemStack) -> bool {
    insert(world, owner, stack, true).count < stack.count
}

/// Takes up to `max` items of the first stack the machine `owner` gives out.
pub fn extract(world: &World, owner: impl Into<Holder>, max: u32) -> ItemStack {
    let Some(span) = gives(world.inventory(owner)) else {
        return ItemStack::EMPTY;
    };
    let mut held = (0..span.len()).map(|i| (i, span.get(i)));
    let Some((i, have)) = held.find(|(_, have)| !have.is_empty()) else {
        return ItemStack::EMPTY;
    };
    let taken = have.count.min(max);
    span.set(i, have.shrink(taken));
    ItemStack {
        count: taken,
        ..have
    }
}

/// Whether the block at `pos` holds items at all.
pub fn has_inventory(world: &World, pos: Pos) -> bool {
    !world.inventory(pos).is_empty()
}

/// Hands one stack of the `output` of the machine at `pos` to a tube or machine beside it.
pub fn eject(world: &World, pos: Pos) {
    let out = world.inventory(pos).group("output");
    let Some(slot) = (0..out.len()).find(|&i| !out.get(i).is_empty()) else {
        return;
    };
    let mut stack = out.get(slot);
    for dir in Dir::ALL {
        let other = pos.toward(dir);
        stack = if crate::tube::is_tube(world, other) {
            crate::tube::inject(world, other, stack)
        } else {
            insert(world, other, stack, false)
        };
        if stack.is_empty() {
            break;
        }
    }
    out.set(slot, stack);
}
