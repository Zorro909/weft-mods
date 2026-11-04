//! Three excavators in a row, all facing south toward the camera with their areas behind them
//! to the north: one that puts up its frame on camera, and two already some layers down, whose
//! output runs south and then west through tubes, sped up by gilded ones, to a sorting tube
//! that fills three chests.

use showcase_api::Set;
use weft_sdk::*;

use super::parts::{CHEST, engine, fuel, line, line_of};
use crate::excavator::EXCAVATOR;
use crate::lib::{BlockState, Pos};
use crate::tube::{DIRECTING_TUBE, GILDED_TUBE, SORTING_TUBE, set_sample, sorting::ROW};

/// The one that builds its frame while the camera watches.
pub const BUILDER: Pos = Pos::new(-16, 0, 4);
/// The two already digging, and how deep their pits are when the film starts.
pub const SHALLOW: Pos = Pos::new(0, 0, 4);
pub const DEEP: Pos = Pos::new(16, 0, 4);
const SHALLOW_DEPTH: i32 = 4;
pub const DEEP_DEPTH: i32 = 8;
/// The middle of each area, north of its excavator.
pub const AREA_Z: f64 = -1.5;
/// Where both excavators' output is sorted, and the tube row it arrives along.
pub const SORTER: Pos = Pos::new(-6, 0, 7);
const ROW_Z: i32 = 7;

pub fn build(set: &Set) {
    for at in [BUILDER, SHALLOW, DEEP] {
        set.place(at.into(), EXCAVATOR.with(&[("facing", "south")]));
        for (place, toward) in engines(at) {
            engine(set, place, toward);
        }
    }
    pit(set, SHALLOW, SHALLOW_DEPTH);
    pit(set, DEEP, DEEP_DEPTH);
    set.place(BUILDER.toward(Dir::South).into(), CHEST.id());
    output_lines(set);
    sorting_station(set);
}

/// Starts the two digging excavators and sets the sorting tube's samples.
pub fn start(set: &Set) {
    for at in [SHALLOW, DEEP] {
        engines(at).iter().for_each(|(place, _)| fuel(set, *place));
    }
    let samples = [
        (Dir::North, "base:cobblestone"),
        (Dir::South, "base:dirt"),
        (Dir::East, "base:obsidian"),
    ];
    let world = set.world();
    let sorter = world.block_entity(set.at(SORTER.into()));
    for (side, item) in samples {
        if let Some(item) = Item::lookup(item) {
            set_sample(&world, sorter, side.face() * ROW as u32, item);
        }
    }
}

/// Starts the builder, which then puts up its frame.
pub fn power_builder(set: &Set) {
    engines(BUILDER)
        .iter()
        .for_each(|(place, _)| fuel(set, *place));
}

/// The engines of the excavator at `at`, and where their trunks point: on top and to the east,
/// and to the west too for the two digging ones, which need the power to dig quickly.
fn engines(at: Pos) -> Vec<(Pos, Dir)> {
    let mut out = vec![
        (at.toward(Dir::Up), Dir::Down),
        (at.toward(Dir::East), Dir::West),
    ];
    if at != BUILDER {
        out.push((at.toward(Dir::West), Dir::East));
    }
    out
}

/// The 9 by 9 area behind the excavator at `at` dug out `depth` blocks, as if it had been
/// working a while.
fn pit(set: &Set, at: Pos, depth: i32) {
    let air = Block::lookup("core:air").unwrap_or(0);
    let near_corner = at.add_x(-4).add_z(-10).with_y(-depth);
    let far_corner = at.add_x(4).add_z(-2).with_y(-1);
    set.fill(near_corner.into(), far_corner.into(), air);
}

/// From each digging excavator's front south to the row, then west along it to the sorter; a
/// directing tube where the lines meet sends everything on west.
fn output_lines(set: &Set) {
    let on_row = |x: i32| Pos::new(x, 0, ROW_Z);
    let meet = on_row(SHALLOW.x);
    line(set, SHALLOW.toward(Dir::South), meet.add_z(-1));
    line(set, DEEP.toward(Dir::South), on_row(DEEP.x));
    line(set, on_row(DEEP.x - 1), on_row(11));
    line_of(set, on_row(10), on_row(6), BlockState::of(&GILDED_TUBE));
    line(set, on_row(5), meet.add_x(1));
    set.place(meet.into(), DIRECTING_TUBE.with(&[("out", "west")]));
    line(set, meet.add_x(-1), SORTER.toward(Dir::East));
}

/// The sorting tube with a chest on three sides: cobblestone north, dirt south, the rest west.
fn sorting_station(set: &Set) {
    set.place(SORTER.into(), SORTING_TUBE.id());
    for side in [Dir::North, Dir::South, Dir::West] {
        set.place(SORTER.toward(side).into(), CHEST.id());
    }
}
