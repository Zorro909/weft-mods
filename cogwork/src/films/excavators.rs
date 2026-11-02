//! Three excavators in a row, all facing north with their areas behind them to the south: one
//! that puts up its frame on camera, and two already some layers down, whose output runs
//! through tubes, sped up by gilded ones, to a sorting tube that fills three chests.

use showcase_api::Set;
use weft_sdk::*;

use super::parts::{CHEST, engine, fuel, line, line_of};
use crate::excavator::EXCAVATOR;
use crate::tube::{DIRECTING_TUBE, GILDED_TUBE, SORTING_TUBE, set_sample, sorting::ROW};

/// The one that builds its frame while the camera watches.
pub const BUILDER: [i32; 3] = [-16, 0, -4];
/// The two already digging, and how deep their pits are when the film starts.
pub const SHALLOW: [i32; 3] = [0, 0, -4];
pub const DEEP: [i32; 3] = [16, 0, -4];
const SHALLOW_DEPTH: i32 = 4;
pub const DEEP_DEPTH: i32 = 8;
/// Where both excavators' output is sorted, and the tube row it arrives along.
pub const SORTER: [i32; 3] = [-6, 0, -8];
const ROW_Z: i32 = -8;

pub fn build(set: &Set) {
    for at in [BUILDER, SHALLOW, DEEP] {
        set.place(at, EXCAVATOR.with(&[("facing", "north")]));
        for (place, toward) in engines(at) {
            engine(set, place, toward);
        }
    }
    pit(set, SHALLOW, SHALLOW_DEPTH);
    pit(set, DEEP, DEEP_DEPTH);
    set.place(beside(BUILDER, Dir::North), CHEST.id());
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
    let sorter = world.block_entity(set.at(SORTER));
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
fn engines(at: [i32; 3]) -> Vec<([i32; 3], Dir)> {
    let mut out = vec![
        (beside(at, Dir::Up), Dir::Down),
        (beside(at, Dir::East), Dir::West),
    ];
    if at != BUILDER {
        out.push((beside(at, Dir::West), Dir::East));
    }
    out
}

/// The 9 by 9 area behind the excavator at `at` dug out `depth` blocks, as if it had been
/// working a while.
fn pit(set: &Set, at: [i32; 3], depth: i32) {
    let air = Block::lookup("core:air").unwrap_or(0);
    set.fill(
        [at[0] - 4, -depth, at[2] + 2],
        [at[0] + 4, -1, at[2] + 10],
        air,
    );
}

/// From each digging excavator's front north to the row, then west along it to the sorter; a
/// directing tube where the lines meet sends everything on west.
fn output_lines(set: &Set) {
    line(set, beside(SHALLOW, Dir::North), [SHALLOW[0], 0, ROW_Z + 1]);
    line(set, beside(DEEP, Dir::North), [DEEP[0], 0, ROW_Z]);
    line(set, [DEEP[0] - 1, 0, ROW_Z], [11, 0, ROW_Z]);
    line_of(set, [10, 0, ROW_Z], [6, 0, ROW_Z], GILDED_TUBE.id());
    line(set, [5, 0, ROW_Z], [SHALLOW[0] + 1, 0, ROW_Z]);
    set.place(
        [SHALLOW[0], 0, ROW_Z],
        DIRECTING_TUBE.with(&[("out", "west")]),
    );
    line(set, [SHALLOW[0] - 1, 0, ROW_Z], beside(SORTER, Dir::East));
}

/// The sorting tube with a chest on three sides: cobblestone north, dirt south, the rest west.
fn sorting_station(set: &Set) {
    set.place(SORTER, SORTING_TUBE.id());
    for side in [Dir::North, Dir::South, Dir::West] {
        set.place(beside(SORTER, side), CHEST.id());
    }
}

pub fn beside(at: [i32; 3], side: Dir) -> [i32; 3] {
    let o = side.offset();
    [at[0] + o[0], at[1] + o[1], at[2] + o[2]]
}
