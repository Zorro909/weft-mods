//! What the camera carries while the film goes on: ember and spring engines, the engines the
//! works are built with, in its hotbar under every screen the film opens, where they stand on
//! their base plate as a placed engine does. They go only into slots the camera has free, and
//! only those slots are emptied when the film takes them back.

use std::sync::{Mutex, MutexGuard, PoisonError};

use showcase_api::Set;
use weft_sdk::{Item, ItemStack};

/// The engines and how many of each, from the last slot of the hotbar on.
const KIT: [(&str, u32); 2] = [("cogwork:ember_engine", 12), ("cogwork:spring_engine", 6)];
/// The slots of the hotbar, which every screen shows under it.
const HOTBAR: u32 = 9;

/// A slot of a camera's that the film filled.
struct Given {
    player: u64,
    slot: u32,
}

static GIVEN: Mutex<Vec<Given>> = Mutex::new(Vec::new());

fn given() -> MutexGuard<'static, Vec<Given>> {
    GIVEN.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Puts the engines into free slots of the camera's hotbar, the last first, and keeps the
/// selected one free: the camera's hand stays empty.
pub fn pack(set: &Set) {
    let world = set.world();
    let player = set.player();
    let inventory = world.inventory(player);
    let selected = world.selected_slot(player);
    let mut given = given();
    let mut free = (0..HOTBAR)
        .rev()
        .filter(|&slot| slot != selected && inventory.get(slot).is_empty());
    for (name, count) in KIT {
        let (Some(item), Some(slot)) = (Item::lookup(name), free.next()) else {
            continue;
        };
        inventory.set(slot, ItemStack::new(item, count));
        given.push(Given { player, slot });
    }
}

/// Empties the slots the film filled for the camera.
pub fn unpack(set: &Set) {
    let player = set.player();
    let inventory = set.world().inventory(player);
    given().retain(|given| {
        let ours = given.player == player;
        if ours {
            inventory.set(given.slot, ItemStack::EMPTY);
        }
        !ours
    });
}
