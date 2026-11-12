//! The oil seep west of the hall: a puddle of oil in oil-soaked ground, and its field 90 blocks
//! below. The world's own fields are few and none lies here, so the film lays this one as the
//! world would. A pump stands on the puddle under a timber derrick,
//! with ember engines on two sides and on top, its south side left clear for the camera; lit
//! on camera, it bores down to the field and pumps. A line of liquid tubes carries the oil
//! over the ground into the hall's oil tank, with a gravel path beside it.

use showcase_api::Set;
use weft_sdk::{Dir, Shape};

use super::ground::{fell_near, ground_top, mow, over_ground, pave, route};
use super::parts::{GLOWSTONE, GRAVEL, OAK_LOG, area, engine, fuel, liquid_tubes};
use crate::lib::{BlockState, Pos};
use crate::oil::{OIL, OILY_DIRT};
use crate::pump::PUMP;

/// The pump, on the seep's puddle.
pub const PUMP_AT: Pos = Pos::new(-32, 3, -13);
/// The seep's soaked ground, as far from the puddle along x and z, and its field: how far
/// under the puddle the middle of its ball of oil lies, and the ball's radius.
const SOAKED: [(i32, i32); 8] = [
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (0, 2),
    (1, -1),
    (1, 0),
    (1, 1),
];
const FIELD_DEPTH: i32 = 90;
const FIELD_RADIUS: i32 = 6;
/// The sides of the pump its engines stand on; its line leaves it to the east.
const ENGINE_SIDES: [Dir; 3] = [Dir::North, Dir::West, Dir::Up];
/// The oil line, from beside the pump to the block against the hall's oil tank: east along
/// the seep's row, then south to the hall's.
const LINE: [(i32, i32); 4] = [(-31, -13), (-14, -6), (-13, -5), (3, -5)];
/// The path beside the line: its rows along z, and its ends along x.
const PATH_Z: [i32; 2] = [-12, -11];
const PATH_X: [i32; 2] = [-28, -15];
/// The derrick: how far east of the pump its two legs stand, how far north and south of the
/// pump's row, and the height of its top.
const LEGS_EAST: i32 = 2;
const LEGS_APART: i32 = 2;
const DERRICK_TOP: i32 = 9;
/// Blocks around the pump within which no tree is left standing, and within which the tall
/// grass is mown, which hid the pump.
const CLEARING: i32 = 9;
const MOWN: i32 = 4;

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
    derrick(set);
}

/// The puddle under the pump, the soaked ground round it, and the ball of oil far below, which
/// the engine fills in as its cubes load: all but the air of a cave.
fn field(set: &Set) {
    let puddle = PUMP_AT.add_y(-1);
    set.place(puddle, BlockState::of(&OIL));
    for (dx, dz) in SOAKED {
        let (x, z) = (puddle.x + dx, puddle.z + dz);
        let ground = Pos::new(x, ground_top(set, x, z), z);
        set.place(ground, BlockState::of(&OILY_DIRT));
    }
    let middle = set.at(puddle.add_y(-FIELD_DEPTH));
    // Half a block past its radius, the roundness of the world's own fields.
    let ball = Shape::ball(middle.center(), f64::from(FIELD_RADIUS) + 0.5);
    set.world().fill(ball, OIL.id()).keep("core:air");
}

/// Two legs east of the pump, a beam across their tops, and from its middle a jib out over
/// the pump with a lamp under it: all on the side the camera does not come round.
fn derrick(set: &Set) {
    let (x, z) = (PUMP_AT.x, PUMP_AT.z);
    let legs_x = x + LEGS_EAST;
    for leg_z in [z - LEGS_APART, z + LEGS_APART] {
        let foot = Pos::new(legs_x, ground_top(set, legs_x, leg_z) + 1, leg_z);
        set.fill(
            area(foot, foot.with_y(DERRICK_TOP)),
            BlockState::of(&OAK_LOG),
        );
    }
    let across = area(
        Pos::new(legs_x, DERRICK_TOP, z - LEGS_APART + 1),
        Pos::new(legs_x, DERRICK_TOP, z + LEGS_APART - 1),
    );
    set.fill(across, OAK_LOG.with(&[("axis", "z")]).into());
    let jib = area(
        Pos::new(x - 1, DERRICK_TOP, z),
        Pos::new(legs_x - 1, DERRICK_TOP, z),
    );
    set.fill(jib, OAK_LOG.with(&[("axis", "x")]).into());
    set.place(Pos::new(x, DERRICK_TOP - 1, z), BlockState::of(&GLOWSTONE));
}

/// Lights the pump's engines: it bores down to the field and pumps.
pub fn bore(set: &Set) {
    for side in ENGINE_SIDES {
        fuel(set, PUMP_AT.toward(side));
    }
}
