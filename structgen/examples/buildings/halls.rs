//! The guildhall and the town hall, facing a town's market square.

mod town_hall;

use structgen_api::Structure;

use crate::chapels::stone_walls;
use crate::draft::{AIR, Draft};
use crate::kit::{
    Building, Ridge, chimney, gable_end, gable_roof, log, plinth, side_windows, slab, slot, stair,
    timber_wall, window,
};

pub const BUILDINGS: &[Building] = &[
    ("guildhall", guildhall),
    ("town_hall", town_hall::town_hall),
];

/// 15 x 11, two storeys: a close-studded upper storey jutting over the street from a stone
/// ground floor, two tall cross gables on the front and a red tiled roof; the guild's great
/// hall fills the upper storey under the open roof.
fn guildhall() -> Structure {
    let mut d = Draft::new([17, 17, 13]);
    civic_roof(&mut d);
    plinth(&mut d, [1, 2], [15, 11], 2);
    stone_walls(&mut d, [1, 2], [15, 11], [1, 3]);
    upper_storey(&mut d, [1, 1], [15, 11], 4);
    gable_roof(&mut d, Ridge::X, [0, 16], [0, 12], 8);
    for x in [1, 15] {
        gable_end(&mut d, Ridge::X, x, [0, 12], [1, 11], 8, &slot("gable"));
        d.fill([x, 9, 6], [x, 11, 6], &slot("window"));
    }
    for x in [1, 11] {
        cross_gable(&mut d, [x, 0], 8);
    }
    guildhall_openings(&mut d);
    chimney(&mut d, [8, 11], [1, 16]);
    guildhall_stair(&mut d);
    furnish_guildhall(&mut d);
    d.done()
}

/// Red tile on the roof, as civic buildings wear it.
pub fn civic_roof(d: &mut Draft) {
    d.default("roof/stair", "base:bricks_stairs");
    d.default("roof/slab", "base:bricks_slab");
    d.default("roof/block", "base:bricks");
}

/// A timber-framed storey over the ring from `from` to `to` (x, z): a lying log along its floor
/// line at height `y`, posts every 2 blocks to a lying log 3 above, a `floor` inside.
pub fn upper_storey(d: &mut Draft, from: [usize; 2], to: [usize; 2], y: usize) {
    d.fill([from[0], y, from[1]], [to[0], y, to[1]], &slot("floor"));
    d.fill([from[0], y, from[1]], [to[0], y, from[1]], &log("x"));
    d.fill([from[0], y, to[1]], [to[0], y, to[1]], &log("x"));
    d.fill([from[0], y, from[1]], [from[0], y, to[1]], &log("z"));
    d.fill([to[0], y, from[1]], [to[0], y, to[1]], &log("z"));
    for (a, b) in [
        (from, [to[0], from[1]]),
        ([from[0], to[1]], to),
        (from, [from[0], to[1]]),
        ([to[0], from[1]], to),
    ] {
        timber_wall(d, a, b, [y + 1, y + 3], 2);
    }
}

/// A gabled bay 5 wide from `x` on the front of a roof whose eaves are at `y` over row `z`:
/// its walls stand a storey above those eaves and its own roof runs back into the main one,
/// the valleys in inner corners.
pub fn cross_gable(d: &mut Draft, [x, z]: [usize; 2], y: usize) {
    for c in x..=x + 4 {
        for i in 0..bay_height(x, c, y) - y {
            d.set([c, y + i, z + i], AIR);
        }
    }
    timber_wall(d, [x, z + 1], [x + 4, z + 1], [y, y + 1], 2);
    for c in x - 1..=x + 5 {
        let height = bay_height(x, c, y);
        let valley = z + height - y;
        let (facing, inner) = if c < x + 2 {
            ("east", "inner_left")
        } else {
            ("west", "inner_right")
        };
        let roof = if c == x + 2 {
            slab("roof/slab", "bottom")
        } else {
            stair("roof/stair", facing, "bottom")
        };
        d.fill([c, height, z], [c, height, valley - 1], &roof);
        if c != x + 2 {
            let corner = format!("@roof/stair[facing=south,half=bottom,shape={inner}]");
            d.set([c, height, valley], &corner);
        }
    }
    gable_end(
        d,
        Ridge::Z,
        z + 1,
        [x - 1, x + 5],
        [x, x + 4],
        y + 2,
        &slot("gable"),
    );
    d.fill(
        [x + 2, y + 2, z + 1],
        [x + 2, y + 3, z + 1],
        &slot("window"),
    );
    for c in [x + 1, x + 3] {
        d.set([c, y, z + 1], &slot("window"));
    }
}

/// How high the roof of the bay from `x` is over column `c`: its eaves 2 above the main
/// roof's `y`, its ridge 3 above that.
fn bay_height(x: usize, c: usize, y: usize) -> usize {
    y + 2 + (c + 1 - x).min(x + 5 - c)
}

/// Brackets under the jetty, the doorway in the middle of the front, windows and lights.
fn guildhall_openings(d: &mut Draft) {
    for x in (1..=15).step_by(2) {
        d.set([x, 3, 1], &stair("trim/stair", "south", "top"));
    }
    d.fill([8, 1, 2], [8, 2, 2], AIR);
    for x in [7, 9] {
        d.set([x, 2, 1], "@light[facing=north]");
    }
    for x in [4, 12] {
        window(d, [x, 2, 2], 1, "north");
    }
    for x in [4, 14] {
        window(d, [x, 2, 11], 1, "south");
    }
    for x in [2, 4, 8, 12, 14] {
        d.fill([x, 5, 1], [x, 6, 1], &slot("window"));
    }
    for x in [4, 6, 10, 12] {
        window(d, [x, 5, 11], 2, "south");
    }
    side_windows(d, [1, 15], 5, 2, &[4, 8]);
    side_windows(d, [1, 15], 2, 1, &[5, 8]);
}

/// A flight of stairs up the back wall into the hall, a rail along its well.
fn guildhall_stair(d: &mut Draft) {
    for (i, x) in (11..=14).enumerate() {
        d.set([x, 1 + i, 10], &stair("trim/stair", "east", "bottom"));
    }
    for x in 11..=13 {
        d.set([x, 4, 10], AIR);
        d.set([x, 5, 9], &slab("trim/slab", "bottom"));
    }
}

/// The great hall upstairs, with its long table and hearth; the guild's store below.
fn furnish_guildhall(d: &mut Draft) {
    d.fill([5, 5, 6], [11, 5, 6], &slab("trim/slab", "top"));
    for z in [5, 7] {
        d.fill([5, 5, z], [11, 5, z], &slab("trim/slab", "bottom"));
    }
    d.set([12, 5, 6], &stair("trim/stair", "east", "bottom"));
    d.set([8, 5, 10], "base:furnace[facing=north,lit=true]");
    d.set([8, 1, 10], "base:furnace[facing=north,lit=false]");
    d.set([3, 5, 10], "base:chest[facing=north]");
    for z in [6, 7] {
        d.set([2, 1, z], "base:chest[facing=east]");
    }
    d.set([14, 1, 4], "base:crafting_table");
    d.set([13, 1, 4], "base:woodcutter");
    d.fill([5, 1, 6], [6, 1, 6], &slab("trim/slab", "top"));
    for x in [6, 10] {
        d.set([x, 6, 2], "@light[facing=south]");
        d.set([x, 2, 3], "@light[facing=south]");
        d.set([x, 2, 10], "@light[facing=north]");
    }
    for x in [3, 13] {
        d.set([x, 6, 10], "@light[facing=north]");
    }
    d.set([2, 6, 6], "@light[facing=east]");
    d.set([14, 6, 6], "@light[facing=west]");
}
