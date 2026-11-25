//! Keeping the works running with nobody near: an ultra cuboid loader on a stone mast at the
//! corner of the digger's pad, north of the hall's east end, and the cuboid lens, which the
//! camera is handed for a while to show what is kept: the loader's cubes in cyan, the working
//! machines' in orange.

use std::sync::{Mutex, MutexGuard, PoisonError};

use showcase_api::{Film, Look, Set, dolly, flyby};
use weft_sdk::ItemStack;

use super::camera::speeds;
use super::quarry::{self, FRAME_LOOK, PIT_LOOK};
use super::site::{above, lowered, lowered_block};
use crate::lib::{BlockState, Pos};
use crate::loading::{CUBOID_LENS, ULTRA_CUBOID_LOADER};

/// The ultra cuboid loader standing directly on Pad 1's north-west gravel (no plinth, 2 blocks
/// west and 3 blocks north of the digger frame at x 28..38, z -9..1). Still sits in the same
/// y = 64..79 cube so its 7 by 7 reach covers the whole campus.
pub const LOADER: Pos = Pos::new(26, 0, -12);

/// Camera waypoints for the finale: high over the knoll and north of the digger frame to a
/// north-west view of the loader (with the frame behind it), cuboid lens panorama, and a closer
/// final look across the hall and campus.
const OVER_KNOLL: [f64; 3] = [50.0, 9.5, 7.5];
const PAST_DIGGER: [f64; 3] = lowered([42.0, 12.0, -16.0]);
const BY_LOADER: [[f64; 3]; 3] = [
    lowered([22.5, 3.2, -16.2]),
    lowered([22.0, 3.5, -16.7]),
    lowered([21.6, 3.9, -17.3]),
];
const LOADER_LOOK: [f64; 3] = lowered(above(LOADER, 0.2));
const OVER_LOADER_LOOK: [f64; 3] = lowered(above(LOADER, 1.8));
const WORKS_LOOK: [f64; 3] = lowered([20.0, 1.5, -4.0]);
const SHORE_LOOK: [f64; 3] = lowered([14.0, 2.5, -2.0]);

pub fn build(set: &Set) {
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

/// Section 11 (Finale): over the levelled knoll to the ultra cuboid loader, then high out over
/// the lake with the cuboid lens and back down to a wide final look along the works.
pub fn shots(film: Film) -> Film {
    let from_frame = quarry::frame_end_eye();
    let down = [OVER_KNOLL, PAST_DIGGER, BY_LOADER[0]];
    let over_lake = lowered([35.0, 14.0, -28.0]);
    let north_east = lowered([62.0, 24.0, -32.0]);
    let with_lens = [over_lake, lowered([48.0, 22.0, -38.0]), north_east];
    let to_shore = [
        north_east,
        lowered([54.0, 14.0, -24.0]),
        lowered([42.0, 7.5, -18.0]),
    ];

    film.shot(
        dolly(from_frame, OVER_KNOLL)
            .look(Look::Pan(FRAME_LOOK, PIT_LOOK))
            .ease(speeds(2.0, 5.0))
            .secs(3.2),
    )
    .shot(
        flyby(&down)
            .look(Look::Pan(PIT_LOOK, OVER_LOADER_LOOK))
            .ease(speeds(5.0, 0.25))
            .secs(6.0),
    )
    .title("cogwork.film.loader", 3.0)
    .shot(
        dolly(BY_LOADER[0], BY_LOADER[1])
            .look_at(OVER_LOADER_LOOK)
            .ease(speeds(0.25, 0.2))
            .secs(3.2),
    )
    .menu("cogwork:ultra_cuboid_loader", lowered_block(LOADER), 4.0)
    .shot(
        dolly(BY_LOADER[1], BY_LOADER[2])
            .look_at(OVER_LOADER_LOOK)
            .ease(speeds(0.2, 0.3))
            .secs(4.2),
    )
    .shot(
        dolly(BY_LOADER[2], over_lake)
            .look(Look::Pan(OVER_LOADER_LOOK, LOADER_LOOK))
            .ease(speeds(0.3, 5.0))
            .secs(4.0),
    )
    .cue("lens")
    .title("cogwork.film.lens", 5.0)
    .shot(
        flyby(&with_lens)
            .look(Look::Pan(LOADER_LOOK, WORKS_LOOK))
            .ease(speeds(5.0, 4.5))
            .secs(7.5),
    )
    .cue("lens_away")
    .title("cogwork.film.end", 7.0)
    .shot(
        flyby(&to_shore)
            .look(Look::Pan(WORKS_LOOK, SHORE_LOOK))
            .ease(speeds(4.5, 0.3))
            .secs(6.5),
    )
    .wait(1.5)
    .cue("unpack")
}
