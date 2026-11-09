//! Town streets: tiles of 7 by 7 blocks to lay side by side. A street is cobbled from edge to
//! edge, with stone kerbs along both sides and a gutter of slabs down the middle. The tiles open
//! north the way the paths' do (`paths.rs`): straight north and south, corner north and east,
//! tee north, east and south, end north only.
//!
//! A square tile is paving to every edge, so squares of any size are laid from them.

use structgen_api::Structure;

use crate::draft::{AIR, Draft};
use crate::kit::{Building, light, log, slab, stair};

pub const BUILDINGS: &[Building] = &[
    ("street_straight", straight),
    ("street_straight_worn", straight_worn),
    ("street_lamp", lamp),
    ("street_corner", corner),
    ("street_tee", tee),
    ("street_cross", cross),
    ("street_end", end),
    ("street_steps", steps),
    ("street_square", square),
    ("street_square_plain", square_plain),
    ("street_to_square", to_square),
    ("street_to_path", to_path),
];

const STRAIGHT: [&str; 7] = ["kcc_cck"; 7];

fn straight() -> Structure {
    tile(&STRAIGHT)
}

/// Mossy where the traffic doesn't wear it.
fn straight_worn() -> Structure {
    tile(&[
        "kcc_cck", "kcm_cck", "kcc_cmk", "kmc_cck", "kcc_cck", "kcc_cmk", "kcm_cck",
    ])
}

/// A straight with a lamp post on its west kerb.
fn lamp() -> Structure {
    let mut d = draft(&STRAIGHT, 5);
    d.fill([0, 1, 3], [0, 3, 3], &log("y"));
    d.set([0, 4, 3], &light("up"));
    d.done()
}

fn corner() -> Structure {
    tile(&[
        "kcc_cck", "kcc_ccc", "kcc_ccc", "kcc____", "kcccccc", "kkccccc", "kkkkkkk",
    ])
}

fn tee() -> Structure {
    tile(&[
        "kcc_cck", "kcc_ccc", "kcc_ccc", "kcc____", "kcc_ccc", "kcc_ccc", "kcc_cck",
    ])
}

fn cross() -> Structure {
    tile(&[
        "kcc_cck", "ccc_ccc", "ccc_ccc", "_______", "ccc_ccc", "ccc_ccc", "kcc_cck",
    ])
}

/// Closed by a kerb, the gutter running out short of it.
fn end() -> Structure {
    tile(&[
        "kcc_cck", "kcc_cck", "kcc_cck", "kccccck", "kkccckk", "kkkkkkk", ".kkkkk.",
    ])
}

/// Rising a block toward the north: a flight of stairs across the street, the upper half on
/// cobblestone.
fn steps() -> Structure {
    let mut d = Draft::new([7, 3, 7]);
    let under = ["ccccccc"; 3];
    let flight = ["sssssss"];
    let low = ["kcc_cck"; 3];
    let rows: Vec<&str> = under.iter().chain(&flight).chain(&low).copied().collect();
    let (gutter, step) = (gutter(), stair("street/step", "north", "bottom"));
    let mut legend = legend(&gutter);
    legend.push(('s', &step));
    d.paint(0, &rows, &legend);
    d.paint(1, &["kcc_cck", "kcc_cck", "kcc_cck"], &legend);
    d.done()
}

/// Paving in a lattice of kerb stone: laid together, the squares' tiles make diamonds.
fn square() -> Structure {
    tile(&[
        "ccckccc", "cckckcc", "ckcccck", "kccmcck", "ckcccck", "cckckcc", "ccckccc",
    ])
}

fn square_plain() -> Structure {
    tile(&[
        "ccccccc", "cmccccc", "ccccmcc", "ccccccc", "ccmcccc", "cccccmc", "ccccccc",
    ])
}

/// A street from the south opening onto a square to the north.
fn to_square() -> Structure {
    tile(&[
        "ccccccc", "ccccccc", "kcccccc", "kcc_cck", "kcc_cck", "kcc_cck", "kcc_cck",
    ])
}

/// A street from the south narrowing into a path to the north, whose 5-block tile stands in the
/// middle of the street's 7.
fn to_path() -> Structure {
    tile(&[
        "..ggg..", "..ggg..", ".kgggk.", "kcgggck", "kccgcck", "kcc_cck", "kcc_cck",
    ])
}

/// `.` grass (the ground's own top block), `c` paving, `m` a mossy patch, `k` kerb, `_` gutter,
/// `g` a path's gravel.
fn legend(gutter: &str) -> Vec<(char, &str)> {
    vec![
        ('.', "@ground"),
        ('_', gutter),
        ('c', "@street/paving"),
        ('m', "@street/patch"),
        ('k', "@street/kerb"),
        ('g', "@path/surface"),
        (' ', AIR),
    ]
}

/// A tile of one layer and air above, which clears the grass and flowers off the street.
fn tile(rows: &[&str]) -> Structure {
    draft(rows, 2).done()
}

fn draft(rows: &[&str], height: usize) -> Draft {
    let mut d = Draft::new([7, height, 7]);
    d.paint(0, rows, &legend(&gutter()));
    d
}

/// The gutter: a slab down in the paving.
fn gutter() -> String {
    slab("street/gutter", "bottom")
}
