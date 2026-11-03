//! Putting items into and taking them out of any mod's machines, by the names of their slot
//! groups: `fuel` and `input` take, `output` gives, a chest's `chest` does both. An inventory
//! without any of these names is one big chest. An assembly bench takes what its pattern asks
//! for.

use base_api::FUEL;
use weft_sdk::*;

/// Group names this file knows the meaning of.
const KNOWN: [&str; 4] = ["fuel", "input", "output", "chest"];

/// A run of slots: one group, or the whole inventory.
enum Span {
    Group(InventoryGroup),
    All(Inventory),
}

impl Span {
    fn len(&self) -> u32 {
        match self {
            Span::Group(g) => g.len(),
            Span::All(inv) => inv.size(),
        }
    }
    fn get(&self, i: u32) -> ItemStack {
        match self {
            Span::Group(g) => g.get(i),
            Span::All(inv) => inv.get(i),
        }
    }
    fn set(&self, i: u32, stack: ItemStack) {
        match self {
            Span::Group(g) => g.set(i, stack),
            Span::All(inv) => inv.set(i, stack),
        }
    }
}

/// Whether the inventory names none of its groups the way machines do.
fn plain(inv: &Inventory) -> bool {
    !KNOWN.iter().any(|name| inv.find_group(name).is_some())
}

/// The slots `stack` may go into: fuel into a fuel slot, anything into an input or a chest.
fn takes(inv: Inventory, stack: ItemStack) -> Option<Span> {
    if inv.size() == 0 {
        return None;
    }
    if FUEL.on_item(stack.item).is_some()
        && let Some(g) = inv.find_group("fuel")
    {
        return Some(Span::Group(g));
    }
    for name in ["input", "chest"] {
        if let Some(g) = inv.find_group(name) {
            return Some(Span::Group(g));
        }
    }
    plain(&inv).then_some(Span::All(inv))
}

/// The slots items may be taken from: results, or a chest's contents.
fn gives(inv: Inventory) -> Option<Span> {
    if inv.size() == 0 {
        return None;
    }
    for name in ["output", "chest"] {
        if let Some(g) = inv.find_group(name) {
            return Some(Span::Group(g));
        }
    }
    plain(&inv).then_some(Span::All(inv))
}

/// Puts `stack` into the machine `owner` and returns what didn't fit. With `simulate` nothing
/// changes.
pub fn insert(w: &World, owner: u64, stack: ItemStack, simulate: bool) -> ItemStack {
    if let Some(left) = crate::bench::insert(w, owner, stack, simulate) {
        return left;
    }
    let Some(span) = takes(w.inventory(owner), stack) else {
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
            let takes = match into_empty {
                true => slot.is_empty(),
                false => !slot.is_empty() && slot.stacks_with(&left),
            };
            if left.is_empty() || !takes {
                continue;
            }
            let held = if slot.is_empty() { 0 } else { slot.count };
            let moved = cap(left.item).saturating_sub(held).min(left.count);
            if moved > 0 {
                *slot = ItemStack {
                    count: held + moved,
                    ..left
                };
                left = left.shrink(moved);
            }
        }
    }
    left
}

/// Puts `stack` into the slots of `group` and returns what didn't fit; with `simulate` nothing
/// changes.
pub fn insert_into(group: InventoryGroup, stack: ItemStack, simulate: bool) -> ItemStack {
    fill_span(&Span::Group(group), stack, simulate)
}

/// Puts every one of `stacks` into `group`, or none of them when they don't all fit together;
/// returns whether they went in.
pub fn insert_all(group: InventoryGroup, stacks: &[ItemStack]) -> bool {
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
pub fn accepts(w: &World, owner: u64, stack: ItemStack) -> bool {
    insert(w, owner, stack, true).count < stack.count
}

/// Takes up to `max` items of the first stack the machine gives out.
pub fn extract(w: &World, owner: u64, max: u32) -> ItemStack {
    let Some(span) = gives(w.inventory(owner)) else {
        return ItemStack::EMPTY;
    };
    for i in 0..span.len() {
        let have = span.get(i);
        if have.is_empty() {
            continue;
        }
        let taken = have.count.min(max);
        span.set(i, have.shrink(taken));
        return ItemStack {
            count: taken,
            ..have
        };
    }
    ItemStack::EMPTY
}

/// Whether the block entity at `pos` holds items at all.
pub fn has_inventory(w: &World, pos: [i32; 3]) -> bool {
    let id = w.block_entity(pos);
    id != 0 && w.inventory(id).size() > 0
}

/// Hands one stack of the `output` of the machine `id` at `pos` to a tube or machine beside it.
pub fn eject(w: &World, id: u64, pos: [i32; 3]) {
    let out = w.inventory(id).group("output");
    let Some(slot) = (0..out.len()).find(|&i| !out.get(i).is_empty()) else {
        return;
    };
    let mut stack = out.get(slot);
    for face in 0..6 {
        let other = w.block_entity(crate::neighbor(pos, face));
        if other == 0 || other == id {
            continue;
        }
        stack = if w.component(other, &crate::tube::TUBE).is_some() {
            crate::tube::inject(w, other, stack)
        } else {
            insert(w, other, stack, false)
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
