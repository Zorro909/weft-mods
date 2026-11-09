//! Putting items into and taking them out of any mod's machines, by the names of their slot
//! groups: `fuel` and `input` take, `output` gives, a chest's `chest` does both. An inventory
//! without any of these names is one big chest. An assembly bench takes what its pattern asks
//! for. Liquid groups are left alone both ways: these are item tubes.

use base_api::FUEL;
use weft_sdk::*;

use crate::lib::Pos;

/// Group names this file knows the meaning of.
const KNOWN: [&str; 4] = ["fuel", "input", "output", "chest"];

/// A run of slots: one group, or the whole inventory.
enum Span {
    Group(SlotGroup<'static>),
    All(Inventory),
}

impl Span {
    fn len(&self) -> u32 {
        match self {
            Span::Group(group) => group.len(),
            Span::All(inventory) => inventory.len(),
        }
    }
    fn get(&self, i: u32) -> ItemStack {
        match self {
            Span::Group(group) => group.get(i),
            Span::All(inventory) => inventory.get(i),
        }
    }
    fn set(&self, i: u32, stack: ItemStack) {
        match self {
            Span::Group(group) => group.set(i, stack),
            Span::All(inventory) => inventory.set(i, stack),
        }
    }
}

/// Whether the inventory names none of its groups the way machines do.
fn plain(inventory: &Inventory) -> bool {
    !KNOWN
        .iter()
        .any(|name| inventory.find_group(name).is_some())
}

/// The group `name` of `inventory`, if it has one for items.
fn item_group(inventory: &Inventory, name: &str) -> Option<Span> {
    items_only(Span::Group(inventory.find_group(name)?))
}

/// The whole of `inventory` as one big chest, if it is plain and holds items.
fn whole(inventory: Inventory) -> Option<Span> {
    if !plain(&inventory) {
        return None;
    }
    items_only(Span::All(inventory))
}

/// `span`, unless it is a liquid group, which item tubes neither fill nor empty.
fn items_only(span: Span) -> Option<Span> {
    for_items(&read(&span), |item| FLUID.on_item(item).is_some()).then_some(span)
}

/// Whether slots holding `slots` are for items: nothing but a liquid (`is_liquid`) goes into a
/// liquid group. The SDK doesn't say which groups those are, so an empty one looks like this.
fn for_items(slots: &[ItemStack], is_liquid: impl Fn(u32) -> bool) -> bool {
    !slots
        .iter()
        .any(|stack| !stack.is_empty() && is_liquid(stack.item))
}

/// The slots of `inventory` to use: the first of the groups `names` it has for items, else the whole
/// of a plain inventory.
fn slots(inventory: Inventory, names: &[&str]) -> Option<Span> {
    if inventory.is_empty() {
        return None;
    }
    let named = names.iter().find_map(|name| item_group(&inventory, name));
    named.or_else(|| whole(inventory))
}

/// The slots `stack` may go into: fuel into a fuel slot, anything into an input or a chest.
fn takes(inventory: Inventory, stack: ItemStack) -> Option<Span> {
    if FUEL.on_item(stack.item).is_some() {
        slots(inventory, &["fuel", "input", "chest"])
    } else {
        slots(inventory, &["input", "chest"])
    }
}

/// The slots items may be taken from: results, or a chest's contents.
fn gives(inventory: Inventory) -> Option<Span> {
    slots(inventory, &["output", "chest"])
}

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
    let mut slots = read(span);
    let left = fill_slots(&mut slots, stack, max_stack);
    if !simulate {
        write(span, &slots);
    }
    left
}

fn read(span: &Span) -> Vec<ItemStack> {
    (0..span.len()).map(|i| span.get(i)).collect()
}

/// The stacks in the slots of `group`, in order.
pub fn stacks(group: &SlotGroup) -> Vec<ItemStack> {
    (0..group.len()).map(|i| group.get(i)).collect()
}

/// Sets the slots of `span` that `slots` changed.
fn write(span: &Span, slots: &[ItemStack]) {
    for (i, &slot) in (0..).zip(slots) {
        if span.get(i) != slot {
            span.set(i, slot);
        }
    }
}

/// Puts `stack` into `slots`, onto stacks it joins first and then into empty ones, at most
/// `cap(item)` to a slot; returns what didn't fit.
fn fill_slots(slots: &mut [ItemStack], stack: ItemStack, cap: impl Fn(u32) -> u32) -> ItemStack {
    let mut left = stack;
    for into_empty in [false, true] {
        for slot in slots.iter_mut() {
            let joins = !slot.is_empty() && slot.stacks_with(&left);
            let takes = if into_empty { slot.is_empty() } else { joins };
            if takes && !left.is_empty() {
                left = top_up(slot, left, cap(left.item));
            }
        }
    }
    left
}

/// Puts as much of `stack` into `slot`, empty or holding the same, as brings it to `cap`;
/// returns the rest.
fn top_up(slot: &mut ItemStack, stack: ItemStack, cap: u32) -> ItemStack {
    let held = if slot.is_empty() { 0 } else { slot.count };
    let moved = cap.saturating_sub(held).min(stack.count);
    if moved == 0 {
        return stack;
    }
    *slot = ItemStack {
        count: held + moved,
        ..stack
    };
    stack.shrink(moved)
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
    let mut slots = read(&span);
    let all_fit = fill_all(&mut slots, stacks, max_stack);
    if all_fit {
        write(&span, &slots);
    }
    all_fit
}

/// Puts `stacks` into `slots` one after another; false when they don't all fit.
fn fill_all(slots: &mut [ItemStack], stacks: &[ItemStack], cap: impl Fn(u32) -> u32) -> bool {
    stacks
        .iter()
        .all(|&stack| fill_slots(slots, stack, &cap).is_empty())
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

#[cfg(test)]
mod tests {
    use super::*;

    const PLANK: u32 = 7;
    const BUCKET: u32 = 9;

    /// Planks stack to 64, buckets not at all.
    fn cap(item: u32) -> u32 {
        if item == BUCKET { 1 } else { 64 }
    }

    #[test]
    fn items_join_their_stacks_before_taking_empty_slots() {
        let mut slots = [ItemStack::EMPTY, ItemStack::new(PLANK, 60)];
        let left = fill_slots(&mut slots, ItemStack::new(PLANK, 10), cap);
        assert_eq!(slots[1].count, 64, "the stack filled up first");
        assert_eq!(
            slots[0],
            ItemStack::new(PLANK, 6),
            "the rest in the empty slot"
        );
        assert!(left.is_empty(), "all of it went in");
    }

    #[test]
    fn what_has_no_room_is_handed_back() {
        let mut slots = [ItemStack::new(BUCKET, 1), ItemStack::EMPTY];
        let left = fill_slots(&mut slots, ItemStack::new(BUCKET, 3), cap);
        assert_eq!(
            slots[1],
            ItemStack::new(BUCKET, 1),
            "one into the empty slot"
        );
        assert_eq!(left, ItemStack::new(BUCKET, 2), "two left over");
    }

    #[test]
    fn a_group_holding_a_liquid_is_no_place_for_items() {
        const WATER: u32 = 11;
        let is_liquid = |item| item == WATER;
        let tank = [ItemStack::new(WATER, 4000), ItemStack::EMPTY];
        assert!(!for_items(&tank, is_liquid), "water in it: a liquid group");
        let chest = [ItemStack::new(PLANK, 3), ItemStack::new(BUCKET, 1)];
        assert!(for_items(&chest, is_liquid), "items: an item group");
        assert!(
            for_items(&[ItemStack::EMPTY; 2], is_liquid),
            "empty: taken for items"
        );
    }

    #[test]
    fn stacks_that_each_fit_alone_may_not_fit_together() {
        let output = [ItemStack::new(PLANK, 1), ItemStack::EMPTY];
        let made = [ItemStack::new(BUCKET, 1), ItemStack::new(BUCKET, 1)];
        assert!(
            fill_all(&mut output.clone(), &made[..1], cap),
            "one bucket fits"
        );
        assert!(!fill_all(&mut output.clone(), &made, cap), "two don't");
    }
}
