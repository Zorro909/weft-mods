//! Which slots of an inventory items go into or come out of: one of its groups by name, or the
//! whole of a plain inventory. Liquid groups are no place for items.

use base_api::FUEL;
use weft_sdk::*;

/// Group names this file knows the meaning of.
const KNOWN: [&str; 4] = ["fuel", "input", "output", "chest"];

/// A run of slots: one group, or the whole inventory.
pub enum Span {
    Group(SlotGroup<'static>),
    All(Inventory),
}

impl Span {
    pub fn len(&self) -> u32 {
        match self {
            Span::Group(group) => group.len(),
            Span::All(inventory) => inventory.len(),
        }
    }

    pub fn get(&self, i: u32) -> ItemStack {
        match self {
            Span::Group(group) => group.get(i),
            Span::All(inventory) => inventory.get(i),
        }
    }

    pub fn set(&self, i: u32, stack: ItemStack) {
        match self {
            Span::Group(group) => group.set(i, stack),
            Span::All(inventory) => inventory.set(i, stack),
        }
    }

    /// The stacks in its slots, in order.
    pub fn read(&self) -> Vec<ItemStack> {
        (0..self.len()).map(|i| self.get(i)).collect()
    }

    /// Sets the slots that `slots` changed.
    pub fn write(&self, slots: &[ItemStack]) {
        for (i, &slot) in (0..).zip(slots) {
            if self.get(i) != slot {
                self.set(i, slot);
            }
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
    for_items(&span.read(), |item| FLUID.on_item(item).is_some()).then_some(span)
}

/// Whether slots holding `slots` are for items: nothing but a liquid (`is_liquid`) goes into a
/// liquid group. The SDK doesn't say which groups those are, so an empty one looks like this.
fn for_items(slots: &[ItemStack], is_liquid: impl Fn(u32) -> bool) -> bool {
    !slots
        .iter()
        .any(|stack| !stack.is_empty() && is_liquid(stack.item))
}

/// The slots of `inventory` to use: the first of the groups `names` it has for items, else the
/// whole of a plain inventory.
fn slots(inventory: Inventory, names: &[&str]) -> Option<Span> {
    if inventory.is_empty() {
        return None;
    }
    let named = names.iter().find_map(|name| item_group(&inventory, name));
    named.or_else(|| whole(inventory))
}

/// The slots `stack` may go into: fuel into a fuel slot, anything into an input or a chest.
pub fn takes(inventory: Inventory, stack: ItemStack) -> Option<Span> {
    if FUEL.on_item(stack.item).is_some() {
        slots(inventory, &["fuel", "input", "chest"])
    } else {
        slots(inventory, &["input", "chest"])
    }
}

/// The slots items may be taken from: results, or a chest's contents.
pub fn gives(inventory: Inventory) -> Option<Span> {
    slots(inventory, &["output", "chest"])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_group_holding_a_liquid_is_no_place_for_items() {
        const PLANK: u32 = 7;
        const BUCKET: u32 = 9;
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
}
