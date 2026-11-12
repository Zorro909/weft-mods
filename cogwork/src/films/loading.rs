//! Keeping the works running with nobody near: an ultra cuboid loader on a stone mast at the
//! corner of the digger's pad, north of the hall's east end, and the cuboid lens, which the
//! camera is handed for a while to show what is kept: the loader's cubes in cyan, the working
//! machines' in orange.

use std::sync::{Mutex, MutexGuard, PoisonError};

use showcase_api::Set;
use weft_sdk::ItemStack;

use super::parts::{STONE_BRICKS, area};
use crate::lib::{BlockState, Pos};
use crate::loading::{CUBOID_LENS, ULTRA_CUBOID_LOADER};

/// The loader, on top of its mast. It keeps the 7 by 7 cubes of the layer it stands in: this
/// is the highest block of the layer the machines stand in, and from this cube the 7 by 7
/// reach from the seep's pump in the west to the second excavator in the east.
pub const LOADER: Pos = Pos::new(27, 3, -11);
/// The lowest block of the mast, on the pad.
const MAST_FOOT: i32 = 0;

pub fn build(set: &Set) {
    let mast = area(LOADER.with_y(MAST_FOOT), LOADER.add_y(-1));
    set.fill(mast, BlockState::of(&STONE_BRICKS));
    set.place(LOADER, BlockState::of(&ULTRA_CUBOID_LOADER));
}

/// What a camera held before it was handed the lens, and in which slot.
struct Held {
    player: u64,
    slot: u32,
    stack: ItemStack,
}

/// What each camera now holding the lens held before.
static HELD_BEFORE: Mutex<Vec<Held>> = Mutex::new(Vec::new());

fn held_before() -> MutexGuard<'static, Vec<Held>> {
    HELD_BEFORE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Hands the camera the cuboid lens: every cube cogwork keeps loaded is outlined. A camera
/// that has it already, its film started again, keeps what was noted for it the first time.
pub fn lens(set: &Set) {
    let world = set.world();
    let player = set.player();
    let hand = world.inventory(player);
    let mut held_before = held_before();
    let noted = held_before.iter().find(|held| held.player == player);
    let slot = match noted {
        Some(held) => held.slot,
        None => {
            let slot = world.selected_slot(player);
            let stack = hand.get(slot);
            held_before.push(Held {
                player,
                slot,
                stack,
            });
            slot
        }
    };
    hand.set(slot, ItemStack::new(CUBOID_LENS.id(), 1));
}

/// Takes the lens away again, and gives back what the camera held.
pub fn lens_away(set: &Set) {
    let player = set.player();
    let mut held_before = held_before();
    let Some(index) = held_before.iter().position(|held| held.player == player) else {
        return;
    };
    let held = held_before.swap_remove(index);
    set.world().inventory(player).set(held.slot, held.stack);
}
