//! Section 1 (Tier 0): the timber camp on the upper south valley shoulder. Hand-worked tools (a
//! rubble pile, loaded sieve, wet millstone, and a furnace beside hand bellows) stand directly in
//! front of a wound spring engine driving the first timber, stone and directing tubes. Right
//! beside them, a stone millrace turns a water wheel whose wooden axles power a Timber Quarry
//! inside its lashed frame.

use showcase_api::{Film, Look, Set, dolly, flyby};
use weft_sdk::{Dir, Shape};

use super::camera::speeds;
use super::ground::fell_near;
use super::parts::{
    CHEST, COBBLESTONE, FURNACE, STONE_BRICKS, WATER, area, axles, build_frame, fit_bit, fuel,
    spring_engine, start_quarry, stock, stock_group, tubes, wind_spring,
};
use super::site::{S1_TOP, above, lowered, lowered_block};
use crate::bellows::HAND_BELLOWS;
use crate::excavator::kind::rows::TIMBER;
use crate::excavator::{Frame, LASHED_FRAME, TIMBER_QUARRY};
use crate::lib::{BlockState, Pos};
use crate::millstone::{MILL, MILLSTONE};
use crate::rubble::{Grade, Ore, RUBBLE};
use crate::sieve::{Load, SIEVE, SIFT};
use crate::tube::{DIRECTING_TUBE, TIMBER_TUBE};
use crate::water_wheel::WATER_WHEEL;

/// Floor height above the grass surface of Pad S1, in blocks from the works.
const FLOOR: i32 = S1_TOP + 1;

/// Compact hand-work camp along the front row.
const SIEVE_AT: Pos = Pos::new(-33, FLOOR, 34);
const MILLSTONE_AT: Pos = Pos::new(-32, FLOOR, 34);
const FURNACE_AT: Pos = Pos::new(-30, FLOOR, 34);
const BELLOWS_AT: Pos = Pos::new(-29, FLOOR, 34);

/// First item tube line directly behind the hand tools, driven by a wound spring engine.
const TUBE_IN: Pos = Pos::new(-34, FLOOR, 32);
const PULL_TUBE: Pos = Pos::new(-33, FLOOR, 32);
const SPRING_AT: Pos = Pos::new(-33, FLOOR + 1, 32);
const DIRECTING_AT: Pos = Pos::new(-31, FLOOR, 32);
const SIDE_CHEST: Pos = Pos::new(-31, FLOOR, 31);
const TUBE_OUT: Pos = Pos::new(-29, FLOOR, 32);

/// Millrace and two water wheels (2 × 12 = 24 E/t against the quarry's 16 E/t draw), ending in a
/// stone-capped catch pit cleanly west of the Timber Quarry's frame (x -20..-14).
const WHEELS: [Pos; 2] = [Pos::new(-26, FLOOR, 33), Pos::new(-23, FLOOR, 33)];
const WATER_SOURCE: Pos = Pos::new(-27, S1_TOP, 33);
const MILLRACE_X: [i32; 2] = [-27, -23];
const CATCH_PIT: Pos = Pos::new(-22, S1_TOP - 1, 33);

/// Timber Quarry and its pre-built 5 by 5 lashed frame behind it (x -20..-14, z 28..34).
pub const QUARRY_AT: Pos = Pos::new(-17, FLOOR, 35);
const QUARRY_CHEST: Pos = Pos::new(-16, FLOOR, 35);
const PIT_DEPTH: i32 = 2;

/// Where S1 hands the camera off to S2, already climbing high above the lashed frame and grass
/// slope between pads.
pub const END_EYE: [f64; 3] = lowered([-13.0, FLOOR as f64 + 7.2, 39.2]);
pub const END_LOOK: [f64; 3] = lowered([-17.0, FLOOR as f64 + 1.2, 32.5]);

pub fn build(set: &Set) {
    fell_near(set, -33, 36, 10);
    hand_camp(set);
    spring_line(set);
    millrace_and_wheel(set);
    quarry(set);
}

/// Low rubble pile, sieve, millstone, furnace and hand bellows set close together.
fn hand_camp(set: &Set) {
    for at in [Pos::new(-34, FLOOR, 34), Pos::new(-34, FLOOR, 33)] {
        set.place(at, BlockState::of(&RUBBLE));
    }
    set.place(SIEVE_AT, BlockState::of(&SIEVE));
    set.place(MILLSTONE_AT, MILLSTONE.with(&[("facing", "south")]).into());
    set.place(FURNACE_AT, FURNACE.with(&[("facing", "south")]).into());
    set.place(BELLOWS_AT, HAND_BELLOWS.with(&[("facing", "west")]).into());
}

/// Chests, timber tube under a spring engine, stone tubes, and a directing tube pointing east.
fn spring_line(set: &Set) {
    for chest in [TUBE_IN, SIDE_CHEST, TUBE_OUT] {
        set.place(chest, BlockState::of(&CHEST));
    }
    set.place(PULL_TUBE, BlockState::of(&TIMBER_TUBE));
    spring_engine(set, SPRING_AT, Dir::Down);
    tubes(set, Pos::new(-32, FLOOR, 32), Pos::new(-32, FLOOR, 32));
    set.place(DIRECTING_AT, DIRECTING_TUBE.with(&[("out", "east")]).into());
    tubes(set, Pos::new(-30, FLOOR, 32), Pos::new(-30, FLOOR, 32));
}

/// Recessed stone channel carrying flowing water under two water wheels, capped solid at x = -21
/// so no water reaches the Timber Quarry pit; wooden axles exit both wheels' south ends along
/// z = 35 directly into the quarry's west face.
fn millrace_and_wheel(set: &Set) {
    let bed = area(
        Pos::new(MILLRACE_X[0], S1_TOP - 1, 32),
        Pos::new(CATCH_PIT.x + 1, S1_TOP - 1, 34),
    );
    set.fill(bed, BlockState::of(&COBBLESTONE));
    for curb_z in [32, 34] {
        set.fill(
            area(
                Pos::new(MILLRACE_X[0], S1_TOP, curb_z),
                Pos::new(CATCH_PIT.x + 1, S1_TOP, curb_z),
            ),
            BlockState::of(&STONE_BRICKS),
        );
    }
    set.place(
        Pos::new(CATCH_PIT.x + 1, S1_TOP, 33),
        BlockState::of(&STONE_BRICKS),
    );
    set.fill(
        area(
            Pos::new(MILLRACE_X[0], S1_TOP, 33),
            Pos::new(MILLRACE_X[1], S1_TOP, 33),
        ),
        BlockState::AIR,
    );
    set.place(CATCH_PIT.add_y(-1), BlockState::of(&COBBLESTONE));
    set.place(CATCH_PIT, BlockState::AIR);
    set.place(CATCH_PIT.add_y(1), BlockState::AIR);
    set.place(WATER_SOURCE, BlockState::of(&WATER));
    for wheel in WHEELS {
        set.place(wheel, WATER_WHEEL.with(&[("facing", "north")]).into());
        axles(set, wheel.add_z(1), wheel.add_z(2));
    }
    axles(
        set,
        Pos::new(WHEELS[0].x, FLOOR, 35),
        Pos::new(QUARRY_AT.x - 1, FLOOR, 35),
    );
}

/// Timber Quarry facing south, its lashed frame pre-built and its pit started.
fn quarry(set: &Set) {
    set.place(QUARRY_AT, TIMBER_QUARRY.with(&[("facing", "south")]).into());
    set.place(QUARRY_CHEST, BlockState::of(&CHEST));
    let world = set.world();
    let pos = set.at(QUARRY_AT);
    let frame = Frame::plan(&world, pos, Dir::South, &TIMBER);
    build_frame(set, &frame, &LASHED_FRAME);
    let lo = frame.lo.add_x(1).add_z(1).with_y(pos.y - PIT_DEPTH);
    let hi = frame.hi.with_y(pos.y);
    world.fill(Shape::cuboid(lo, hi), BlockState::AIR.id());
}

/// Loads the hand tools, winds the spring engine, stocks the tubes, and starts the Timber Quarry.
pub fn start(set: &Set) {
    let world = set.world();
    if let Some(sift) = world.component_mut(world.block_entity(set.at(SIEVE_AT)), &SIFT) {
        sift.load = Load::Rubble as u32;
    }
    if let Some(mill) = world.component_mut(world.block_entity(set.at(MILLSTONE_AT)), &MILL) {
        if let Some(iron) = Ore::of("iron") {
            mill.ore = iron;
        }
        mill.grade = Grade::Poor as u32;
        mill.held = 16;
        mill.water = 1_000;
        mill.grit = 8;
    }
    fuel(set, FURNACE_AT);
    stock_group(set, FURNACE_AT, "input", &[("base:raw_iron", 64)]);
    stock(
        set,
        TUBE_IN,
        &["cogwork:rubble", "base:cobblestone", "base:raw_iron"],
        4,
    );
    wind_spring(set, SPRING_AT, 10_000);
    fit_bit(set, QUARRY_AT, "cogwork:stone_bit", 0);
    start_quarry(set, QUARRY_AT, Dir::South, &TIMBER, FLOOR - PIT_DEPTH - 1);
}

/// Opening swoop into the timber camp, then a close glide east past the hand tools, spring tubes,
/// water wheel, and Timber Quarry.
pub fn shots(film: Film) -> Film {
    let f = FLOOR as f64;
    let camp_mid = lowered([-27.0, f + 1.0, 33.0]);
    let hand_look = lowered(above(MILLSTONE_AT, 0.2));
    let spring_look = lowered(above(SPRING_AT, -0.2));
    let wheel_look = lowered([-24.5, f + 0.8, 33.5]);

    let entry = [
        lowered([-31.0, f + 7.5, 41.0]),
        lowered([-32.5, f + 4.4, 39.2]),
        lowered([-32.8, f + 2.4, 37.6]),
    ];
    let past_hand = lowered([-31.6, f + 2.4, 37.6]);
    let past_spring = lowered([-27.8, f + 2.3, 37.8]);
    let at_quarry = lowered([-17.2, f + 2.8, 38.4]);

    film.title("mod.cogwork.name", 4.0)
        .shot(
            flyby(&entry)
                .look(Look::Pan(camp_mid, hand_look))
                .ease(speeds(2.6, 0.9))
                .secs(5.5),
        )
        .title("cogwork.film.t0_hand", 5.0)
        .shot(
            dolly(entry[2], past_hand)
                .look(Look::Pan(hand_look, spring_look))
                .ease(speeds(0.9, 0.8))
                .secs(5.5),
        )
        .title("cogwork.film.t0_spring", 4.5)
        .shot(
            dolly(past_hand, past_spring)
                .look(Look::Pan(spring_look, wheel_look))
                .ease(speeds(0.8, 0.9))
                .secs(5.5),
        )
        .title("cogwork.film.t0_timber", 5.0)
        .shot(
            dolly(past_spring, at_quarry)
                .look(Look::Pan(wheel_look, END_LOOK))
                .ease(speeds(0.9, 0.6))
                .secs(6.0),
        )
        .menu("cogwork:timber_quarry", lowered_block(QUARRY_AT), 4.0)
        .shot(
            dolly(at_quarry, END_EYE)
                .look_at(END_LOOK)
                .ease(speeds(0.6, 1.2))
                .secs(4.5),
        )
}
