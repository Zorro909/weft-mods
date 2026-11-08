//! The farmstead: barns, granaries, stables and fields on the edge of a settlement.

mod barns;
mod hof;
mod stable;

use structgen_api::Structure;

use crate::draft::{AIR, Draft};
use crate::kit::log;

pub const BUILDINGS: &[(&str, fn() -> Structure)] = &[
    ("great_barn", barns::great_barn),
    ("staddle_granary", barns::staddle_granary),
    ("stable", stable::stable),
    ("hof", hof::hof),
    ("wheat_field", wheat_field),
];

/// A `light` leaning out from the wall behind it toward `facing`, or standing on the block
/// below (`up`).
fn light(facing: &str) -> String {
    format!("@light[facing={facing}]")
}

/// A ladder against the `side` of its cell.
fn ladder(side: &str) -> String {
    let on = |s: &str| s == side;
    format!(
        "base:ladder[north={},south={},west={},east={}]",
        on("north"),
        on("south"),
        on("west"),
        on("east")
    )
}

/// 7 x 7 of ripe wheat on farmland, split by a water channel and edged with logs lying on the
/// ground, a torch on two corner posts. Its bottom layer is the farmland, level with the ground.
fn wheat_field() -> Structure {
    let mut d = Draft::new([9, 3, 9]);
    d.fill([1, 0, 1], [7, 0, 7], "base:farmland");
    d.fill([1, 1, 1], [7, 1, 7], "base:wheat[age=7]");
    d.fill([1, 0, 4], [7, 0, 4], "base:water");
    d.fill([1, 1, 4], [7, 1, 4], AIR);
    d.ring(0, [0, 0], [8, 8], "base:dirt");
    for z in [0, 8] {
        d.fill([1, 1, z], [7, 1, z], &log("x"));
    }
    for x in [0, 8] {
        d.fill([x, 1, 1], [x, 1, 7], &log("z"));
    }
    for corner in [[0, 0], [8, 0], [0, 8], [8, 8]] {
        d.set([corner[0], 1, corner[1]], &log("y"));
    }
    d.set([0, 2, 0], &light("up"));
    d.set([8, 2, 8], &light("up"));
    d.done()
}
