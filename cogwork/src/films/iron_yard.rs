//! Section 2 (Tier 1): the iron yard on Pad S2. Ember engines drive an Iron Quarry inside its
//! riveted iron frame, sending output through gilded tubes. Beside it under an open-fronted timber
//! roof stand the driven ore machines: a shaker screen with a void tube for waste, a washer fed
//! from a merged water tank, a grinding mill, and a furnace blown by a driven bellows.

use showcase_api::{Film, Look, Set, dolly};
use weft_sdk::{Dir, Shape};

use super::camera::speeds;
use super::parts::{
    CHEST, FURNACE, OAK_LOG, OAK_SLAB, area, build_frame, engine, fit_bit, fuel, liquid_tubes,
    prefill, shafts, start_quarry, stock, stock_group, stock_rubble, tubes,
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
pub const QUARRY_AT: Pos = Pos::new(-6, FLOOR, 36);
const QUARRY_ENGINES: [(Pos, Dir); 3] = [
    (Pos::new(-9, FLOOR, 36), Dir::East),
    (Pos::new(-8, FLOOR + 1, 36), Dir::Down),
    (Pos::new(-7, FLOOR + 1, 36), Dir::Down),
];
const EMBER_MENU_AT: Pos = Pos::new(-8, FLOOR + 1, 36);
const QUARRY_CHEST: Pos = Pos::new(-1, FLOOR, 36);
const PIT_DEPTH: i32 = 3;

/// Driven ore processing machines along z = 35, all facing south toward the camera.
const SHAKER_AT: Pos = Pos::new(3, FLOOR, 35);
const SHAKER_IN: Pos = Pos::new(1, FLOOR, 35);
const SHAKER_PULL: Pos = Pos::new(2, FLOOR, 35);
const SHAKER_PULL_ENGINE: Pos = Pos::new(2, FLOOR, 34);
const VOID_AT: Pos = Pos::new(2, FLOOR, 36);
const SHAKER_OUT: Pos = Pos::new(4, FLOOR, 36);

const WASHER_AT: Pos = Pos::new(6, FLOOR, 35);
const WASHER_OUT: Pos = Pos::new(5, FLOOR, 36);
const MILL_AT: Pos = Pos::new(9, FLOOR, 35);
const FURNACE_AT: Pos = Pos::new(12, FLOOR, 35);
const BELLOWS_AT: Pos = Pos::new(13, FLOOR, 35);

/// Water tank feeding both the washer and the grinding mill.
const WATER_TANK: Area = Area {
    lo: Pos::new(7, FLOOR, 31),
    hi: Pos::new(8, FLOOR + 1, 32),
};

/// Ember engines directly behind each driven machine.
const LINE_ENGINES: [(Pos, Dir); 4] = [
    (Pos::new(3, FLOOR, 34), Dir::South),
    (Pos::new(6, FLOOR, 34), Dir::South),
    (Pos::new(9, FLOOR, 34), Dir::South),
    (Pos::new(13, FLOOR, 34), Dir::South),
];

/// Where S2 hands the camera off to S3, heading east.
pub const END_EYE: [f64; 3] = lowered([14.5, FLOOR as f64 + 2.4, 40.0]);
pub const END_LOOK: [f64; 3] = lowered([13.0, FLOOR as f64 + 0.5, 35.5]);

pub fn build(set: &Set) {
    iron_quarry(set);
    shed(set);
    processing_line(set);
}

/// Iron Quarry with pre-built iron frame, 3 ember engines, and gilded tube run to its chest.
fn iron_quarry(set: &Set) {
    set.place(QUARRY_AT, IRON_QUARRY.with(&[("facing", "south")]).into());
    shafts(set, Pos::new(-8, FLOOR, 36), Pos::new(-7, FLOOR, 36));
    for (at, toward) in QUARRY_ENGINES {
        engine(set, at, toward);
    }
    tubes(set, Pos::new(-5, FLOOR, 36), Pos::new(-5, FLOOR, 36));
    set.fill(
        area(Pos::new(-4, FLOOR, 36), Pos::new(-2, FLOOR, 36)),
        BlockState::of(&GILDED_TUBE),
    );
    set.place(QUARRY_CHEST, BlockState::of(&CHEST));
    let world = set.world();
    let pos = set.at(QUARRY_AT);
    let frame = Frame::plan(&world, pos, Dir::South, &IRON);
    build_frame(set, &frame, &IRON_FRAME);
    let lo = frame.lo.add_x(1).add_z(1).with_y(pos.y - PIT_DEPTH);
    let hi = frame.hi.with_y(pos.y);
    world.fill(Shape::cuboid(lo, hi), BlockState::AIR.id());
}

/// Light open-fronted timber canopy over the processing line, posts clear of every machine front.
fn shed(set: &Set) {
    let roof_y = FLOOR + 4;
    for x in [1, 14] {
        for z in [30, 36] {
            set.fill(
                area(Pos::new(x, FLOOR, z), Pos::new(x, roof_y - 1, z)),
                BlockState::of(&OAK_LOG),
            );
        }
    }
    set.fill(
        area(Pos::new(1, roof_y, 30), Pos::new(14, roof_y, 36)),
        OAK_SLAB.with(&[("type", "bottom")]).into(),
    );
}

/// Shaker screen + void tube, washer + water tank, grinding mill, and furnace + driven bellows.
fn processing_line(set: &Set) {
    for chest in [SHAKER_IN, SHAKER_OUT, WASHER_OUT] {
        set.place(chest, BlockState::of(&CHEST));
    }
    set.place(SHAKER_PULL, BlockState::of(&TIMBER_TUBE));
    engine(set, SHAKER_PULL_ENGINE, Dir::South);
    set.place(SHAKER_AT, SHAKER_SCREEN.with(&[("facing", "south")]).into());
    tubes(set, Pos::new(3, FLOOR, 36), Pos::new(3, FLOOR, 36));
    set.place(VOID_AT, BlockState::of(&VOID_TUBE));

    set.place(WASHER_AT, WASHER.with(&[("facing", "south")]).into());
    tubes(set, Pos::new(6, FLOOR, 36), Pos::new(6, FLOOR, 36));
    set.fill(WATER_TANK, BlockState::of(&LIQUID_TANK));
    liquid_tubes(
        set,
        &[
            Pos::new(7, FLOOR, 33),
            Pos::new(7, FLOOR, 34),
            Pos::new(7, FLOOR, 35),
            Pos::new(8, FLOOR, 33),
            Pos::new(8, FLOOR, 34),
            Pos::new(8, FLOOR, 35),
        ],
    );

    set.place(MILL_AT, GRINDING_MILL.with(&[("facing", "south")]).into());
    tubes(set, Pos::new(9, FLOOR, 36), Pos::new(11, FLOOR, 36));
    tubes(set, Pos::new(11, FLOOR, 35), Pos::new(11, FLOOR, 35));
    set.place(FURNACE_AT, FURNACE.with(&[("facing", "south")]).into());
    set.place(
        BELLOWS_AT,
        DRIVEN_BELLOWS.with(&[("facing", "west")]).into(),
    );

    for (at, toward) in LINE_ENGINES {
        engine(set, at, toward);
    }
}

/// Fuels every engine, prefills the water tank, stocks rubble and ore rubble, and starts the
/// Iron Quarry.
pub fn start(set: &Set) {
    for (at, _) in QUARRY_ENGINES
        .into_iter()
        .chain(LINE_ENGINES)
        .chain([(SHAKER_PULL_ENGINE, Dir::South)])
    {
        fuel(set, at);
    }
    fit_bit(set, QUARRY_AT, "cogwork:iron_bit", 0);
    start_quarry(set, QUARRY_AT, Dir::South, &IRON, FLOOR - PIT_DEPTH - 1);

    stock(set, SHAKER_IN, &["cogwork:rubble"], 8);
    stock_group(set, SHAKER_AT, "input", &[("cogwork:rubble", 64); 3]);
    stock_rubble(set, WASHER_AT, "input", "iron", Grade::Poor, 3);
    stock_rubble(set, MILL_AT, "input", "iron", Grade::Common, 3);
    stock_group(set, MILL_AT, "sand", &[("base:sand", 64)]);
    fuel(set, FURNACE_AT);
    stock_group(set, FURNACE_AT, "input", &[("base:raw_iron", 64)]);

    let water_blocks = WATER_TANK.blocks().chain([
        Pos::new(7, FLOOR, 33),
        Pos::new(7, FLOOR, 34),
        Pos::new(7, FLOOR, 35),
        Pos::new(8, FLOOR, 33),
        Pos::new(8, FLOOR, 34),
        Pos::new(8, FLOOR, 35),
        WASHER_AT,
        MILL_AT,
    ]);
    prefill(set, water_blocks, "base:water", [1, 1]);
}

/// Continuous glide east past the Iron Quarry and along the driven ore processing shed, opening
/// each machine's screen over it.
pub fn shots(film: Film) -> Film {
    let f = FLOOR as f64;
    let quarry_look = lowered([-6.0, f + 1.5, 34.0]);
    let shaker_look = lowered(above(SHAKER_AT, 0.2));
    let washer_look = lowered(above(WASHER_AT, 0.2));
    let mill_look = lowered(above(MILL_AT, 0.2));

    let at_engines = lowered([-9.0, f + 3.4, 41.5]);
    let at_quarry = lowered([-3.5, f + 3.4, 41.5]);
    let past_quarry = lowered([-0.5, f + 2.8, 40.5]);
    let at_shaker = lowered([3.5, f + 2.2, 40.0]);
    let at_washer = lowered([6.5, f + 2.2, 40.0]);
    let at_mill = lowered([10.0, f + 2.2, 40.0]);

    film.title("cogwork.film.t1_iron", 5.5)
        .shot(
            dolly(timber_camp::END_EYE, at_engines)
                .look(Look::Pan(timber_camp::END_LOOK, quarry_look))
                .ease(speeds(1.2, 0.9))
                .secs(5.5),
        )
        .menu("cogwork:ember_engine", lowered_block(EMBER_MENU_AT), 3.5)
        .shot(
            dolly(at_engines, at_quarry)
                .look_at(quarry_look)
                .ease(speeds(0.9, 0.9))
                .secs(4.5),
        )
        .menu("cogwork:iron_quarry", lowered_block(QUARRY_AT), 4.0)
        .shot(
            dolly(at_quarry, past_quarry)
                .look(Look::Pan(quarry_look, shaker_look))
                .ease(speeds(0.9, 0.8))
                .secs(5.0),
        )
        .title("cogwork.film.t1_sift", 5.5)
        .menu("cogwork:shaker_screen", lowered_block(SHAKER_AT), 3.5)
        .shot(
            dolly(past_quarry, at_shaker)
                .look_at(shaker_look)
                .ease(speeds(0.8, 0.7))
                .secs(5.0),
        )
        .title("cogwork.film.t1_wash", 5.5)
        .menu("cogwork:washer", lowered_block(WASHER_AT), 3.5)
        .shot(
            dolly(at_shaker, at_washer)
                .look(Look::Pan(shaker_look, washer_look))
                .ease(speeds(0.7, 0.7))
                .secs(4.8),
        )
        .menu("cogwork:grinding_mill", lowered_block(MILL_AT), 3.5)
        .shot(
            dolly(at_washer, at_mill)
                .look(Look::Pan(washer_look, mill_look))
                .ease(speeds(0.7, 0.8))
                .secs(4.8),
        )
        .title("cogwork.film.t1_bellows", 4.5)
        .menu("cogwork:driven_bellows", lowered_block(BELLOWS_AT), 3.5)
        .shot(
            dolly(at_mill, END_EYE)
                .look(Look::Pan(mill_look, END_LOOK))
                .ease(speeds(0.8, 1.0))
                .secs(5.0),
        )
}
