//! Films of cogwork for the showcase mod, built only with the `showcase` feature:
//! `weft --showcase cogwork:tubes`.

use showcase_api::{Film, Look, Set, crane, dolly, film, flyby, orbit};
use weft_sdk::*;

use crate::engines::EMBER_ENGINE;
use crate::tube::{GILDED_TUBE, STONE_TUBE, TIMBER_TUBE};

static CHEST: Block = Block::named("base:chest");
/// What the first chest holds, a stack each, so that the tubes carry a mix.
const CARGO: [&str; 5] = [
    "base:iron_ingot",
    "base:gold_ingot",
    "base:diamond",
    "base:coal",
    "base:cobblestone",
];

pub fn register(r: &mut Registry) {
    // Block entities only exist once the tick they were placed in is over.
    film(r, "tubes", tubes).on_cue("fill", fill);
}

/// Items leaving a chest, over an arch of tubes, into another chest.
fn tubes(set: &Set) -> Film {
    build_tube_arch(set);
    let middle = [0.5, 1.5, 0.5];
    Film::new()
        .cue("fill")
        .title("cogwork.film.tubes", 4.0)
        .shot(
            orbit(middle, 10.0)
                .arc(200.0, 20.0)
                .height(5.0, 2.0)
                .secs(10.0),
        )
        .shot(
            dolly([-6.0, 2.0, 3.5], [6.0, 2.0, 3.5])
                .look(Look::Pan([-5.0, 0.5, 0.5], [5.0, 0.5, 0.5]))
                .secs(8.0)
                .blend(1.5),
        )
        .shot(
            crane([5.5, 0.5, 0.5], 4.0, 60.0, [0.5, 5.0])
                .secs(5.0)
                .blend(1.0),
        )
        .shot(
            flyby(&[[7.0, 6.0, 7.0], [0.0, 10.0, 11.0], [-11.0, 14.0, 5.0]])
                .look_at(middle)
                .secs(6.0)
                .blend(1.0),
        )
}

/// A chest, a timber tube driven by an ember engine, stone tubes up over a gilded top and down
/// again, and the chest they end in.
fn build_tube_arch(set: &Set) {
    set.place([-6, 0, 0], CHEST.id());
    set.place([-5, 0, 0], TIMBER_TUBE.id());
    // `facing` is the side its base plate is on; the trunk points the other way.
    set.place([-5, 1, 0], EMBER_ENGINE.with(&[("facing", "up")]));
    let path: [[i32; 3]; 13] = [
        [-4, 0, 0],
        [-3, 0, 0],
        [-2, 0, 0],
        [-2, 1, 0],
        [-2, 2, 0],
        [-1, 2, 0],
        [0, 2, 0],
        [1, 2, 0],
        [2, 2, 0],
        [2, 1, 0],
        [2, 0, 0],
        [3, 0, 0],
        [4, 0, 0],
    ];
    for at in path {
        let gilded = at[1] == 2 && at[0].abs() <= 1;
        set.place(
            at,
            if gilded {
                GILDED_TUBE.id()
            } else {
                STONE_TUBE.id()
            },
        );
    }
    set.place([5, 0, 0], CHEST.id());
}

/// Cargo into the first chest and coal into the engine.
fn fill(set: &Set) {
    let world = set.world();
    let chest = world.inventory(world.block_entity(set.at([-6, 0, 0])));
    for item in CARGO.iter().filter_map(|name| Item::lookup(name)) {
        chest.insert(ItemStack::new(item, 64));
    }
    let engine = world.inventory(world.block_entity(set.at([-5, 1, 0])));
    if let Some(coal) = Item::lookup("base:coal") {
        engine.group("fuel").set(0, ItemStack::new(coal, 64));
    }
}
