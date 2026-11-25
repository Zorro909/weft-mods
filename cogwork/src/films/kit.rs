//! Keeps the film player's hotbar empty while the film runs so no stacks appear in the hotbar
//! row of open machine screens, restoring whatever the player held when the film ends.

use std::sync::{Mutex, MutexGuard, PoisonError};

use showcase_api::Set;
use weft_sdk::ItemStack;

/// The slots of the hotbar, which every screen shows under it.
const HOTBAR: u32 = 9;

struct Saved {
    player: u64,
    slots: [ItemStack; HOTBAR as usize],
}

static SAVED: Mutex<Vec<Saved>> = Mutex::new(Vec::new());

fn saved() -> MutexGuard<'static, Vec<Saved>> {
    SAVED.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Saves and empties every hotbar slot so machine screens show a clean, empty hotbar.
pub fn pack(set: &Set) {
    let player = set.player();
    let inventory = set.world().inventory(player);
    let mut all = saved();
    if !all.iter().any(|entry| entry.player == player) {
        let slots = std::array::from_fn(|slot| inventory.get(slot as u32));
        all.push(Saved { player, slots });
    }
    for slot in 0..HOTBAR {
        inventory.set(slot, ItemStack::EMPTY);
    }
}

/// Restores the hotbar slots saved by `pack`.
pub fn unpack(set: &Set) {
    let player = set.player();
    let mut all = saved();
    let Some(index) = all.iter().position(|entry| entry.player == player) else {
        return;
    };
    let entry = all.swap_remove(index);
    let inventory = set.world().inventory(player);
    for (slot, stack) in entry.slots.into_iter().enumerate() {
        inventory.set(slot as u32, stack);
    }
}
