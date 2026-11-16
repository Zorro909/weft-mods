//! What the camera carries while the film goes on: ember and spring engines, the engines the
//! works are built with, in its hotbar under every screen the film opens, where they stand on
//! their base plate as a placed engine does. They go only into slots the camera has free, and
//! the film takes back only stacks that are still exactly as it gave them.

use showcase_api::Set;
use weft_sdk::{Item, ItemStack};

/// The engines and how many of each, from the last slot of the hotbar on.
const KIT: [(&str, u32); 2] = [("cogwork:ember_engine", 12), ("cogwork:spring_engine", 6)];
/// The slots of the hotbar, which every screen shows under it.
const HOTBAR: u32 = 9;

/// The kit's stacks, as the film hands them out.
fn stacks() -> impl Iterator<Item = ItemStack> {
    KIT.into_iter()
        .filter_map(|(name, count)| Some(ItemStack::new(Item::lookup(name)?, count)))
}

/// Puts the engines into free slots of the camera's hotbar, the last first, and keeps the
/// selected one free: the camera's hand stays empty. A kit left over from a take cut short is
/// taken back first.
pub fn pack(set: &Set) {
    unpack(set);
    let world = set.world();
    let player = set.player();
    let inventory = world.inventory(player);
    let selected = world.selected_slot(player);
    let free = (0..HOTBAR)
        .rev()
        .filter(|&slot| slot != selected && inventory.get(slot).is_empty());
    for (slot, stack) in free.zip(stacks()) {
        inventory.set(slot, stack);
    }
}

/// Empties the hotbar slots that hold a stack of the kit.
pub fn unpack(set: &Set) {
    let inventory = set.world().inventory(set.player());
    let kit: Vec<ItemStack> = stacks().collect();
    for slot in 0..HOTBAR {
        if kit.contains(&inventory.get(slot)) {
            inventory.set(slot, ItemStack::EMPTY);
        }
    }
}
