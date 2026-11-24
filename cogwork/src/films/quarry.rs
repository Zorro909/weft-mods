//! The two Gantry Excavators east of the hall, both facing south with their areas behind them.
//! The nearer one (`DIGGER`) stands on a legacy `gantry_frame` over a pit dug past y = -64, fed
//! lubricant from Pad R into its top and backed by a Bit Magazine on its south face that swaps a
//! worn diamond-tipped bit on camera; its output leaves west through gilded tubes to a sorting
//! tube. The farther one (`BUILDER`), beyond the knoll, holds composite panels and builds its
//! composite frame on camera.

use showcase_api::{Ease, Film, Look, Set, dolly, flyby, hold, orbit};
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

/// Camera landmarks shared with S10 (knoll entry) and S11 (loader departure).
pub const PIT_LOOK: [f64; 3] = lowered(above(PIT, 0.5));
pub const FRAME_LOOK: [f64; 3] = lowered(above(RAISED, 0.5));
const SORTER_LOOK: [f64; 3] = lowered(above(SORTER, 0.0));
const DRILL_LOOK: [f64; 3] = lowered(above(DRILL, 0.0));
const MAGAZINE_LOOK: [f64; 3] = lowered(above(MAGAZINE_AT, 0.3));

const SORTER_OUT: f64 = 3.0;
const SORTER_UP: f64 = 3.0;
const SORTER_ARC: [f64; 2] = [125.0, 85.0];

pub const PIT_OUT: f64 = 8.0;
pub const PIT_UP: [f64; 2] = [9.0, 2.5];
const PIT_ARC: [f64; 2] = [108.0, 180.0];
pub const PIT_LEFT: f64 = 215.0;

const INTO_PIT: [f64; 3] = lowered([30.0, 2.0, -4.0]);
const DOWN_PIT: [f64; 3] = lowered([30.25, -8.0, -4.0]);
const OVER_DRILL: f64 = 4.0;

const FRAME_OUT: f64 = 10.0;
const FRAME_UP: [f64; 2] = [8.0, 7.0];
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

/// The digger's legacy `gantry_frame` and deep pit, showing legacy frame compatibility.
fn frame_and_pit(set: &Set) {
    let world = set.world();
    let pos = set.at(DIGGER);
    let frame = Frame::plan(&world, pos, Dir::South, &GANTRY);
    build_frame(set, &frame, &GANTRY_FRAME);
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
    // One continuous curve north-east off the elevated Pad P terrace, staying west of the look line
    // so the camera sweeps smoothly across the courtyard onto the sorting tube.
    let to_sorter = [plastics_bay::END_EYE, lowered([23.0, 4.8, 7.6]), at_sorter];
    let by_magazine = [
        leave_sorter,
        lowered([31.2, 2.8, 5.4]),
        lowered([35.0, 3.0, 5.2]),
    ];
    let over_pit = on_orbit(PIT_LOOK, PIT_OUT, PIT_UP[0], PIT_ARC[0]);
    let west_of_pit = on_orbit(PIT_LOOK, PIT_OUT, PIT_UP[1], PIT_ARC[1]);
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
            flyby(&to_sorter)
                .look(Look::Pan(plastics_bay::END_LOOK, SORTER_LOOK))
                .ease(speeds(0.9, 0.5))
                .secs(6.0),
        )
        .title("cogwork.film.sorting", 4.5)
        .shot(round(SORTER_ARC[0], 110.0, speeds(0.5, 0.15), 2.5))
        .menu("cogwork:sorting_tube", lowered_block(SORTER), 4.0)
        .shot(round(110.0, 100.0, speeds(0.15, 0.15), 4.0))
        .shot(round(100.0, SORTER_ARC[1], speeds(0.15, 0.9), 2.5))
        .cue("prep_swap")
        .title("cogwork.film.t5_magazine", 5.5)
        .shot(
            flyby(&by_magazine)
                .look(Look::Pan(SORTER_LOOK, MAGAZINE_LOOK))
                .ease(speeds(0.9, 0.7))
                .secs(4.5),
        )
        .menu("cogwork:bit_magazine", lowered_block(MAGAZINE_AT), 4.0)
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
                .ease(speeds(1.5, 1.5))
                .secs(7.0),
        )
        .shot(
            dolly(west_of_pit, INTO_PIT)
                .look(Look::Pan(PIT_LOOK, DOWN_PIT))
                .ease(speeds(1.5, 4.0))
                .secs(3.0),
        )
        .title("cogwork.film.deep", 6.0)
        .shot(
            dolly(INTO_PIT, at_drill)
                .look_at(DRILL_LOOK)
                .ease(speeds(4.0, 0.3))
                .secs(13.0),
        )
        .menu("cogwork:excavator", lowered_block(DIGGER), 5.0)
        .shot(hold(at_drill).look_at(DRILL_LOOK).secs(5.0))
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
