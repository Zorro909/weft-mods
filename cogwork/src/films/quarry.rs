//! The two Gantry Excavators east of the hall, both facing south with their areas behind them.
//! The nearer one (`DIGGER`) stands on a legacy `gantry_frame` over a pit dug past y = -64, fed
//! lubricant from Pad R into its top and backed by a Bit Magazine on its south face that swaps a
//! worn diamond-tipped bit on camera; its output leaves west through gilded tubes to a sorting
//! tube. The farther one (`BUILDER`), beyond the knoll, holds composite panels and builds its
//! composite frame on camera.

use showcase_api::{Ease, Film, Look, Set, dolly, flyby, hold, orbit};
use weft_sdk::Block;
use weft_sdk::{Dir, Item, Shape};

use super::camera::{on_orbit, speeds};
use super::leveling;
use super::parts::{
    CHEST, area, build_frame, engine, fit_bit, fuel, shafts, start_quarry, stock_group, tubes,
};
use super::plastics_bay;
use super::site::{SITE, WORKS_DOWN, above, lowered, lowered_block};
use crate::bit_magazine::BIT_MAGAZINE;
use crate::excavator::bit::DIAMOND_BIT;
use crate::excavator::kind::rows::GANTRY;
use crate::excavator::{EXCAVATOR, Frame, GANTRY_FRAME};
use crate::lib::{BlockState, Pos};
use crate::tube::{GILDED_TUBE, SORTING_TUBE, set_sample, sorting::ROW};

static STONE: Block = Block::named("base:stone");

/// The one digging, the height in the world of the layer it mines when the film starts, and
/// so how deep its pit is then.
pub const DIGGER: Pos = Pos::new(33, 0, 2);
pub const MAGAZINE_AT: Pos = Pos::new(33, 0, 3);
const MINING_Y: i32 = -70;
const PIT_DEPTH: i32 = DIGGER.y + SITE.y - WORKS_DOWN - MINING_Y - 1;
/// The one that builds its composite frame while the camera watches, and its engines' sides.
const BUILDER: Pos = Pos::new(64, 1, 3);
const BUILDER_ENGINES: [Dir; 2] = [Dir::Up, Dir::East];
/// How far behind an excavator the middle of its 9 by 9 area lies.
const AREA_BEHIND: i32 = 6;
/// What the camera turns round and goes down to: the middle of the pit at the top of the
/// ground, and of the layer the drill mines; and the middle of the frame the builder puts up.
pub const PIT: Pos = Pos::new(DIGGER.x, DIGGER.y, DIGGER.z - AREA_BEHIND);
pub const DRILL: Pos = PIT.add_y(-PIT_DEPTH - 1);
pub const RAISED: Pos = Pos::new(BUILDER.x, BUILDER.y + 1, BUILDER.z - AREA_BEHIND);
/// Where the digger's output is sorted, west of it in its own row, and the gilded stretch of
/// the tubes between, along x.
pub const SORTER: Pos = Pos::new(26, 0, DIGGER.z);
const GILDED_X: [i32; 2] = [28, 30];
/// The shaft from the hall: the row it leaves the hall in, and the column it turns north in
/// to reach the digger's east side.
const SHAFT_Z: i32 = super::crafting::SHAFT_Z;
const SHAFT_X: i32 = DIGGER.x + 1;

/// Camera landmarks shared with S11 (loader departure).
pub const FRAME_LOOK: [f64; 3] = lowered(above(RAISED, 0.5));
pub const PIT_LOOK: [f64; 3] = lowered(above(PIT, 0.5));
const SORTER_LOOK: [f64; 3] = lowered(above(SORTER, 0.0));
const DRILL_LOOK: [f64; 3] = lowered(above(DRILL, 0.0));
const MAGAZINE_LOOK: [f64; 3] = lowered(above(MAGAZINE_AT, 0.3));

const SORTER_OUT: f64 = 3.0;
const SORTER_UP: f64 = 3.0;
const SORTER_ARC: [f64; 2] = [125.0, 85.0];

const PIT_OUT: f64 = 9.0;
const PIT_UP: [f64; 2] = [9.5, 6.2];
const PIT_ARC: [f64; 2] = [108.0, 170.0];

const OVER_PIT_CENTER: [f64; 3] = lowered([30.8, 6.8, -2.6]);
const INTO_PIT: [f64; 3] = lowered([32.0, 2.2, -3.2]);
const DOWN_PIT: [f64; 3] = lowered([33.5, -14.0, -3.5]);
const OVER_DRILL: f64 = 3.2;

const FRAME_OUT: f64 = 13.5;
const FRAME_UP: [f64; 2] = [8.5, 7.5];
const FRAME_ARC: [f64; 2] = [225.0, 95.0];

pub fn frame_end_eye() -> [f64; 3] {
    on_orbit(FRAME_LOOK, FRAME_OUT, FRAME_UP[1], FRAME_ARC[1])
}

pub fn build(set: &Set) {
    for at in [DIGGER, BUILDER] {
        set.place(at, EXCAVATOR.with(&[("facing", "south")]).into());
    }
    set.place(
        MAGAZINE_AT,
        BIT_MAGAZINE.with(&[("facing", "north")]).into(),
    );
    frame_and_pit(set);
    let shaft_row = |x: i32| Pos::new(x, 0, SHAFT_Z);
    shafts(set, shaft_row(super::hall::EAST + 1), shaft_row(SHAFT_X));
    let beside_digger = Pos::new(SHAFT_X, 0, DIGGER.z);
    shafts(
        set,
        beside_digger,
        beside_digger.add_z(SHAFT_Z - 1 - DIGGER.z),
    );
    output_line(set);
    for side in BUILDER_ENGINES {
        engine(set, BUILDER.toward(side), side.opposite());
    }
    set.place(BUILDER.toward(Dir::South), BlockState::of(&CHEST));
}

/// The digger's legacy `gantry_frame` and deep pit, lining all four vertical pit walls with stone
/// down to `MINING_Y` so natural caves never open dark voids into the shaft.
fn frame_and_pit(set: &Set) {
    let world = set.world();
    let pos = set.at(DIGGER);
    let frame = Frame::plan(&world, pos, Dir::South, &GANTRY);
    build_frame(set, &frame, &GANTRY_FRAME);
    let wall_lo_y = pos.y - PIT_DEPTH - 2;
    let wall_hi_y = pos.y - 1;
    let stone = BlockState::of(&STONE).id();
    for (lo, hi) in [
        (
            Pos::new(frame.lo.x, wall_lo_y, frame.lo.z),
            Pos::new(frame.hi.x, wall_hi_y, frame.lo.z),
        ),
        (
            Pos::new(frame.lo.x, wall_lo_y, frame.hi.z),
            Pos::new(frame.hi.x, wall_hi_y, frame.hi.z),
        ),
        (
            Pos::new(frame.lo.x, wall_lo_y, frame.lo.z),
            Pos::new(frame.lo.x, wall_hi_y, frame.hi.z),
        ),
        (
            Pos::new(frame.hi.x, wall_lo_y, frame.lo.z),
            Pos::new(frame.hi.x, wall_hi_y, frame.hi.z),
        ),
    ] {
        world.fill(Shape::cuboid(lo, hi), stone);
    }
    let lowest = frame.lo.add_x(1).add_z(1).with_y(pos.y - PIT_DEPTH);
    let far_corner = frame.hi.with_y(pos.y);
    world.fill(Shape::cuboid(lowest, far_corner), BlockState::AIR.id());
}

/// From the digger's west side along its row to the sorting tube, which has a chest on three
/// sides: cobblestone north, dirt south, the rest west.
fn output_line(set: &Set) {
    let on_row = |x: i32| Pos::new(x, 0, DIGGER.z);
    tubes(set, SORTER.toward(Dir::East), DIGGER.toward(Dir::West));
    let gilded = area(on_row(GILDED_X[0]), on_row(GILDED_X[1]));
    set.fill(gilded, BlockState::of(&GILDED_TUBE));
    set.place(SORTER, BlockState::of(&SORTING_TUBE));
    for side in [Dir::North, Dir::South, Dir::West] {
        set.place(SORTER.toward(side), BlockState::of(&CHEST));
    }
}

/// Sets the sorting tube's samples, fits diamond bits in DIGGER and its Bit Magazine, stocks
/// composite panels in BUILDER, and puts DIGGER to mining inside its legacy frame.
pub fn start(set: &Set) {
    fit_bit(set, DIGGER, "cogwork:diamond_bit", 0);
    stock_group(set, MAGAZINE_AT, "bits", &[("cogwork:diamond_bit", 1); 4]);
    stock_group(
        set,
        BUILDER,
        "materials",
        &[("cogwork:composite_panel", 32)],
    );
    start_quarry(set, DIGGER, Dir::South, &GANTRY, DRILL.y);

    let samples = [(Dir::North, "base:cobblestone"), (Dir::South, "base:dirt")];
    let world = set.world();
    let sorter = world.block_entity(set.at(SORTER));
    for (side, item) in samples {
        if let Some(item) = Item::lookup(item) {
            set_sample(&world, sorter, side.face() * ROW as u32, item);
        }
    }
}

/// Wears DIGGER's fitted diamond bit down to two uses left so the Bit Magazine swaps in a fresh
/// bit live as the camera glides past.
pub fn prep_swap(set: &Set) {
    fit_bit(set, DIGGER, "cogwork:diamond_bit", DIAMOND_BIT.lasts - 2);
}

/// Lights the builder's engines: it puts up its composite frame.
pub fn raise_frame(set: &Set) {
    for side in BUILDER_ENGINES {
        fuel(set, BUILDER.toward(side));
    }
}

/// Section 9 (Tier 5b): north from Pad P to the sorting tube, east past the lubricated Gantry
/// Excavator and its Bit Magazine swap, then down the deep pit past y = -64 and fade out.
pub fn sorting_and_digging_shots(film: Film) -> Film {
    let at_sorter = on_orbit(SORTER_LOOK, SORTER_OUT, SORTER_UP, SORTER_ARC[0]);
    let leave_sorter = on_orbit(SORTER_LOOK, SORTER_OUT, SORTER_UP, SORTER_ARC[1]);
    // Cross north high above the grass bank (6-8 blocks clear of the y = 8 hillside and above the
    // hall's roof ridge), looking north-east over the open quarry yard and lake so the slope stays
    // in the lower third, then descend cleanly onto the sorting tube.
    let mid_north = lowered([25.6, 16.0, 8.2]);
    let over_yard = lowered([25.6, 15.0, 4.2]);
    let east_sky = lowered([31.5, 13.5, 6.0]);
    let far_lake = lowered([31.5, 6.5, -2.5]);
    let by_magazine = [
        leave_sorter,
        lowered([31.2, 4.3, 6.6]),
        lowered([35.0, 4.5, 6.4]),
    ];
    let over_pit = on_orbit(PIT_LOOK, PIT_OUT, PIT_UP[0], PIT_ARC[0]);
    let west_of_pit = on_orbit(PIT_LOOK, PIT_OUT, PIT_UP[1], PIT_ARC[1]);
    let into_center = [west_of_pit, OVER_PIT_CENTER, INTO_PIT];
    let at_drill = [INTO_PIT[0], DRILL_LOOK[1] + OVER_DRILL, INTO_PIT[2]];

    let round = |from: f64, to: f64, ease: Ease, secs: f32| {
        orbit(SORTER_LOOK, SORTER_OUT)
            .arc(from, to)
            .height(SORTER_UP, SORTER_UP)
            .ease(ease)
            .secs(secs)
    };

    film.cue("noon_3")
        .shot(
            dolly(plastics_bay::END_EYE, mid_north)
                .look(Look::Pan(plastics_bay::END_LOOK, east_sky))
                .ease(speeds(0.8, 1.0))
                .secs(4.2),
        )
        .shot(
            dolly(mid_north, over_yard)
                .look(Look::Pan(east_sky, far_lake))
                .ease(speeds(1.0, 1.0))
                .secs(3.0),
        )
        .shot(
            dolly(over_yard, at_sorter)
                .look(Look::Pan(far_lake, SORTER_LOOK))
                .ease(speeds(1.0, 0.5))
                .secs(3.5),
        )
        .title("cogwork.film.sorting", 3.0)
        .shot(round(SORTER_ARC[0], 110.0, speeds(0.5, 0.15), 3.2))
        .menu("cogwork:sorting_tube", lowered_block(SORTER), 3.5)
        .shot(round(110.0, 100.0, speeds(0.15, 0.15), 3.8))
        .shot(round(100.0, SORTER_ARC[1], speeds(0.15, 0.9), 2.4))
        .cue("prep_swap")
        .title("cogwork.film.t5_magazine", 3.8)
        .shot(
            flyby(&by_magazine)
                .look(Look::Pan(SORTER_LOOK, MAGAZINE_LOOK))
                .ease(speeds(0.9, 0.7))
                .secs(4.2),
        )
        .menu("cogwork:bit_magazine", lowered_block(MAGAZINE_AT), 3.8)
        .shot(
            dolly(by_magazine[2], over_pit)
                .look(Look::Pan(MAGAZINE_LOOK, PIT_LOOK))
                .ease(speeds(0.7, 1.5))
                .secs(5.5),
        )
        .title("cogwork.film.dig", 5.0)
        .shot(
            orbit(PIT_LOOK, PIT_OUT)
                .arc(PIT_ARC[0], PIT_ARC[1])
                .height(PIT_UP[0], PIT_UP[1])
                .ease(speeds(1.5, 1.4))
                .secs(6.5),
        )
        .shot(
            flyby(&into_center)
                .look(Look::Pan(PIT_LOOK, DOWN_PIT))
                .ease(speeds(1.4, 4.0))
                .secs(3.5),
        )
        .title("cogwork.film.deep", 5.0)
        .shot(
            dolly(INTO_PIT, at_drill)
                .look_at(DRILL_LOOK)
                .ease(speeds(4.0, 0.3))
                .secs(6.5),
        )
        .menu("cogwork:excavator", lowered_block(DIGGER), 4.0)
        .shot(hold(at_drill).look_at(DRILL_LOOK).secs(4.5))
        .fade_out(1.5)
}

/// Section 10c: east from the seismic thumper into the builder's orbit as it puts up its
/// composite frame.
pub fn framing_shots(film: Film) -> Film {
    let at_frame = on_orbit(FRAME_LOOK, FRAME_OUT, FRAME_UP[0], FRAME_ARC[0]);
    film.cue("frame")
        .shot(
            dolly(leveling::END_EYE, at_frame)
                .look(Look::Pan(leveling::THUMPER_LOOK, FRAME_LOOK))
                .ease(speeds(1.0, 1.6))
                .secs(4.5),
        )
        .title("cogwork.film.frame", 5.0)
        .shot(
            orbit(FRAME_LOOK, FRAME_OUT)
                .arc(FRAME_ARC[0], FRAME_ARC[1])
                .height(FRAME_UP[0], FRAME_UP[1])
                .ease(speeds(1.6, 2.0))
                .secs(12.0),
        )
}
