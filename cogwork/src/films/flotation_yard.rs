//! Section 4 (Tier 3): prospecting, steel excavation, and closed-loop ore flotation on Pad S4.
//! A core drill stands beside a Steel Excavator powered by the torque shaft from S3's steam
//! engine. Beyond it, creosote piped from S3's coke oven enters a frothing flotation cell standing
//! beside two turning clarifiers: one straight rear pipe carries muddy water east to both
//! clarifiers, and one straight low front pipe returns clear water west to the cell.

use showcase_api::{Film, Look, Set, dolly, flyby};
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

/// Core drill and Steel Excavator (frame x 47..57, z 24..34), with the excavator's output chest
/// tucked east of the frame away from the surface power shaft.
const CORE_DRILL_AT: Pos = Pos::new(44, FLOOR, 33);
pub const EXCAVATOR_AT: Pos = Pos::new(52, FLOOR, 35);
const EXCAVATOR_CHEST: Pos = Pos::new(58, FLOOR, 34);
const PIT_DEPTH: i32 = 3;

/// Flotation cell and two clarifiers along z = 32, with every chest and buffer tank tucked behind
/// them so nothing stands in the foreground or in the camera's path.
const CREOSOTE_TANK: Area = Area {
    lo: Pos::new(60, FLOOR, 29),
    hi: Pos::new(60, FLOOR + 1, 29),
};
const FLOTATION_AT: Pos = Pos::new(60, FLOOR, 32);
const FLOTATION_CHEST: Pos = Pos::new(59, FLOOR, 30);
const FLOTATION_ENGINES: [(Pos, Dir); 2] = [
    (Pos::new(59, FLOOR, 28), Dir::Down),
    (Pos::new(59, FLOOR, 29), Dir::Down),
];
const CLARIFIERS: [Pos; 2] = [Pos::new(63, FLOOR, 32), Pos::new(65, FLOOR, 32)];
const CLAY_CHESTS: [Pos; 2] = [Pos::new(63, FLOOR, 31), Pos::new(65, FLOOR, 31)];
const WATER_TANK: Area = Area {
    lo: Pos::new(67, FLOOR, 29),
    hi: Pos::new(69, FLOOR + 1, 30),
};

pub fn build(set: &Set) {
    set.place(
        CORE_DRILL_AT,
        CORE_DRILL.with(&[("facing", "south")]).into(),
    );
    steel_excavator(set);
    flotation_loop(set);
}

/// Steel Excavator powered via surface torque shaft from S3's steam engine (no open trenches)
/// with its output chest tucked east of the steel frame.
fn steel_excavator(set: &Set) {
    set.place(
        EXCAVATOR_AT,
        STEEL_EXCAVATOR.with(&[("facing", "south")]).into(),
    );
    shafts(
        set,
        Pos::new(43, FLOOR, SHAFT_Z),
        Pos::new(45, FLOOR, SHAFT_Z),
    );
    shafts(
        set,
        Pos::new(45, FLOOR, SHAFT_Z + 1),
        Pos::new(45, FLOOR, 35),
    );
    shafts(set, Pos::new(46, FLOOR, 35), Pos::new(51, FLOOR, 35));
    tubes(set, Pos::new(53, FLOOR, 35), Pos::new(58, FLOOR, 35));
    set.place(EXCAVATOR_CHEST, BlockState::of(&CHEST));

    let world = set.world();
    let pos = set.at(EXCAVATOR_AT);
    let frame = Frame::plan(&world, pos, Dir::South, &STEEL);
    build_frame(set, &frame, &STEEL_FRAME);
    let lo = frame.lo.add_x(1).add_z(1).with_y(pos.y - PIT_DEPTH);
    let hi = frame.hi.with_y(pos.y);
    world.fill(Shape::cuboid(lo, hi), BlockState::AIR.id());
}

/// Flotation cell and two clarifiers along z = 32 with every chest and buffer tank placed behind
/// them: muddy water branches east and north behind the clarifiers, clear water returns along
/// z = 34 with one-block stubs so muddy and clear lines never touch.
fn flotation_loop(set: &Set) {
    set.place(
        FLOTATION_AT,
        FLOTATION_CELL.with(&[("facing", "west")]).into(),
    );
    tubes(set, Pos::new(59, FLOOR, 31), Pos::new(59, FLOOR, 32));
    set.place(FLOTATION_CHEST, BlockState::of(&CHEST));

    // Power runs horizontally at FLOOR - 3 so solid dirt/stone supports every falling `base:gravel`
    // cell at FLOOR - 1, rising to FLOOR - 1 only beneath the two engines and flotation cell.
    shafts(
        set,
        Pos::new(59, FLOOR - 3, 28),
        Pos::new(59, FLOOR - 1, 28),
    );
    shafts(
        set,
        Pos::new(59, FLOOR - 3, 29),
        Pos::new(59, FLOOR - 1, 29),
    );
    shafts(
        set,
        Pos::new(59, FLOOR - 3, 28),
        Pos::new(59, FLOOR - 3, 29),
    );
    shafts(
        set,
        Pos::new(59, FLOOR - 3, 29),
        Pos::new(60, FLOOR - 3, 29),
    );
    shafts(
        set,
        Pos::new(60, FLOOR - 3, 29),
        Pos::new(60, FLOOR - 3, 32),
    );
    shafts(
        set,
        Pos::new(60, FLOOR - 3, 32),
        Pos::new(60, FLOOR - 1, 32),
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

/// Creosote leaves S3 at (43, FLOOR, 28), skirts cleanly around the north side of the Steel
/// Excavator's frame (x 47..57, z 24..34) along z = 23, and enters CREOSOTE_TANK at (60, 29).
fn creosote_line() -> Vec<Pos> {
    let mut blocks: Vec<Pos> = (43..=45).map(|x| Pos::new(x, FLOOR, CREOSOTE_Z)).collect();
    blocks.extend((23..CREOSOTE_Z).rev().map(|z| Pos::new(45, FLOOR, z)));
    blocks.extend((46..=60).map(|x| Pos::new(x, FLOOR, 23)));
    blocks.extend((24..=28).map(|z| Pos::new(60, FLOOR, z)));
    blocks.extend([Pos::new(60, FLOOR, 30), Pos::new(60, FLOOR, 31)]);
    blocks
}

/// Muddy water exits the flotation cell's east (back) face, loops behind the first clay chest,
/// and enters the shared junction at (64, FLOOR, 32) directly between both clarifiers so both
/// receive muddy water equally at the same distance.
fn muddy_line() -> Vec<Pos> {
    vec![
        Pos::new(61, FLOOR, 32),
        Pos::new(61, FLOOR, 31),
        Pos::new(61, FLOOR, 30),
        Pos::new(62, FLOOR, 30),
        Pos::new(63, FLOOR, 30),
        Pos::new(64, FLOOR, 30),
        Pos::new(64, FLOOR, 31),
        Pos::new(64, FLOOR, 32),
    ]
}

/// Clear water leaves both clarifiers' south fronts into a low return line along z = 34 (fed by
/// the rear water tank at the far east) and enters the flotation cell's south face.
fn clear_water_line() -> Vec<Pos> {
    let mut blocks: Vec<Pos> = (60..=67).map(|x| Pos::new(x, FLOOR, 34)).collect();
    blocks.extend([
        Pos::new(60, FLOOR, 33),
        Pos::new(63, FLOOR, 33),
        Pos::new(65, FLOOR, 33),
        Pos::new(67, FLOOR, 33),
        Pos::new(67, FLOOR, 32),
        Pos::new(67, FLOOR, 31),
    ]);
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
    // WATER_TANK is 3×2×2 (192,000 mB total; 96,000 mB on y = FLOOR). Prefilling [3, 8] puts
    // 72,000 mB into the bottom layer (75% full, 0% in FLOOR + 1), leaving 24,000 mB of free
    // headroom on y = FLOOR so both clarifiers pour freely while supplying ~390 s of net flotation
    // water (183 mB/s net loss) so FLOTATION_AT never runs low on camera.
    prefill(set, WATER_TANK.blocks(), "base:water", [3, 8]);
    prefill(set, [FLOTATION_AT].into_iter(), "base:water", [1, 1]);
    // Prefill muddy_line and both clarifiers (4,000 mB each; clarifiers settle 10 mB/t combined
    // against 8.33 mB/t produced, so both drain immediately and stay evenly settling past 2:20).
    prefill(
        set,
        muddy_line().into_iter().chain(CLARIFIERS),
        "cogwork:muddy_water",
        [1, 1],
    );
}

/// Glides across the open gravel past the core drill, Steel Excavator, flotation cell and
/// clarifiers before fading out Part 1.
pub fn shots(film: Film) -> Film {
    let f = FLOOR as f64;
    let core_look = lowered(above(CORE_DRILL_AT, 0.3));
    let steel_look = lowered([52.0, f + 2.0, 31.5]);
    let float_look = lowered(above(FLOTATION_AT, 0.2));
    let clarifier_look = lowered([64.0, f + 0.5, 32.5]);

    let at_core = lowered([44.0, f + 2.3, 37.2]);
    let at_steel = lowered([53.5, f + 5.2, 38.4]);
    let past_sw_corner = [at_core, lowered([48.2, f + 4.4, 38.2]), at_steel];
    let to_float = lowered([59.6, f + 3.2, 37.8]);
    let past_se_corner = [at_steel, lowered([58.2, f + 5.2, 38.4]), to_float];
    let at_float = lowered([61.5, f + 2.4, 37.4]);
    let at_clarifier = lowered([63.8, f + 2.3, 37.2]);
    let end_eye = lowered([65.0, f + 2.4, 37.2]);

    film.title("cogwork.film.t3_core", 3.8)
        .shot(
            dolly(foundry::END_EYE, at_core)
                .look(Look::Pan(foundry::END_LOOK, core_look))
                .ease(speeds(1.0, 0.9))
                .secs(4.2),
        )
        .title("cogwork.film.t3_steel", 5.5)
        .shot(
            flyby(&past_sw_corner)
                .look(Look::Pan(core_look, steel_look))
                .ease(speeds(0.9, 0.9))
                .secs(6.2),
        )
        .menu("cogwork:steel_excavator", lowered_block(EXCAVATOR_AT), 3.8)
        .shot(
            flyby(&past_se_corner)
                .look(Look::Pan(steel_look, float_look))
                .ease(speeds(0.9, 0.7))
                .secs(5.0),
        )
        .title("cogwork.film.t3_float", 3.2)
        .shot(
            dolly(to_float, at_float)
                .look_at(float_look)
                .ease(speeds(0.7, 0.7))
                .secs(3.5),
        )
        .menu("cogwork:flotation_cell", lowered_block(FLOTATION_AT), 3.8)
        .shot(
            dolly(at_float, at_clarifier)
                .look(Look::Pan(float_look, clarifier_look))
                .ease(speeds(0.7, 0.6))
                .secs(5.5),
        )
        .menu("cogwork:clarifier", lowered_block(CLARIFIERS[0]), 3.5)
        .shot(
            dolly(at_clarifier, end_eye)
                .look_at(clarifier_look)
                .ease(speeds(0.6, 0.3))
                .secs(4.2),
        )
        .fade_out(1.5)
}
