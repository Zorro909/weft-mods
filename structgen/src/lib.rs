//! Structures from files: the `.wst` files compiled into the mod appear in base's overworld, a
//! few alone and the rest in hamlets and villages. `/structure` and the wand select a box of the
//! world and save it to a file, `/structure` loads one in front of the player and finds the
//! nearest structure.

mod command;
mod load;
mod locate;
mod save;
mod selection;
mod settlement;
mod styles;
mod villages;
mod wand;
mod worldgen;

use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

fn init(r: &mut Registry) {
    worldgen::register(r);
    villages::register(r);
    wand::register(r);
    selection::register(r);
    command::register(r);
}
