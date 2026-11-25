//! The north side of the hall: the diesel tank (fed from Pad R's refinery), the timber liquid
//! tube that lifts diesel out of it into the oil engine once its own engine is lit, the oil
//! engine (whose controller snaps shut on cue so the combustion chamber inside is glimpsed first),
//! and the water tank. The engine's drive is at its back: a torque shaft climbs from it and
//! crosses the hall under the roof to the benches.

use showcase_api::{Film, Look, Set, dolly, flyby, orbit};
use weft_sdk::Dir;

use super::camera::{on_orbit, speeds};
use super::parts::{engine, fuel, liquid_tubes, prefill, shafts};
use super::refinery_yard;
use super::signals;
use super::site::{above, lowered, lowered_block};
use crate::lib::{Area, BlockState, Pos};
use crate::oil_engine::{
    COMBUSTION_CHAMBER, OIL_ENGINE_CASING, OIL_ENGINE_CONTROLLER, OIL_ENGINE_DRIVE, OIL_INTAKE,
};
use crate::tank::LIQUID_TANK;
use crate::timber_liquid_tube::TIMBER_LIQUID_TUBE;

/// The row everything here stands along.
const ROW_Z: i32 = -5;
/// The diesel tank, a single column two high.
pub const OIL_TANK: Area = Area {
    lo: Pos::new(4, 0, ROW_Z),
    hi: Pos::new(4, 1, ROW_Z),
};
/// How full the diesel tank is when the film starts: half, so its level shows without spilling
/// into the intake before the timber liquid tube starts.
const OIL_FULL: [u32; 2] = [1, 2];
/// The water tank, two by two and three high.
pub const WATER_TANK: Area = Area {
    lo: Pos::new(12, 0, -6),
    hi: Pos::new(13, 2, ROW_Z),
};
/// The timber liquid tube against the diesel tank, and the engine on top of it.
const LIFT: Pos = Pos::new(5, 0, ROW_Z);
const LIFT_ENGINE: Pos = Pos::new(5, 1, ROW_Z);
/// From the timber liquid tube up to the engine's intake.
pub const OIL_RISE: [Pos; 3] = [
    Pos::new(6, 0, ROW_Z),
    Pos::new(7, 0, ROW_Z),
    Pos::new(7, 1, ROW_Z),
];
/// The oil engine: its cube, and the parts in it that are not casing. Its controller looks
/// south, across the hall.
const CUBE: Area = Area {
    lo: Pos::new(8, 0, -6),
    hi: Pos::new(10, 2, -4),
};
pub const CONTROLLER: Pos = Pos::new(9, 1, -4);
pub const CHAMBER: Pos = Pos::new(9, 1, ROW_Z);
const OIL_IN: Pos = Pos::new(8, 1, ROW_Z);
const WATER_IN: Pos = Pos::new(10, 0, ROW_Z);
const DRIVE: Pos = Pos::new(9, 1, -6);
/// Between the engine's water intake and the water tank.
const WATER_FEED: Pos = Pos::new(11, 0, ROW_Z);
/// The shaft: up behind the drive, across the hall directly under the x = 10 beam, and down the
/// inner face of the x = 10 south post into the benches' shaft.
const SHAFT_HEIGHT: i32 = 5;
pub const SHAFT_X: i32 = 10;
const SHAFT_BACK: i32 = -7;

/// Camera landmarks inside the hall, and where the haze shot hands off to the benches (clear of
/// the hall posts, the oil engine casing and the overhead x = 10 shaft).
const IN_HALL: [f64; 3] = lowered([1.0, 2.6, 1.2]);
const BY_TANK: [f64; 3] = lowered([4.6, 2.1, 0.2]);
const ENGINE_OUT: f64 = 5.5;
const ENGINE_FROM: f64 = 130.0;
const IN_HAZE: [[f64; 3]; 2] = [lowered([13.0, 4.1, -0.2]), lowered([13.2, 4.0, 0.2])];
pub const END_EYE: [f64; 3] = IN_HAZE[1];
pub const END_LOOK: [f64; 3] = lowered([6.5, 2.8, -8.5]);

const OIL_TANK_LOOK: [f64; 3] = lowered(above(OIL_TANK.lo, 0.7));
const OIL_RISE_LOOK: [f64; 3] = lowered(above(OIL_RISE[1], 0.5));
const OIL_ENGINE_LOOK: [f64; 3] = lowered(above(CHAMBER, 0.3));

pub fn build(set: &Set) {
    for tank in [OIL_TANK, WATER_TANK] {
        set.fill(tank, BlockState::of(&LIQUID_TANK));
    }
    set.place(LIFT, BlockState::of(&TIMBER_LIQUID_TUBE));
    engine(set, LIFT_ENGINE, Dir::Down);
    liquid_tubes(set, &OIL_RISE);
    liquid_tubes(set, &[WATER_FEED]);
    oil_engine(set);
    // Up behind DRIVE at x = 9, then step onto x = 10 directly beneath the oak cross-beam and drop
    // down the north face of the x = 10 south post (z = 6) into the bench shaft (z = 5).
    shafts(
        set,
        Pos::new(DRIVE.x, 1, SHAFT_BACK),
        Pos::new(DRIVE.x, SHAFT_HEIGHT, SHAFT_BACK),
    );
    shafts(
        set,
        Pos::new(DRIVE.x, SHAFT_HEIGHT, SHAFT_BACK + 1),
        Pos::new(SHAFT_X, SHAFT_HEIGHT, SHAFT_BACK + 1),
    );
    shafts(
        set,
        Pos::new(SHAFT_X, SHAFT_HEIGHT, SHAFT_BACK + 2),
        Pos::new(SHAFT_X, SHAFT_HEIGHT, 6),
    );
    shafts(
        set,
        Pos::new(SHAFT_X, 0, 6),
        Pos::new(SHAFT_X, SHAFT_HEIGHT - 1, 6),
    );
}

/// The oil engine cube, leaving its controller slot open until cue `"lift"` so the combustion
/// chamber inside is briefly visible as the camera enters the hall.
fn oil_engine(set: &Set) {
    set.fill(CUBE, BlockState::of(&OIL_ENGINE_CASING));
    set.place(CHAMBER, BlockState::of(&COMBUSTION_CHAMBER));
    for intake in [OIL_IN, WATER_IN] {
        set.place(intake, BlockState::of(&OIL_INTAKE));
    }
    set.place(DRIVE, BlockState::of(&OIL_ENGINE_DRIVE));
    set.place(CONTROLLER, BlockState::AIR);
}

/// Puts the stock of diesel into its tank.
pub fn stock_oil(set: &Set) {
    prefill(set, OIL_TANK.blocks(), "cogwork:diesel", OIL_FULL);
}

/// Places the controller (sealing the combustion chamber inside the riveted panel) and lights the
/// timber liquid tube's engine so diesel rises into the oil engine.
pub fn lift(set: &Set) {
    let controller = OIL_ENGINE_CONTROLLER.with(&[("facing", "south")]);
    set.place(CONTROLLER, controller.into());
    fuel(set, LIFT_ENGINE);
}

/// Enters the hall from Pad R, glimpses the combustion chamber, seals the controller as diesel
/// rises, orbits the running oil engine, and rises into its warm exhaust haze.
pub fn shots(film: Film) -> Film {
    let enter = [refinery_yard::end_eye(), lowered([-3.0, 3.2, 0.2]), IN_HALL];
    let at_engine = on_orbit(OIL_ENGINE_LOOK, ENGINE_OUT, 0.8, ENGINE_FROM);
    let from_engine = on_orbit(OIL_ENGINE_LOOK, ENGINE_OUT, 2.4, 40.0);
    let round = |from: f64, to: f64, up: [f64; 2], secs: f32| {
        orbit(OIL_ENGINE_LOOK, ENGINE_OUT)
            .arc(from, to)
            .height(up[0], up[1])
            .ease(speeds(0.7, 0.7))
            .secs(secs)
    };
    film.shot(
        flyby(&enter)
            .look(Look::Pan(refinery_yard::ORBIT_CENTER, OIL_TANK_LOOK))
            .ease(speeds(0.9, 0.8))
            .secs(5.5),
    )
    .shot(
        dolly(IN_HALL, BY_TANK)
            .look(Look::Pan(OIL_TANK_LOOK, OIL_ENGINE_LOOK))
            .ease(speeds(0.8, 0.4))
            .secs(4.0),
    )
    .cue("lift")
    .title("cogwork.film.lift", 5.0)
    .shot(
        dolly(BY_TANK, at_engine)
            .look(Look::Pan(OIL_ENGINE_LOOK, OIL_RISE_LOOK))
            .ease(speeds(0.4, 0.7))
            .secs(5.0),
    )
    .wait_for(signals::RUNNING, signals::WAIT_SECS)
    .title("cogwork.film.oil_engine", 5.0)
    .shot(round(ENGINE_FROM, 75.0, [0.8, 2.0], 7.5).blend(1.0))
    .menu("cogwork:oil_engine", lowered_block(CONTROLLER), 5.0)
    .shot(round(75.0, 40.0, [2.0, 2.4], 5.0))
    .title("cogwork.film.haze", 6.0)
    .shot(
        dolly(from_engine, IN_HAZE[0])
            .look(Look::Pan(OIL_ENGINE_LOOK, END_LOOK))
            .ease(speeds(0.7, 0.15))
            .secs(4.0),
    )
    .shot(
        dolly(IN_HAZE[0], IN_HAZE[1])
            .look_at(END_LOOK)
            .ease(speeds(0.15, 0.15))
            .secs(2.5),
    )
}
