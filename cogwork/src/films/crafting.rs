//! The south side of the hall, east of the oil engine: autocrafting in two steps along one
//! row, west to east. Sticks go by tube into a bench that makes wooden cogs, a timber tube
//! takes them on to a second bench, which also gets cobblestone from the chest at the row's
//! east end and makes stone cogs; those leave it to the north, into the last chest.
//!
//! Nothing here has an engine of its own but the last timber tube: a torque shaft runs behind
//! the row and drives both benches and the timber tubes with what the oil engine makes, so the
//! row starts when the engine does. The camera looks at it from the north.

use showcase_api::Set;
use weft_sdk::{Dir, Item};

use super::parts::{CHEST, engine, fuel, shafts, stock, tubes};
use crate::bench::{ASSEMBLY_BENCH, set_ghost};
use crate::lib::{BlockState, Pos};
use crate::tube::TIMBER_TUBE;

/// The row the machines stand in, and the row behind it the shaft runs in.
const ROW_Z: i32 = 4;
pub const SHAFT_Z: i32 = ROW_Z + 1;
const STICKS: Pos = Pos::new(8, 0, ROW_Z);
const COGS: Pos = Pos::new(12, 0, ROW_Z);
pub const STONE_COGS: Pos = Pos::new(16, 0, ROW_Z);
const COBBLE: Pos = Pos::new(19, 0, ROW_Z);
/// Timber tubes the shaft drives: out of the stick chest, out of the first bench, and out of
/// the cobblestone chest.
const PULLS: [Pos; 3] = [
    Pos::new(9, 0, ROW_Z),
    Pos::new(13, 0, ROW_Z),
    Pos::new(18, 0, ROW_Z),
];
/// Out of the second bench to the north: a timber tube with its own engine east of it, low
/// and out of the camera's way to the bench, a tube and the chest the stone cogs end in.
const OUT_PULL: Pos = Pos::new(16, 0, ROW_Z - 1);
const OUT_ENGINE: Pos = Pos::new(17, 0, ROW_Z - 1);
const OUT_TUBE: Pos = Pos::new(16, 0, ROW_Z - 2);
const OUTPUT: Pos = Pos::new(16, 0, ROW_Z - 3);
/// The shaft behind the row, from where it comes down from the oil engine to the hall's east
/// end.
const SHAFT_X: [i32; 2] = [super::engine_bay::SHAFT_X, super::hall::EAST];

/// The cells of the patterns: a wooden cog is four sticks round a gap, a stone cog four
/// cobblestones round a wooden cog.
const ROUND: [u32; 4] = [1, 3, 5, 7];
const MIDDLE: u32 = 4;

pub fn build(set: &Set) {
    for chest in [STICKS, COBBLE, OUTPUT] {
        set.place(chest, BlockState::of(&CHEST));
    }
    for bench in [COGS, STONE_COGS] {
        set.place(bench, ASSEMBLY_BENCH.with(&[("facing", "north")]).into());
    }
    for pull in PULLS.into_iter().chain([OUT_PULL]) {
        set.place(pull, BlockState::of(&TIMBER_TUBE));
    }
    engine(set, OUT_ENGINE, Dir::West);
    let on_row = |x: i32| Pos::new(x, 0, ROW_Z);
    tubes(set, on_row(10), on_row(11));
    tubes(set, on_row(14), on_row(15));
    tubes(set, on_row(17), on_row(17));
    tubes(set, OUT_TUBE, OUT_TUBE);
    let behind = |x: i32| Pos::new(x, 0, SHAFT_Z);
    shafts(set, behind(SHAFT_X[0]), behind(SHAFT_X[1]));
}

/// Fills the supply chests, lays out both patterns and lights the last timber tube's engine;
/// the row waits for the shaft to turn.
pub fn start(set: &Set) {
    stock(set, STICKS, &["base:stick"], 6);
    stock(set, COBBLE, &["base:cobblestone"], 6);
    fuel(set, OUT_ENGINE);
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
}
