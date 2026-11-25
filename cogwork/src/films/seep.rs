//! The oil seep west of the hall: a puddle of oil in oil-soaked ground (both oily dirt and oil
//! sand on its sandy lake-side edge), and its field 70 blocks below. A pump stands on the puddle
//! with ember engines on two sides and on top, its south side left clear for the camera; lit on
//! camera, it bores down to the field and pumps. A line of liquid tubes carries the crude oil
//! over the ground into Pad R's crude tank, with a gravel path beside it.

use showcase_api::{Film, Look, Set, dolly, flyby, orbit};
use weft_sdk::{Block, Dir, ENERGY, Shape};

use super::camera::{between, on_orbit, speeds};
use super::ground::{clear_trees_near, ground_top, mow};
use super::jetty;
use super::parts::{GRASS, engine, fuel, liquid_tubes, prefill, shafts};
use super::site::{above, lowered, lowered_block};
use crate::lib::{BlockState, Pos};
use crate::oil::{OIL, OIL_SAND, OILY_DIRT};
use crate::pump::PUMP;

static DIRT: Block = Block::named("base:dirt");

/// The pump, on the seep's puddle.
pub const PUMP_AT: Pos = Pos::new(-32, 3, -13);
/// The seep's soaked ground: oily dirt on the meadow side and oil sand on the north lake side.
const SOAKED_DIRT: [(i32, i32); 4] = [(-1, 1), (0, 1), (0, 2), (1, 1)];
const SOAKED_SAND: [(i32, i32); 4] = [(-1, 0), (0, -1), (1, -1), (1, 0)];
const FIELD_DEPTH: i32 = 70;
const FIELD_RADIUS: i32 = 6;
/// Five ember engines (5 × 40 = 200 E/t = the pump's full `DRAW_RATE`) behind and above the pump,
/// leaving the south face unobstructed for the camera: two directly on the pump (west and up) and
/// three feeding a 1-block torque shaft on its north side.
const DIRECT_ENGINES: [Dir; 2] = [Dir::West, Dir::Up];
const NORTH_SHAFT: Pos = Pos::new(PUMP_AT.x, PUMP_AT.y, PUMP_AT.z - 1);
const SHAFT_ENGINES: [(Pos, Dir); 3] = [
    (Pos::new(PUMP_AT.x, PUMP_AT.y, PUMP_AT.z - 2), Dir::South),
    (Pos::new(PUMP_AT.x - 1, PUMP_AT.y, PUMP_AT.z - 1), Dir::East),
    (Pos::new(PUMP_AT.x + 1, PUMP_AT.y, PUMP_AT.z - 1), Dir::West),
];
/// Blocks around the pump within which all trees, stumps, hollow logs and bushes are cleared.
const CLEARING: i32 = 18;
const MOWN: i32 = 6;

/// Camera orbit on the pump's open south face (120° SSW to 42° SE) heading east toward Pad R,
/// kept wide enough that the lens never grazes the pump's engines.
pub const SEEP_LOOK: [f64; 3] = lowered(above(PUMP_AT, 0.7));
const UP_BANK: [f64; 3] = [-17.0, -2.0, -12.0];
const HALL_WEST: [f64; 3] = lowered([super::hall::WEST as f64, 3.0, 0.0]);
const SEEP_ARC: [f64; 2] = [120.0, 42.0];
const SEEP_OUT: f64 = 6.8;
const SEEP_UP: [f64; 2] = [2.8, 3.2];

pub fn end_eye() -> [f64; 3] {
    on_orbit(SEEP_LOOK, SEEP_OUT, SEEP_UP[1], SEEP_ARC[1])
}

pub fn build(set: &Set) {
    clear_trees_near(set, PUMP_AT.x, PUMP_AT.z, CLEARING);
    mow(set, PUMP_AT.x, PUMP_AT.z, MOWN);
    // Before the pump and its engines stand there: it soaks the top of the ground.
    field(set);
    set.place(PUMP_AT, BlockState::of(&PUMP));
    for side in DIRECT_ENGINES {
        engine(set, PUMP_AT.toward(side), side.opposite());
    }
    shafts(set, NORTH_SHAFT, NORTH_SHAFT);
    for (at, toward) in SHAFT_ENGINES {
        engine(set, at, toward);
    }
    grade_pipe_berm(set);
    liquid_tubes(set, &oil_line());
}

/// Straight right-angle crude line from the seep pump at (-31, 3, -13) south to z = -4, east at
/// constant y = 3 to Pad R's west wall, one vertical drop at x = -20, and straight east along
/// Pad R's gravel into CRUDE_TANK at (-16, 0, -4).
fn oil_line() -> Vec<Pos> {
    let mut blocks: Vec<Pos> = (-13..=-4).map(|z| Pos::new(-31, 3, z)).collect();
    blocks.extend((-30..=-21).map(|x| Pos::new(x, 3, -4)));
    blocks.extend((0..=3).rev().map(|y| Pos::new(-20, y, -4)));
    blocks.extend((-19..=-17).map(|x| Pos::new(x, 0, -4)));
    blocks
}

/// Grades a flush grassy berm (y = 2) under the horizontal y = 3 stretch of `oil_line` so every
/// tube rests on solid grass with no diagonal steps or floating gaps.
fn grade_pipe_berm(set: &Set) {
    let cols = (-12..=-4)
        .map(|z| (-31, z))
        .chain((-30..=-21).map(|x| (x, -4)));
    for (x, z) in cols {
        let top = ground_top(set, x, z);
        if top < 2 {
            for y in top..2 {
                set.place(Pos::new(x, y, z), BlockState::of(&DIRT));
            }
            set.place(Pos::new(x, 2, z), BlockState::of(&GRASS));
        } else if top > 2 {
            for y in 3..=top + 1 {
                set.place(Pos::new(x, y, z), BlockState::AIR);
            }
            set.place(Pos::new(x, 2, z), BlockState::of(&GRASS));
        }
    }
}

/// The puddle under the pump, the soaked dirt and oil sand round it (kept flush so no terrain step
/// sticks up as a pillar), and the ball of oil below.
fn field(set: &Set) {
    let puddle = PUMP_AT.add_y(-1);
    set.place(puddle, BlockState::of(&OIL));
    for (offsets, block) in [(SOAKED_DIRT, &OILY_DIRT), (SOAKED_SAND, &OIL_SAND)] {
        for (dx, dz) in offsets {
            let (x, z) = (puddle.x + dx, puddle.z + dz);
            let top = ground_top(set, x, z);
            if top > puddle.y {
                for y in puddle.y + 1..=top {
                    set.place(Pos::new(x, y, z), BlockState::AIR);
                }
            }
            let at_y = top.min(puddle.y);
            set.place(Pos::new(x, at_y, z), BlockState::of(block));
        }
    }
    let middle = set.at(puddle.add_y(-FIELD_DEPTH));
    // Half a block past its radius, the roundness of the world's own fields.
    let ball = Shape::ball(middle.center(), f64::from(FIELD_RADIUS) + 0.5);
    set.world().fill(ball, OIL.id()).keep("core:air");
}

/// Prefills the crude oil line from the seep to Pad R so its glass tubes appear joined from the
/// first frame they enter view.
pub fn prefill_line(set: &Set) {
    prefill(set, oil_line().into_iter(), "cogwork:oil", [1, 1]);
}

/// Lights all five pump engines (200 E/t) and primes the pump's energy store so it never reads
/// short of power while boring or pumping.
pub fn bore(set: &Set) {
    for side in DIRECT_ENGINES {
        fuel(set, PUMP_AT.toward(side));
    }
    for (at, _) in SHAFT_ENGINES {
        fuel(set, at);
    }
    let world = set.world();
    let id = world.block_entity(set.at(PUMP_AT));
    if let Some(energy) = world.component_mut(id, &ENERGY) {
        energy.stored = 8_000;
    }
}

/// Up the bank from the jetty to the oil seep's open south face, orbiting west-to-east while it
/// bores down, with every title clear of the pump screen.
pub fn shots(film: Film) -> Film {
    let up = |angle: f64| {
        let part = (angle - SEEP_ARC[0]) / (SEEP_ARC[1] - SEEP_ARC[0]);
        between(SEEP_UP, part)
    };
    let start_orbit = on_orbit(SEEP_LOOK, SEEP_OUT, SEEP_UP[0], SEEP_ARC[0]);
    let to_seep = [UP_BANK, [-26.0, -0.4, -5.6], start_orbit];
    let round = |from: f64, to: f64, secs: f32| {
        orbit(SEEP_LOOK, SEEP_OUT)
            .arc(from, to)
            .height(up(from), up(to))
            .ease(speeds(1.1, 1.1))
            .secs(secs)
    };
    film.shot(
        dolly(jetty::PAST_JETTY, UP_BANK)
            .look(Look::Pan(jetty::SMALL_LOADER_LOOK, HALL_WEST))
            .ease(speeds(4.5, 4.0))
            .secs(3.0),
    )
    .shot(
        flyby(&to_seep)
            .look(Look::Pan(HALL_WEST, SEEP_LOOK))
            .ease(speeds(4.0, 1.1))
            .secs(6.0),
    )
    .cue("bore")
    .title("cogwork.film.seep", 3.5)
    .shot(round(SEEP_ARC[0], 98.0, 4.0))
    .menu("cogwork:pump", lowered_block(PUMP_AT), 4.0)
    .shot(round(98.0, 74.0, 4.5))
    .cue("start_refinery")
    .title("cogwork.film.field", 5.0)
    .shot(round(74.0, SEEP_ARC[1], 9.4))
}
