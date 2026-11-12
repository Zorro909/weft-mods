//! The two excavators east of the hall, both facing south with their areas behind them. The
//! nearer one stands on the pad next to the hall's, its frame up and its pit some layers deep
//! already: the shaft from the hall drives it from its east side, and its output leaves it to
//! the west through tubes, sped up by gilded ones, to a sorting tube with three chests by the
//! hall's east end. The farther one, beyond the knoll, has ember engines of its own and puts
//! up its frame on camera.

use showcase_api::Set;
use weft_sdk::{Dir, Item};

use super::parts::{CHEST, area, engine, fuel, shafts, tubes};
use crate::excavator::{DIG, EXCAVATOR, Frame, GANTRY_FRAME, Stage, Status};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::tube::{GILDED_TUBE, SORTING_TUBE, set_sample, sorting::ROW};

/// The one digging, and how deep its pit is when the film starts.
const DIGGER: Pos = Pos::new(33, 0, 2);
const PIT_DEPTH: i32 = 5;
/// The one that builds its frame while the camera watches, and the sides its engines are on.
const BUILDER: Pos = Pos::new(64, 1, 3);
const BUILDER_ENGINES: [Dir; 2] = [Dir::Up, Dir::East];
/// How far behind an excavator the middle of its 9 by 9 area lies.
const AREA_BEHIND: i32 = 6;
/// What the camera turns round: the middle of the pit, a block above its lowest layer, and
/// the middle of the frame the builder puts up.
pub const PIT: Pos = Pos::new(DIGGER.x, 1 - PIT_DEPTH, DIGGER.z - AREA_BEHIND);
pub const RAISED: Pos = Pos::new(BUILDER.x, BUILDER.y + 1, BUILDER.z - AREA_BEHIND);
/// Where the digger's output is sorted, west of it in its own row, and the gilded stretch of
/// the tubes between, along x.
pub const SORTER: Pos = Pos::new(26, 0, DIGGER.z);
const GILDED_X: [i32; 2] = [28, 30];
/// The shaft from the hall: the row it leaves the hall in, and the column it turns north in
/// to reach the digger's east side.
const SHAFT_Z: i32 = super::crafting::SHAFT_Z;
const SHAFT_X: i32 = DIGGER.x + 1;

pub fn build(set: &Set) {
    for at in [DIGGER, BUILDER] {
        set.place(at, EXCAVATOR.with(&[("facing", "south")]).into());
    }
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

/// The digger's frame, and the area inside it dug out `PIT_DEPTH` deep, as if it had been
/// working a while.
fn frame_and_pit(set: &Set) {
    let world = set.world();
    let pos = set.at(DIGGER);
    let frame = Frame::plan(&world, pos, Dir::South);
    for cell in frame.cells() {
        world.set_block_typed(cell, BlockState::of(&GANTRY_FRAME));
    }
    let lowest = frame.lo.add_x(1).add_z(1).with_y(pos.y - PIT_DEPTH);
    let highest = frame.hi.add_x(-1).add_z(-1).with_y(pos.y - 1);
    for block in area(lowest, highest).blocks() {
        world.set_block_typed(block, BlockState::AIR);
    }
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

/// Sets the sorting tube's samples, and puts the digger to mining inside the frame it stands
/// in; it waits for the shaft to turn.
pub fn start(set: &Set) {
    mine(set);
    let samples = [(Dir::North, "base:cobblestone"), (Dir::South, "base:dirt")];
    let world = set.world();
    let sorter = world.block_entity(set.at(SORTER));
    for (side, item) in samples {
        if let Some(item) = Item::lookup(item) {
            set_sample(&world, sorter, side.face() * ROW as u32, item);
        }
    }
}

/// Has the digger take the frame around it for its own, built, and start mining from its top.
fn mine(set: &Set) {
    let world = set.world();
    let pos = set.at(DIGGER);
    let frame = Frame::plan(&world, pos, Dir::South);
    let Some(dig) = world.component_mut(world.block_entity(pos), &DIG) else {
        return;
    };
    dig.frame = frame;
    dig.step = frame.cells().len() as u32;
    dig.set_stage(Stage::Mine);
    dig.set_status(Status::NoPower);
    dig.cursor = frame.first();
    dig.set_head(dig.cursor.floor_middle());
}

/// Lights the builder's engines: it puts up its frame.
pub fn raise_frame(set: &Set) {
    for side in BUILDER_ENGINES {
        fuel(set, BUILDER.toward(side));
    }
}
