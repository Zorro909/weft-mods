//! Redstone: signals, wire and the devices around them, predictable and cheap. The rules live in
//! `redstone_core`; this crate maps Weft's blocks onto them.

mod adapter;
mod blocks;
mod driver;
mod persist;

use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

fn init(r: &mut Registry) {
    r.config(&redstone_api::RANGE).choice(0, &["16", "32", "64"]).world();
    blocks::register(r);
    driver::register(r);
}
