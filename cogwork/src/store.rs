//! Putting items into and taking them out of any mod's machines, by the names of their slot
//! groups: `fuel` and `input` take, `output` gives, a chest's `chest` does both. An inventory
//! without any of these names is one big chest.

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
    if FUEL.on_item(stack.item).is_some() {
        if let Some(g) = inv.find_group("fuel") {
            return Some(Span::Group(g));
        }
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
    let Some(span) = takes(w.inventory(owner), stack) else {
        return stack;
    };
    fill_span(&span, stack, simulate)
}

fn fill_span(span: &Span, stack: ItemStack, simulate: bool) -> ItemStack {
    let mut left = stack.count;
    // Onto stacks of the same item first, then into empty slots.
    for into_empty in [false, true] {
        for i in 0..span.len() {
            if left > 0 {
                left -= fill(
                    span,
                    i,
                    ItemStack {
                        count: left,
                        ..stack
                    },
                    into_empty,
                    simulate,
                );
            }
        }
    }
    if left == 0 {
        ItemStack::EMPTY
    } else {
        ItemStack {
            count: left,
            ..stack
        }
    }
}

/// How many of `stack` slot `i` takes on this pass.
fn fill(span: &Span, i: u32, stack: ItemStack, into_empty: bool, simulate: bool) -> u32 {
    let have = span.get(i);
    let held = match (have.is_empty(), into_empty) {
        (true, true) => 0,
        (false, false) if have.stacks_with(&stack) => have.count,
        _ => return 0,
    };
    let moved = max_stack(stack.item).saturating_sub(held).min(stack.count);
    if moved > 0 && !simulate {
        span.set(
            i,
            ItemStack {
                count: held + moved,
                ..stack
            },
        );
    }
    moved
}

/// Puts `stack` into the slots of `group` and returns what didn't fit; with `simulate` nothing
/// changes.
pub fn insert_into(group: InventoryGroup, stack: ItemStack, simulate: bool) -> ItemStack {
    fill_span(&Span::Group(group), stack, simulate)
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
pub fn has_inventory(w: &World, pos: [i32; 3]) -> Option<u64> {
    let id = w.block_entity(pos);
    (id != 0 && w.inventory(id).size() > 0).then_some(id)
}
