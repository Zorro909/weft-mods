//! Keeping the works running with nobody near: an ultra cuboid loader on a stone mast at the
//! corner of the digger's pad, north of the hall's east end, and the cuboid lens, which the
//! camera is handed for a while to show what is kept: the loader's cubes in cyan, the working
//! machines' in orange.

use std::sync::{Mutex, MutexGuard, PoisonError};

use showcase_api::{Film, Look, Set, dolly, flyby};
use weft_sdk::ItemStack;

use super::camera::speeds;
use super::parts::{STONE_BRICKS, area};
use super::quarry::{self, FRAME_LOOK, PIT_LOOK};
use super::site::{above, lowered, lowered_block};
use crate::lib::{BlockState, Pos};
use crate::loading::{CUBOID_LENS, ULTRA_CUBOID_LOADER};

/// The loader, on top of its mast. It keeps the 7 by 7 cubes of the layer it stands in: this
/// is the highest block of the layer the machines stand in, and from this cube the 7 by 7
/// reach across the campus.
pub const LOADER: Pos = Pos::new(27, 3, -11);
/// The lowest block of the mast, on the pad.
const MAST_FOOT: i32 = 0;

/// Camera waypoints for the finale: loader close-up, cuboid lens panorama, and closing rest.
const OVER_KNOLL: [f64; 3] = [50.0, 5.0, 7.0];
const PAST_DIGGER: [f64; 3] = lowered([41.0, 9.0, -11.0]);
const BY_LOADER: [[f64; 3]; 3] = [
    lowered([31.6, 5.2, -12.2]),
    lowered([31.7, 5.4, -12.7]),
    lowered([31.9, 5.8, -13.6]),
];
const LOADER_LOOK: [f64; 3] = lowered(above(LOADER, 0.0));
const OVER_LOADER_LOOK: [f64; 3] = lowered(above(LOADER, 1.3));
const WORKS_LOOK: [f64; 3] = lowered([22.0, 0.0, -8.0]);
const SHORE_LOOK: [f64; 3] = lowered([18.0, -2.0, -15.0]);

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

/// Section 11 (Finale): over the levelled knoll to the ultra cuboid loader, then high out over
/// the lake with the cuboid lens and back down to a wide final look along the works.
pub fn shots(film: Film) -> Film {
    let from_frame = quarry::frame_end_eye();
    let down = [OVER_KNOLL, PAST_DIGGER, BY_LOADER[0]];
    let over_lake = lowered([37.0, 13.0, -28.0]);
    let north_east = lowered([68.0, 27.0, -34.0]);
    let with_lens = [over_lake, lowered([50.0, 25.0, -42.0]), north_east];
    let to_shore = [
        north_east,
        lowered([66.0, 15.0, -27.0]),
        lowered([54.0, 8.5, -22.0]),
    ];

    film.shot(
        dolly(from_frame, OVER_KNOLL)
            .look(Look::Pan(FRAME_LOOK, PIT_LOOK))
            .ease(speeds(2.0, 5.0))
            .secs(3.0),
    )
    .shot(
        flyby(&down)
            .look(Look::Pan(PIT_LOOK, OVER_LOADER_LOOK))
            .ease(speeds(5.0, 0.2))
            .secs(5.5),
    )
    .title("cogwork.film.loader", 5.0)
    .shot(
        dolly(BY_LOADER[0], BY_LOADER[1])
            .look_at(OVER_LOADER_LOOK)
            .ease(speeds(0.2, 0.2))
            .secs(1.5),
    )
    .menu("cogwork:ultra_cuboid_loader", lowered_block(LOADER), 4.5)
    .shot(
        dolly(BY_LOADER[1], BY_LOADER[2])
            .look_at(OVER_LOADER_LOOK)
            .ease(speeds(0.2, 0.3))
            .secs(4.5),
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
