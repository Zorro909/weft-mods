//! Structures from files: the `.wst` files compiled into the mod appear in base's overworld,
//! alone and gathered into hamlets. `/structure` and the wand select a box of the world and save
//! it to a file, `/structure` loads one in front of the player and finds the nearest structure.

mod command;
mod hamlet;
mod load;
mod locate;
mod save;
mod selection;
mod styles;
mod wand;
mod worldgen;

use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

fn init(r: &mut Registry) {
    worldgen::register(r);
    hamlet::register(r);
    wand::register(r);
    selection::register(r);
    command::register(r);
}
