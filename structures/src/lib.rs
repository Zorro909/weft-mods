//! Structures from files: the `.wst` files compiled into the mod appear in base's overworld,
//! alone and gathered into hamlets, and `/structure save` writes a box of the world to one.

mod hamlet;
mod save;
mod worldgen;

use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

fn init(r: &mut Registry) {
    worldgen::register(r);
    hamlet::register(r);
    save::register(r);
}
