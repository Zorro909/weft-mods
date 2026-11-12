//! The north side of the hall: the oil tank, which holds some oil from the start and which
//! the line from the seep fills, the timber liquid tube that lifts oil out of it into the oil
//! engine once its own engine is lit, the oil engine, and the water tank, which the line from
//! the lake fills and the engine draws from. The engine's drive is at its back: a torque shaft
//! climbs from it and crosses the hall under the roof to the benches.

use showcase_api::Set;
use weft_sdk::Dir;

use super::parts::{engine, fuel, liquid_tubes, prefill, shafts};
use crate::lib::{Area, BlockState, Pos};
use crate::oil_engine::{
    COMBUSTION_CHAMBER, OIL_ENGINE_CASING, OIL_ENGINE_CONTROLLER, OIL_ENGINE_DRIVE, OIL_INTAKE,
};
use crate::tank::LIQUID_TANK;
use crate::timber_liquid_tube::TIMBER_LIQUID_TUBE;

/// The row everything here stands along.
const ROW_Z: i32 = -5;
/// The oil tank, a single column two high.
pub const OIL_TANK: Area = Area {
    lo: Pos::new(4, 0, ROW_Z),
    hi: Pos::new(4, 1, ROW_Z),
};
/// How full the oil tank is when the film starts: half, so that its level shows and the
/// engine starts as soon as the oil is lifted to it. Any fuller and the oil stands higher than
/// the engine's intake, and runs into it before the timber liquid tube is started.
const OIL_FULL: [u32; 2] = [1, 2];
/// The water tank, two by two and three high.
pub const WATER_TANK: Area = Area {
    lo: Pos::new(12, 0, -6),
    hi: Pos::new(13, 2, ROW_Z),
};
/// The timber liquid tube against the oil tank, and the engine on top of it.
const LIFT: Pos = Pos::new(5, 0, ROW_Z);
const LIFT_ENGINE: Pos = Pos::new(5, 1, ROW_Z);
/// From the timber liquid tube up to the engine's oil intake.
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
/// The shaft: up behind the drive, and across the hall to the row the benches' shaft runs in.
pub const SHAFT_HEIGHT: i32 = 5;
pub const SHAFT_X: i32 = 9;
const SHAFT_BACK: i32 = -7;

pub fn build(set: &Set) {
    for tank in [OIL_TANK, WATER_TANK] {
        set.fill(tank, BlockState::of(&LIQUID_TANK));
    }
    set.place(LIFT, BlockState::of(&TIMBER_LIQUID_TUBE));
    engine(set, LIFT_ENGINE, Dir::Down);
    liquid_tubes(set, &OIL_RISE);
    liquid_tubes(set, &[WATER_FEED]);
    oil_engine(set);
    let back = |y: i32| Pos::new(SHAFT_X, y, SHAFT_BACK);
    shafts(set, back(1), back(SHAFT_HEIGHT));
    let across = |z: i32| Pos::new(SHAFT_X, SHAFT_HEIGHT, z);
    shafts(
        set,
        across(SHAFT_BACK + 1),
        across(super::crafting::SHAFT_Z),
    );
    let down = |y: i32| Pos::new(SHAFT_X, y, super::crafting::SHAFT_Z);
    shafts(set, down(1), down(SHAFT_HEIGHT - 1));
}

/// The 27 blocks of the oil engine.
fn oil_engine(set: &Set) {
    set.fill(CUBE, BlockState::of(&OIL_ENGINE_CASING));
    set.place(CHAMBER, BlockState::of(&COMBUSTION_CHAMBER));
    for intake in [OIL_IN, WATER_IN] {
        set.place(intake, BlockState::of(&OIL_INTAKE));
    }
    set.place(DRIVE, BlockState::of(&OIL_ENGINE_DRIVE));
    let controller = OIL_ENGINE_CONTROLLER.with(&[("facing", "south")]);
    set.place(CONTROLLER, controller.into());
}

/// Puts the stock of oil into its tank.
pub fn stock_oil(set: &Set) {
    prefill(set, OIL_TANK.blocks(), "cogwork:oil", OIL_FULL);
}

/// Lights the engine on the timber liquid tube: oil goes up into the oil engine, which starts.
pub fn lift(set: &Set) {
    fuel(set, LIFT_ENGINE);
}
