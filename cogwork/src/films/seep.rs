//! The oil seep west of the hall, as the world made it: a puddle of oil in oil-soaked ground,
//! and its field some 80 blocks below. A pump stands on the puddle under a timber derrick,
//! with ember engines on two sides and on top, its south side left clear for the camera; lit
//! on camera, it bores down to the field and pumps. A line of liquid tubes carries the oil
//! over the ground into the hall's oil tank, with a gravel path beside it.

use showcase_api::Set;
use weft_sdk::Dir;

use super::ground::{fell_near, ground_top, mow, over_ground, pave, route};
use super::parts::{GLOWSTONE, GRAVEL, OAK_LOG, area, engine, fuel, liquid_tubes};
use crate::lib::{BlockState, Pos};
use crate::pump::PUMP;

/// The pump, on the seep's puddle.
pub const PUMP_AT: Pos = Pos::new(-32, 3, -13);
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
    set.place(PUMP_AT, BlockState::of(&PUMP));
    for side in ENGINE_SIDES {
        engine(set, PUMP_AT.toward(side), side.opposite());
    }
    fell_near(set, PUMP_AT.x, PUMP_AT.z, CLEARING);
    mow(set, PUMP_AT.x, PUMP_AT.z, MOWN);
    let path = (PATH_X[0]..=PATH_X[1]).flat_map(|x| PATH_Z.map(|z| (x, z)));
    pave(set, path, &GRAVEL);
    liquid_tubes(set, &over_ground(set, &route(&LINE)));
    derrick(set);
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
