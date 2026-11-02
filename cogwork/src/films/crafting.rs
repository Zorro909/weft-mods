//! Autocrafting in two steps along one row, west to east: sticks go by tube into a bench that
//! makes wooden cogs, a timber tube takes them on to a second bench, which also gets
//! cobblestone from its own line from the north and makes stone cogs into the last chest. The
//! camera looks at the row from the south, so nothing stands in front of it there.

use showcase_api::Set;
use weft_sdk::*;

use super::excavators::beside;
use super::parts::{CHEST, engine, fuel, line, stock};
use crate::bench::{ASSEMBLY_BENCH, set_ghost};
use crate::tube::TIMBER_TUBE;

/// The row everything stands on, and where along it.
pub const ROW_Z: i32 = 14;
pub const STICKS: [i32; 3] = [-12, 0, ROW_Z];
pub const COGS: [i32; 3] = [-7, 0, ROW_Z];
pub const STONE_COGS: [i32; 3] = [-2, 0, ROW_Z];
const OUTPUT: [i32; 3] = [3, 0, ROW_Z];
/// The cobblestone's chest, north of the second bench.
const COBBLE: [i32; 3] = [-2, 0, ROW_Z - 4];
/// Timber tubes, each with an engine on top: out of the stick chest, out of each bench, and
/// out of the cobblestone chest.
const PULLS: [[i32; 3]; 4] = [
    [-11, 0, ROW_Z],
    [-6, 0, ROW_Z],
    [-1, 0, ROW_Z],
    [-2, 0, ROW_Z - 3],
];

/// The cells of the patterns: a wooden cog is four sticks round a gap, a stone cog four
/// cobblestones round a wooden cog.
const ROUND: [u32; 4] = [1, 3, 5, 7];
const MIDDLE: u32 = 4;

pub fn build(set: &Set) {
    for chest in [STICKS, COBBLE, OUTPUT] {
        set.place(chest, CHEST.id());
    }
    for bench in [COGS, STONE_COGS] {
        set.place(bench, ASSEMBLY_BENCH.with(&[("facing", "south")]));
        engine(set, beside(bench, Dir::Up), Dir::Down);
    }
    for pull in PULLS {
        set.place(pull, TIMBER_TUBE.id());
        engine(set, beside(pull, Dir::Up), Dir::Down);
    }
    line(set, [-10, 0, ROW_Z], [-8, 0, ROW_Z]);
    line(set, [-5, 0, ROW_Z], [-3, 0, ROW_Z]);
    line(set, [-2, 0, ROW_Z - 2], [-2, 0, ROW_Z - 1]);
    line(set, [0, 0, ROW_Z], [2, 0, ROW_Z]);
}

/// Fills the supply chests, lays out both patterns and starts every engine.
pub fn start(set: &Set) {
    stock(set, STICKS, &["base:stick"], 6);
    stock(set, COBBLE, &["base:cobblestone"], 6);
    let world = set.world();
    let item = |name: &str| Item::lookup(name).unwrap_or(0);
    let (cogs, stone_cogs) = (
        world.block_entity(set.at(COGS)),
        world.block_entity(set.at(STONE_COGS)),
    );
    for cell in ROUND {
        set_ghost(&world, cogs, cell, item("base:stick"));
        set_ghost(&world, stone_cogs, cell, item("base:cobblestone"));
    }
    set_ghost(&world, stone_cogs, MIDDLE, item("cogwork:wooden_cog"));
    for machine in [COGS, STONE_COGS].into_iter().chain(PULLS) {
        fuel(set, beside(machine, Dir::Up));
    }
}
