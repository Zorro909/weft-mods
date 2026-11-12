//! Village paths: tiles of 5 by 5 blocks to lay side by side, a gravel path 3 wide down the
//! middle of each edge it opens on, grass left either side. Every tile is drawn with the north
//! edge open; turned, it serves the other ways.
//!
//! - straight: open north and south;
//! - corner: north and east;
//! - tee: north, east and south;
//! - cross: every side;
//! - end: north only.
//!
//! A cart track is two ruts of worn earth, on x 1 and 3, with grass between.

use crate::draft::{AIR, Draft};
use crate::kit::{Building, stair};
use crate::structure_file::Structure;

pub const BUILDINGS: &[Building] = &[
    ("path_straight", straight),
    ("path_straight_worn", straight_worn),
    ("path_straight_wide", straight_wide),
    ("path_corner", corner),
    ("path_tee", tee),
    ("path_cross", cross),
    ("path_end", end),
    ("path_slope", slope),
    ("track_straight", track_straight),
    ("track_corner", track_corner),
    ("track_to_path", track_to_path),
];

/// `.` grass (the ground's own top block), `g` gravel, `d` worn earth.
const LEGEND: &[(char, &str)] = &[
    ('.', "@ground"),
    ('g', "@path/surface"),
    ('d', "@path/worn"),
];

fn straight() -> Structure {
    tile(&[".ggg.", ".ggg.", ".gg..", ".ggg.", ".ggg."])
}

/// Narrowed by grass growing in, with worn patches.
fn straight_worn() -> Structure {
    tile(&[".ggg.", ".gdd.", "..gd.", ".gg..", ".ggg."])
}

/// Bulging out to the west: room to pass a cart.
fn straight_wide() -> Structure {
    tile(&[".ggg.", "gggg.", "ggdd.", "gggg.", ".ggg."])
}

/// The inner corner rounded off.
fn corner() -> Structure {
    tile(&[".ggg.", ".gggg", ".ggdg", "..ggg", "....."])
}

fn tee() -> Structure {
    tile(&[".ggg.", ".gggg", ".gdgg", ".gggg", ".ggg."])
}

fn cross() -> Structure {
    tile(&[".ggg.", "ggggg", "ggddg", "ggggg", ".ggg."])
}

/// Rounded off: in front of a door, or where the path gives out.
fn end() -> Structure {
    tile(&[".ggg.", ".ggg.", ".dgg.", ".gg..", "....."])
}

/// Rising a block toward the north: a step of stone stairs across the path, the upper half on
/// earth.
fn slope() -> Structure {
    let mut d = Draft::new([5, 3, 5]);
    d.paint(
        0,
        &["ddddd", "ddddd", ".sss.", ".ggg.", ".ggg."],
        &[
            ('.', "@ground"),
            ('g', "@path/surface"),
            ('d', "base:dirt"),
            ('s', &stair("path/step", "north", "bottom")),
        ],
    );
    d.paint(1, &[".ggg.", ".gdg.", "     ", "     ", "     "], &upper());
    d.done()
}

/// The legend one layer up, where a space is air.
fn upper() -> Vec<(char, &'static str)> {
    let mut legend = LEGEND.to_vec();
    legend.push((' ', AIR));
    legend
}

fn track_straight() -> Structure {
    tile(&[".d.d.", ".d.d.", ".dgd.", ".d.d.", ".d.d."])
}

/// From the north round to the east, the outer rut on the long way round.
fn track_corner() -> Structure {
    tile(&[".d.d.", ".d.dd", ".d...", ".dddd", "....."])
}

/// The path to the north, the track to the south: where a village gives way to the fields.
fn track_to_path() -> Structure {
    tile(&[".ggg.", ".ggg.", ".gdg.", ".dgd.", ".d.d."])
}

/// A tile of one layer drawn with `LEGEND`, and air above, which clears the grass and flowers
/// off the path.
fn tile(rows: &[&str]) -> Structure {
    let mut d = Draft::new([5, 2, 5]);
    d.paint(0, rows, LEGEND);
    d.done()
}
