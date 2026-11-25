//! Section 2 (Tier 1): the iron yard on Pad S2. Ember engines drive an Iron Quarry inside its
//! riveted iron frame, sending output through gilded tubes. Beside it under a back-posted timber
//! awning runs one continuous driven ore line: a shaker screen with a void tube for waste, linked
//! by stone tubes into a washer, a grinding mill, and a furnace blown by a driven bellows.

use showcase_api::{Film, Look, Set, dolly, flyby};
use weft_sdk::{Dir, Shape};

use super::camera::speeds;
use super::parts::{
    CHEST, FURNACE, OAK_LOG, OAK_SLAB, area, build_frame, engine, fit_bit, fuel, liquid_tubes,
    prefill, shafts, spring_engine, start_quarry, stock, stock_group, stock_rubble, tubes,
    wind_spring,
};
use super::site::{VALLEY_TOP, above, lowered, lowered_block};
use super::timber_camp;
use crate::bellows::DRIVEN_BELLOWS;
use crate::excavator::kind::rows::IRON;
use crate::excavator::{Frame, IRON_FRAME, IRON_QUARRY};
use crate::grinding_mill::GRINDING_MILL;
use crate::lib::{Area, BlockState, Pos};
use crate::rubble::Grade;
use crate::shaker_screen::SHAKER_SCREEN;
use crate::tank::LIQUID_TANK;
use crate::tube::{GILDED_TUBE, TIMBER_TUBE, VOID_TUBE};
use crate::washer::WASHER;

/// Floor height above Pad S2's gravel, in blocks from the works.
const FLOOR: i32 = VALLEY_TOP + 1;

/// Iron Quarry, its 3 ember engines on a short torque shaft, and its gilded output line.
pub const QUARRY_AT: Pos = Pos::new(-6, FLOOR, 35);
const QUARRY_ENGINES: [(Pos, Dir); 3] = [
    (Pos::new(-9, FLOOR, 35), Dir::East),
    (Pos::new(-8, FLOOR + 1, 35), Dir::Down),
    (Pos::new(-7, FLOOR + 1, 35), Dir::Down),
];
const EMBER_MENU_AT: Pos = Pos::new(-8, FLOOR + 1, 35);
/// Output chest outside the Iron Quarry's frame (x -11..-1, z 24..34), on x = 0.
const QUARRY_CHEST: Pos = Pos::new(0, FLOOR, 33);
const PIT_DEPTH: i32 = 3;

/// Connected driven ore processing line along z = 32, with separate tube networks for each stage
/// so washed ore never reaches the void tube and raw ore overflows cleanly into a buffer chest.
const SHAKER_IN: Pos = Pos::new(1, FLOOR, 32);
const SHAKER_PULL: Pos = Pos::new(2, FLOOR, 32);
const SHAKER_PULL_ENGINE: Pos = Pos::new(2, FLOOR, 31);
const SHAKER_AT: Pos = Pos::new(3, FLOOR, 32);
const VOID_AT: Pos = Pos::new(3, FLOOR, 30);

const WASHER_AT: Pos = Pos::new(6, FLOOR, 32);
const MILL_AT: Pos = Pos::new(9, FLOOR, 32);
const FURNACE_BUFFER: Pos = Pos::new(10, FLOOR, 31);
const FURNACE_AT: Pos = Pos::new(11, FLOOR, 32);
const BELLOWS_AT: Pos = Pos::new(12, FLOOR, 32);

/// Water tank tucked behind the washer and grinding mill.
const WATER_TANK: Area = Area {
    lo: Pos::new(7, FLOOR, 29),
    hi: Pos::new(8, FLOOR + 1, 29),
};

/// Ember engines driving the bellows directly behind it and the shaker, washer and grinding mill
/// via shafts buried ONLY directly under engines, tubes and machines (no open trenches).
const DIRECT_ENGINES: [(Pos, Dir); 1] = [(Pos::new(12, FLOOR, 31), Dir::South)];
const UNDER_ENGINES: [(Pos, Pos); 3] = [
    (Pos::new(3, FLOOR, 29), SHAKER_AT),
    (Pos::new(6, FLOOR, 30), WASHER_AT),
    (Pos::new(9, FLOOR, 28), MILL_AT),
];
/// Assisting spring engine beside the grinding mill's ember engine (40 + 10 = 50 E/t against the
/// mill's 40 E/t draw, keeping its store full without overheating during the section).
const MILL_ASSIST: Pos = Pos::new(8, FLOOR, 28);

/// Where S2 hands the camera off to S3, heading east.
pub const END_EYE: [f64; 3] = lowered([14.2, FLOOR as f64 + 2.0, 36.2]);
pub const END_LOOK: [f64; 3] = lowered([11.8, FLOOR as f64 + 0.5, 32.5]);

pub fn build(set: &Set) {
    iron_quarry(set);
    shed(set);
    processing_line(set);
}

/// Iron Quarry with pre-built iron frame, 3 ember engines, and gilded tube run to its chest.
fn iron_quarry(set: &Set) {
    set.place(QUARRY_AT, IRON_QUARRY.with(&[("facing", "south")]).into());
    shafts(set, Pos::new(-8, FLOOR, 35), Pos::new(-7, FLOOR, 35));
    for (at, toward) in QUARRY_ENGINES {
        engine(set, at, toward);
    }
    tubes(set, Pos::new(-5, FLOOR, 35), Pos::new(-5, FLOOR, 35));
    set.fill(
        area(Pos::new(-4, FLOOR, 35), Pos::new(0, FLOOR, 35)),
        BlockState::of(&GILDED_TUBE),
    );
    set.place(Pos::new(0, FLOOR, 34), BlockState::of(&GILDED_TUBE));
    set.place(QUARRY_CHEST, BlockState::of(&CHEST));
    let world = set.world();
    let pos = set.at(QUARRY_AT);
    let frame = Frame::plan(&world, pos, Dir::South, &IRON);
    build_frame(set, &frame, &IRON_FRAME);
    let lo = frame.lo.add_x(1).add_z(1).with_y(pos.y - PIT_DEPTH);
    let hi = frame.hi.with_y(pos.y);
    world.fill(Shape::cuboid(lo, hi), BlockState::AIR.id());
}

/// Cantilevered timber awning supported only by end posts behind the line (x = 0 and x = 14 on
/// z = 28) so no post stands mid-frame or between the camera and the machines.
fn shed(set: &Set) {
    let roof_y = FLOOR + 4;
    for x in [0, 14] {
        set.fill(
            area(Pos::new(x, FLOOR, 28), Pos::new(x, roof_y - 1, 28)),
            BlockState::of(&OAK_LOG),
        );
    }
    set.fill(
        area(Pos::new(0, roof_y, 28), Pos::new(14, roof_y, 33)),
        OAK_SLAB.with(&[("type", "bottom")]).into(),
    );
}

/// Continuous tube-linked processing line along z = 32: shaker + rear void tube -> washer ->
/// grinding mill -> furnace + driven bellows, with item tubes, water and power all behind.
fn processing_line(set: &Set) {
    set.place(SHAKER_IN, BlockState::of(&CHEST));
    set.place(SHAKER_PULL, BlockState::of(&TIMBER_TUBE));
    spring_engine(set, SHAKER_PULL_ENGINE, Dir::South);
    set.place(SHAKER_AT, SHAKER_SCREEN.with(&[("facing", "north")]).into());
    set.place(VOID_AT, BlockState::of(&VOID_TUBE));
    // Shaker output network (stops at x = 4 on z = 31 so (5, 31) stays air and never touches
    // the washer's output network at (6..8, 31)).
    tubes(set, Pos::new(3, FLOOR, 31), Pos::new(4, FLOOR, 31));
    tubes(set, Pos::new(4, FLOOR, 32), Pos::new(5, FLOOR, 32));

    set.place(WASHER_AT, WASHER.with(&[("facing", "north")]).into());
    tubes(set, Pos::new(6, FLOOR, 31), Pos::new(8, FLOOR, 31));
    tubes(set, Pos::new(8, FLOOR, 32), Pos::new(8, FLOOR, 32));

    set.place(MILL_AT, GRINDING_MILL.with(&[("facing", "east")]).into());
    tubes(set, Pos::new(10, FLOOR, 32), Pos::new(10, FLOOR, 32));
    set.place(FURNACE_BUFFER, BlockState::of(&CHEST));

    set.place(FURNACE_AT, FURNACE.with(&[("facing", "south")]).into());
    set.place(
        BELLOWS_AT,
        DRIVEN_BELLOWS.with(&[("facing", "west")]).into(),
    );

    set.fill(WATER_TANK, BlockState::of(&LIQUID_TANK));
    liquid_tubes(set, &water_line());

    for (at, toward) in DIRECT_ENGINES {
        engine(set, at, toward);
    }
    for (at, target) in UNDER_ENGINES {
        engine(set, at, Dir::Down);
        shafts(
            set,
            Pos::new(at.x, FLOOR - 1, at.z),
            Pos::new(at.x, FLOOR - 1, target.z),
        );
    }
    spring_engine(set, MILL_ASSIST, Dir::Down);
    shafts(
        set,
        Pos::new(MILL_ASSIST.x, FLOOR - 1, MILL_ASSIST.z),
        Pos::new(MILL_AT.x, FLOOR - 1, MILL_ASSIST.z),
    );
}

/// Water from the rear tank bridges over the washer's rear item tube into its east side, and runs
/// straight into the grinding mill's north side.
fn water_line() -> Vec<Pos> {
    vec![
        Pos::new(7, FLOOR + 1, 30),
        Pos::new(7, FLOOR + 1, 31),
        Pos::new(7, FLOOR + 1, 32),
        Pos::new(7, FLOOR, 32),
        Pos::new(9, FLOOR, 29),
        Pos::new(9, FLOOR, 30),
        Pos::new(9, FLOOR, 31),
    ]
}

/// Fuels every engine, prefills the water tank, stocks rubble and ore rubble, and starts the
/// Iron Quarry.
pub fn start(set: &Set) {
    for at in QUARRY_ENGINES
        .into_iter()
        .map(|(at, _)| at)
        .chain(DIRECT_ENGINES.into_iter().map(|(at, _)| at))
        .chain(UNDER_ENGINES.into_iter().map(|(at, _)| at))
    {
        fuel(set, at);
    }
    wind_spring(set, SHAKER_PULL_ENGINE, 50_000);
    wind_spring(set, MILL_ASSIST, 50_000);
    fit_bit(set, QUARRY_AT, "cogwork:iron_bit", 0);
    start_quarry(set, QUARRY_AT, Dir::South, &IRON, FLOOR - PIT_DEPTH - 1);

    // Light input stocking so feed tubes never overflow and drop items on the gravel.
    stock(set, SHAKER_IN, &["cogwork:rubble"], 1);
    stock_group(set, SHAKER_AT, "input", &[("cogwork:rubble", 16)]);
    stock_rubble(set, WASHER_AT, "input", "iron", Grade::Poor, 1);
    stock_rubble(set, MILL_AT, "input", "iron", Grade::Common, 1);
    stock_group(set, MILL_AT, "sand", &[("base:sand", 64)]);
    fuel(set, FURNACE_AT);
    stock_group(set, FURNACE_AT, "input", &[("base:raw_iron", 16)]);

    let water_blocks = WATER_TANK
        .blocks()
        .chain(water_line())
        .chain([WASHER_AT, MILL_AT]);
    prefill(set, water_blocks, "base:water", [1, 1]);
}

/// Glide east above the Iron Quarry's top beam and past the driven ore line, timing every title
/// cleanly before its machine screen opens.
pub fn shots(film: Film) -> Film {
    let f = FLOOR as f64;
    let quarry_look = lowered([-6.0, f + 1.5, 32.5]);
    let shaker_look = lowered(above(SHAKER_AT, 0.3));
    let washer_look = lowered(above(WASHER_AT, 0.3));
    let mill_look = lowered(above(MILL_AT, 0.3));
    let bellows_look = lowered([11.8, f + 0.5, 32.5]);

    let leave_camp = lowered([-12.5, f + 8.6, 38.0]);
    let at_engines = lowered([-9.2, f + 5.4, 38.2]);
    let over_slope = [leave_camp, lowered([-10.6, f + 7.2, 38.2]), at_engines];
    let at_quarry = lowered([-3.5, f + 5.2, 38.2]);
    let past_quarry = lowered([0.8, f + 2.8, 37.2]);
    let to_shaker = lowered([2.4, f + 2.4, 37.0]);
    let at_shaker = lowered([4.0, f + 2.3, 37.0]);
    let to_washer = lowered([5.5, f + 2.3, 37.0]);
    let at_washer = lowered([7.2, f + 2.3, 37.0]);
    let at_mill = lowered([9.6, f + 2.2, 36.6]);
    let to_bellows = lowered([11.4, f + 1.9, 35.8]);
    let at_bellows = lowered([12.8, f + 1.8, 35.6]);

    film.shot(
        dolly(timber_camp::END_EYE, leave_camp)
            .look(Look::Pan(timber_camp::END_LOOK, quarry_look))
            .ease(speeds(1.2, 1.3))
            .secs(3.4),
    )
    .title("cogwork.film.t1_iron", 3.2)
    .shot(
        flyby(&over_slope)
            .look_at(quarry_look)
            .ease(speeds(1.3, 0.9))
            .secs(3.5),
    )
    .menu("cogwork:ember_engine", lowered_block(EMBER_MENU_AT), 3.5)
    .shot(
        dolly(at_engines, at_quarry)
            .look_at(quarry_look)
            .ease(speeds(0.9, 0.9))
            .secs(4.5),
    )
    .menu("cogwork:iron_quarry", lowered_block(QUARRY_AT), 3.5)
    .shot(
        dolly(at_quarry, past_quarry)
            .look(Look::Pan(quarry_look, shaker_look))
            .ease(speeds(0.9, 0.8))
            .secs(4.5),
    )
    .title("cogwork.film.t1_sift", 3.0)
    .shot(
        dolly(past_quarry, to_shaker)
            .look_at(shaker_look)
            .ease(speeds(0.8, 0.7))
            .secs(3.2),
    )
    .menu("cogwork:shaker_screen", lowered_block(SHAKER_AT), 3.5)
    .shot(
        dolly(to_shaker, at_shaker)
            .look(Look::Pan(shaker_look, washer_look))
            .ease(speeds(0.7, 0.7))
            .secs(4.0),
    )
    .title("cogwork.film.t1_wash", 3.0)
    .shot(
        dolly(at_shaker, to_washer)
            .look_at(washer_look)
            .ease(speeds(0.7, 0.7))
            .secs(3.2),
    )
    .menu("cogwork:washer", lowered_block(WASHER_AT), 3.5)
    .shot(
        dolly(to_washer, at_washer)
            .look(Look::Pan(washer_look, mill_look))
            .ease(speeds(0.7, 0.7))
            .secs(4.0),
    )
    .menu("cogwork:grinding_mill", lowered_block(MILL_AT), 3.5)
    .shot(
        dolly(at_washer, at_mill)
            .look(Look::Pan(mill_look, bellows_look))
            .ease(speeds(0.7, 0.7))
            .secs(4.2),
    )
    .title("cogwork.film.t1_bellows", 3.0)
    .shot(
        dolly(at_mill, to_bellows)
            .look_at(bellows_look)
            .ease(speeds(0.7, 0.6))
            .secs(3.2),
    )
    .menu("cogwork:driven_bellows", lowered_block(BELLOWS_AT), 3.5)
    .shot(
        dolly(to_bellows, at_bellows)
            .look_at(bellows_look)
            .ease(speeds(0.6, 0.7))
            .secs(3.8),
    )
    .shot(
        dolly(at_bellows, END_EYE)
            .look(Look::Pan(bellows_look, END_LOOK))
            .ease(speeds(0.7, 1.0))
            .secs(2.0),
    )
}
