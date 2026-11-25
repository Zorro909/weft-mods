//! The south side of the hall, east of the oil engine: autocrafting in two steps along one
//! row, west to east. Sticks go by tube into a bench that makes wooden cogs, a timber tube
//! takes them on to a second bench, which also gets cobblestone from the chest at the row's
//! east end and makes stone cogs; those leave it to the north, into the last chest.
//!
//! Nothing here has an engine of its own but the last timber tube: a torque shaft runs behind
//! the row and drives both benches and the timber tubes with what the oil engine makes, so the
//! row starts when the engine does. The camera looks at it from the north.

use showcase_api::{Film, Look, Set, dolly};
use weft_sdk::{Dir, Item};

use super::camera::{halfway, on_orbit, speeds};
use super::engine_bay::{self, SHAFT_X as ENGINE_SHAFT_X};
use super::parts::{CHEST, engine, fuel, shafts, stock, tubes};
use super::site::{above, lowered, lowered_block};
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
/// The shaft behind the row, from PULLS[0] (x = 9) through the drop at x = 10 to the hall's east
/// end.
const SHAFT_X: [i32; 2] = [9, super::hall::EAST];

/// The cells of the patterns: a wooden cog is four sticks round a gap, a stone cog four
/// cobblestones round a wooden cog.
const ROUND: [u32; 4] = [1, 3, 5, 7];
const MIDDLE: u32 = 4;

/// Camera look targets and handoff point leaving the second bench toward the hall's open east end.
pub const BENCH_LOOK: [f64; 3] = lowered(above(STONE_COGS, 0.1));
const SHAFT_LOOK: [f64; 3] = lowered([ENGINE_SHAFT_X as f64 + 0.5, 4.2, 1.5]);
const SHAFT_FOOT_LOOK: [f64; 3] = lowered(above(Pos::new(ENGINE_SHAFT_X + 2, 1, ROW_Z), 0.2));
const BENCH_OUT: f64 = 4.4;
const BENCH_FROM: f64 = -112.0;
const BENCH_UP: [f64; 2] = [2.8, 2.2];

pub fn end_eye() -> [f64; 3] {
    on_orbit(BENCH_LOOK, BENCH_OUT, BENCH_UP[1], BENCH_FROM)
}

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

/// Follows the shaft across the hall to the assembly benches and opens the second bench's screen,
/// panning to the shaft clear of the oil engine before the shaft title begins and keeping every
/// title clear of the bench screen.
pub fn shots(film: Film) -> Film {
    let from_engine = engine_bay::END_EYE;
    let [over_bench, at_bench] = BENCH_UP.map(|up| on_orbit(BENCH_LOOK, BENCH_OUT, up, BENCH_FROM));
    let clear_of_engine = lowered([13.4, 3.8, -0.2]);
    let under_shaft = halfway(clear_of_engine, over_bench);
    film.shot(
        dolly(from_engine, clear_of_engine)
            .look(Look::Pan(engine_bay::END_LOOK, SHAFT_LOOK))
            .ease(speeds(0.15, 0.45))
            .secs(2.6),
    )
    .title("cogwork.film.shaft", 4.5)
    .shot(
        dolly(clear_of_engine, under_shaft)
            .look(Look::Pan(SHAFT_LOOK, SHAFT_FOOT_LOOK))
            .ease(speeds(0.45, 0.5))
            .secs(3.2),
    )
    .shot(
        dolly(under_shaft, over_bench)
            .look_at(SHAFT_FOOT_LOOK)
            .ease(speeds(0.5, 0.3))
            .secs(2.5),
    )
    .title("cogwork.film.bench", 3.5)
    .shot(
        dolly(over_bench, at_bench)
            .look(Look::Pan(SHAFT_FOOT_LOOK, BENCH_LOOK))
            .ease(speeds(0.3, 0.25))
            .secs(4.0),
    )
    .menu("cogwork:assembly_bench", lowered_block(STONE_COGS), 4.0)
    .wait(4.2)
}
