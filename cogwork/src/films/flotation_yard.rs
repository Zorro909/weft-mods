//! Section 4 (Tier 3): prospecting, steel excavation, and closed-loop ore flotation on Pad S4.
//! A core drill stands beside a Steel Excavator powered by the torque shaft from S3's steam
//! engine. Beyond it, creosote piped from S3's coke oven enters a frothing flotation cell standing
//! beside two turning clarifiers: one straight rear pipe carries muddy water east to both
//! clarifiers, and one straight low front pipe returns clear water west to the cell.

use showcase_api::{Film, Look, Set, dolly};
use weft_sdk::{Dir, Shape};

use super::camera::speeds;
use super::foundry::{self, CREOSOTE_Z, SHAFT_Z};
use super::parts::{
    CHEST, build_frame, engine, fit_bit, fuel, liquid_tubes, prefill, shafts, start_quarry,
    stock_rubble, tubes,
};
use super::site::{VALLEY_TOP, above, lowered, lowered_block};
use crate::clarifier::CLARIFIER;
use crate::core_drill::CORE_DRILL;
use crate::excavator::kind::rows::STEEL;
use crate::excavator::{Frame, STEEL_EXCAVATOR, STEEL_FRAME};
use crate::flotation::FLOTATION_CELL;
use crate::lib::{Area, BlockState, Pos};
use crate::rubble::Grade;
use crate::tank::LIQUID_TANK;

/// Floor height above Pad S4's gravel, in blocks from the works.
const FLOOR: i32 = VALLEY_TOP + 1;

/// Core drill and Steel Excavator.
const CORE_DRILL_AT: Pos = Pos::new(45, FLOOR, 35);
pub const EXCAVATOR_AT: Pos = Pos::new(52, FLOOR, 35);
const EXCAVATOR_CHEST: Pos = Pos::new(54, FLOOR, 35);
const PIT_DEPTH: i32 = 3;

/// Straight-line flotation and clarifier loop along z = 34, all facing south with tanks and chests
/// inline or to the side so nothing stands between the camera and the machines.
const CREOSOTE_TANK: Area = Area {
    lo: Pos::new(59, FLOOR, 33),
    hi: Pos::new(59, FLOOR + 1, 33),
};
const FLOTATION_AT: Pos = Pos::new(61, FLOOR, 34);
const FLOTATION_CHEST: Pos = Pos::new(60, FLOOR, 35);
const FLOTATION_ENGINES: [(Pos, Dir); 2] = [
    (Pos::new(60, FLOOR, 32), Dir::Down),
    (Pos::new(61, FLOOR, 32), Dir::Down),
];
const CLARIFIERS: [Pos; 2] = [Pos::new(63, FLOOR, 34), Pos::new(65, FLOOR, 34)];
const CLAY_CHESTS: [Pos; 2] = [Pos::new(64, FLOOR, 34), Pos::new(66, FLOOR, 34)];
const WATER_TANK: Area = Area {
    lo: Pos::new(67, FLOOR, 34),
    hi: Pos::new(67, FLOOR + 1, 34),
};

/// Where S3's last shot hands the camera off to S4.
const START_EYE: [f64; 3] = lowered([43.5, FLOOR as f64 + 2.2, 38.0]);

pub fn build(set: &Set) {
    set.place(
        CORE_DRILL_AT,
        CORE_DRILL.with(&[("facing", "south")]).into(),
    );
    steel_excavator(set);
    flotation_loop(set);
}

/// Steel Excavator powered via torque shaft from S3's steam engine, with pre-built steel frame.
fn steel_excavator(set: &Set) {
    set.place(
        EXCAVATOR_AT,
        STEEL_EXCAVATOR.with(&[("facing", "south")]).into(),
    );
    shafts(
        set,
        Pos::new(43, FLOOR, SHAFT_Z),
        Pos::new(47, FLOOR, SHAFT_Z),
    );
    shafts(
        set,
        Pos::new(47, FLOOR, SHAFT_Z + 1),
        Pos::new(47, FLOOR, 35),
    );
    shafts(set, Pos::new(48, FLOOR, 35), Pos::new(51, FLOOR, 35));
    tubes(set, Pos::new(53, FLOOR, 35), Pos::new(53, FLOOR, 35));
    set.place(EXCAVATOR_CHEST, BlockState::of(&CHEST));

    let world = set.world();
    let pos = set.at(EXCAVATOR_AT);
    let frame = Frame::plan(&world, pos, Dir::South, &STEEL);
    build_frame(set, &frame, &STEEL_FRAME);
    let lo = frame.lo.add_x(1).add_z(1).with_y(pos.y - PIT_DEPTH);
    let hi = frame.hi.with_y(pos.y);
    world.fill(Shape::cuboid(lo, hi), BlockState::AIR.id());
}

/// Side-by-side flotation cell and two clarifiers linked by two straight parallel pipe runs:
/// muddy water behind along z = 33, clear water returning low in front along z = 35.
fn flotation_loop(set: &Set) {
    set.place(
        FLOTATION_AT,
        FLOTATION_CELL.with(&[("facing", "south")]).into(),
    );
    tubes(set, Pos::new(61, FLOOR, 35), Pos::new(61, FLOOR, 35));
    set.place(FLOTATION_CHEST, BlockState::of(&CHEST));

    // Power comes under the gravel from two ember engines tucked behind the cell.
    shafts(
        set,
        Pos::new(60, FLOOR - 1, 32),
        Pos::new(61, FLOOR - 1, 32),
    );
    shafts(
        set,
        Pos::new(61, FLOOR - 1, 33),
        Pos::new(61, FLOOR - 1, 34),
    );
    for (at, toward) in FLOTATION_ENGINES {
        engine(set, at, toward);
    }

    set.fill(CREOSOTE_TANK, BlockState::of(&LIQUID_TANK));
    liquid_tubes(set, &creosote_line());

    for at in CLARIFIERS {
        set.place(at, CLARIFIER.with(&[("facing", "south")]).into());
    }
    for at in CLAY_CHESTS {
        set.place(at, BlockState::of(&CHEST));
    }
    set.fill(WATER_TANK, BlockState::of(&LIQUID_TANK));
    liquid_tubes(set, &muddy_line());
    liquid_tubes(set, &clear_water_line());
}

fn creosote_line() -> Vec<Pos> {
    let mut blocks: Vec<Pos> = (43..=59).map(|x| Pos::new(x, FLOOR, CREOSOTE_Z)).collect();
    blocks.extend((CREOSOTE_Z + 1..=32).map(|z| Pos::new(59, FLOOR, z)));
    blocks.extend([Pos::new(59, FLOOR, 34), Pos::new(60, FLOOR, 34)]);
    blocks
}

/// One straight muddy water header behind the flotation cell and both clarifiers along z = 33.
fn muddy_line() -> Vec<Pos> {
    (61..=65).map(|x| Pos::new(x, FLOOR, 33)).collect()
}

/// One straight clear water return along z = 35 from the water tank and both clarifiers back into
/// the flotation cell's east side at (62, FLOOR, 34).
fn clear_water_line() -> Vec<Pos> {
    let mut blocks: Vec<Pos> = (62..=67).map(|x| Pos::new(x, FLOOR, 35)).collect();
    blocks.push(Pos::new(62, FLOOR, 34));
    blocks
}

/// Starts the Steel Excavator, fuels the flotation engines, and prefills creosote, water and
/// muddy water so both flotation and clarification are in full swing.
pub fn start(set: &Set) {
    fit_bit(set, EXCAVATOR_AT, "cogwork:steel_bit", 0);
    start_quarry(set, EXCAVATOR_AT, Dir::South, &STEEL, FLOOR - PIT_DEPTH - 1);

    for (at, _) in FLOTATION_ENGINES {
        fuel(set, at);
    }
    stock_rubble(set, FLOTATION_AT, "input", "iron", Grade::Common, 3);
    prefill(
        set,
        CREOSOTE_TANK.blocks().chain(creosote_line()),
        "cogwork:creosote",
        [1, 1],
    );
    prefill(
        set,
        WATER_TANK
            .blocks()
            .chain(clear_water_line())
            .chain([FLOTATION_AT]),
        "base:water",
        [1, 1],
    );
    prefill(
        set,
        muddy_line().into_iter().chain(CLARIFIERS),
        "cogwork:muddy_water",
        [1, 1],
    );
}

/// Glides close past the core drill, Steel Excavator, flotation cell and clarifiers before fading
/// out Part 1.
pub fn shots(film: Film) -> Film {
    let f = FLOOR as f64;
    let core_look = lowered(above(CORE_DRILL_AT, 0.2));
    let steel_look = lowered([52.0, f + 1.8, 32.0]);
    let float_look = lowered(above(FLOTATION_AT, 0.2));
    let clarifier_look = lowered([64.5, f + 0.5, 34.0]);

    let at_core = lowered([47.0, f + 2.0, 38.2]);
    let at_steel = lowered([55.2, f + 3.4, 39.2]);
    let at_float = lowered([61.5, f + 2.0, 38.2]);
    let at_clarifier = lowered([65.0, f + 2.1, 38.0]);
    let end_eye = lowered([67.2, f + 2.2, 37.8]);

    film.title("cogwork.film.t3_core", 4.0)
        .shot(
            dolly(START_EYE, at_core)
                .look(Look::Pan(foundry::END_LOOK, core_look))
                .ease(speeds(1.0, 0.9))
                .secs(4.5),
        )
        .title("cogwork.film.t3_steel", 5.5)
        .shot(
            dolly(at_core, at_steel)
                .look(Look::Pan(core_look, steel_look))
                .ease(speeds(0.9, 0.9))
                .secs(6.5),
        )
        .menu("cogwork:steel_excavator", lowered_block(EXCAVATOR_AT), 4.0)
        .shot(
            dolly(at_steel, at_float)
                .look(Look::Pan(steel_look, float_look))
                .ease(speeds(0.9, 0.7))
                .secs(7.0),
        )
        .title("cogwork.film.t3_float", 6.0)
        .menu("cogwork:flotation_cell", lowered_block(FLOTATION_AT), 4.0)
        .shot(
            dolly(at_float, at_clarifier)
                .look(Look::Pan(float_look, clarifier_look))
                .ease(speeds(0.7, 0.6))
                .secs(6.5),
        )
        .menu("cogwork:clarifier", lowered_block(CLARIFIERS[0]), 3.5)
        .shot(
            dolly(at_clarifier, end_eye)
                .look_at(clarifier_look)
                .ease(speeds(0.6, 0.3))
                .secs(4.5),
        )
        .fade_out(1.5)
}
