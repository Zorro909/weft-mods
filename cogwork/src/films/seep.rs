//! The oil seep west of the hall: a puddle of oil in oil-soaked ground (both oily dirt and oil
//! sand on its sandy lake-side edge), and its field 70 blocks below. A pump stands on the puddle
//! with ember engines on two sides and on top, its south side left clear for the camera; lit on
//! camera, it bores down to the field and pumps. A line of liquid tubes carries the crude oil
//! over the ground into Pad R's crude tank, with a gravel path beside it.

use showcase_api::{Film, Look, Set, dolly, flyby, orbit};
use weft_sdk::{Dir, Shape};

use super::camera::{between, on_orbit, speeds};
use super::ground::{fell_near, ground_top, mow, over_ground, pave, route};
use super::jetty;
use super::parts::{GRAVEL, engine, fuel, liquid_tubes};
use super::site::{above, lowered, lowered_block};
use crate::lib::{BlockState, Pos};
use crate::oil::{OIL, OIL_SAND, OILY_DIRT};
use crate::pump::PUMP;

/// The pump, on the seep's puddle.
pub const PUMP_AT: Pos = Pos::new(-32, 3, -13);
/// The seep's soaked ground: oily dirt on the meadow side and oil sand on the north lake side.
const SOAKED_DIRT: [(i32, i32); 4] = [(-1, 1), (0, 1), (0, 2), (1, 1)];
const SOAKED_SAND: [(i32, i32); 4] = [(-1, 0), (0, -1), (1, -1), (1, 0)];
const FIELD_DEPTH: i32 = 70;
const FIELD_RADIUS: i32 = 6;
/// The sides of the pump its engines stand on; its line leaves it to the east.
const ENGINE_SIDES: [Dir; 3] = [Dir::North, Dir::West, Dir::Up];
/// The crude oil line from beside the pump to Pad R's crude tank at (-16, 0, -5).
const LINE: [(i32, i32); 3] = [(-31, -13), (-18, -5), (-17, -5)];
/// The path beside the line: its rows along z, and its ends along x.
const PATH_Z: [i32; 2] = [-12, -11];
const PATH_X: [i32; 2] = [-28, -21];
/// Blocks around the pump within which no tree is left standing, and within which tall grass is
/// mown, which hid the pump.
const CLEARING: i32 = 11;
const MOWN: i32 = 5;

/// Camera orbit around the seep and handoff point heading east toward Pad R.
pub const SEEP_LOOK: [f64; 3] = lowered(above(PUMP_AT, 0.7));
const UP_BANK: [f64; 3] = [-17.0, -4.0, -16.0];
const HALL_WEST: [f64; 3] = lowered([super::hall::WEST as f64, 3.0, 0.0]);
const SEEP_ARC: [f64; 2] = [-90.0, -270.0];
const SEEP_OUT: f64 = 5.6;
const SEEP_UP: [f64; 2] = [2.4, 2.8];

pub fn end_eye() -> [f64; 3] {
    on_orbit(SEEP_LOOK, SEEP_OUT, SEEP_UP[1], SEEP_ARC[1])
}

pub fn build(set: &Set) {
    fell_near(set, PUMP_AT.x, PUMP_AT.z, CLEARING);
    mow(set, PUMP_AT.x, PUMP_AT.z, MOWN);
    // Before the pump and its engines stand there: it soaks the top of the ground.
    field(set);
    set.place(PUMP_AT, BlockState::of(&PUMP));
    for side in ENGINE_SIDES {
        engine(set, PUMP_AT.toward(side), side.opposite());
    }
    let path = (PATH_X[0]..=PATH_X[1]).flat_map(|x| PATH_Z.map(|z| (x, z)));
    pave(set, path, &GRAVEL);
    liquid_tubes(set, &over_ground(set, &route(&LINE)));
}

/// The puddle under the pump, the soaked dirt and oil sand round it, and the ball of oil below.
fn field(set: &Set) {
    let puddle = PUMP_AT.add_y(-1);
    set.place(puddle, BlockState::of(&OIL));
    for (offsets, block) in [(SOAKED_DIRT, &OILY_DIRT), (SOAKED_SAND, &OIL_SAND)] {
        for (dx, dz) in offsets {
            let (x, z) = (puddle.x + dx, puddle.z + dz);
            let ground = Pos::new(x, ground_top(set, x, z), z);
            set.place(ground, BlockState::of(block));
        }
    }
    let middle = set.at(puddle.add_y(-FIELD_DEPTH));
    // Half a block past its radius, the roundness of the world's own fields.
    let ball = Shape::ball(middle.center(), f64::from(FIELD_RADIUS) + 0.5);
    set.world().fill(ball, OIL.id()).keep("core:air");
}

/// Lights the pump's engines: it bores down to the field and pumps.
pub fn bore(set: &Set) {
    for side in ENGINE_SIDES {
        fuel(set, PUMP_AT.toward(side));
    }
}

/// Up the bank from the jetty to the oil seep, and half round the pump while it bores down.
pub fn shots(film: Film) -> Film {
    let up = |angle: f64| {
        let part = (angle - SEEP_ARC[0]) / (SEEP_ARC[1] - SEEP_ARC[0]);
        between(SEEP_UP, part)
    };
    let north = on_orbit(SEEP_LOOK, SEEP_OUT, SEEP_UP[0], SEEP_ARC[0]);
    let to_seep = [UP_BANK, [-24.0, -2.0, -21.0], north];
    let round = |from: f64, to: f64, secs: f32| {
        orbit(SEEP_LOOK, SEEP_OUT)
            .arc(from, to)
            .height(up(from), up(to))
            .ease(speeds(1.2, 1.2))
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
            .ease(speeds(4.0, 1.2))
            .secs(6.0),
    )
    .cue("bore")
    .title("cogwork.film.seep", 5.0)
    .shot(round(SEEP_ARC[0], -114.0, 2.5))
    .menu("cogwork:pump", lowered_block(PUMP_AT), 9.0)
    .shot(round(-114.0, -178.0, 6.5))
    .cue("start_refinery")
    .title("cogwork.film.field", 5.0)
    .shot(round(-178.0, SEEP_ARC[1], 9.4))
}
