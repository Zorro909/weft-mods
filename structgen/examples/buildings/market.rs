//! The market and the square: stalls, the market hall, the fountain and the well.

mod hall;

use structgen_api::Structure;

use crate::draft::Draft;
use crate::kit::{Building, Ridge, gable_end, gable_roof, log, slab, slot, stair};

/// A building's file name and how it is drawn.
pub const BUILDINGS: &[Building] = &[
    ("draw_well", draw_well),
    ("market_fountain", market_fountain),
    ("market_stall", market_stall),
    ("market_stall_large", market_stall_large),
    ("market_hall", hall::market_hall),
];

const WATER: &str = "base:water";

/// A round stone well in a ring of path, two log posts carrying a windlass beam and a small gabled
/// roof that runs east to west over the water.
fn draw_well() -> Structure {
    let mut d = Draft::new([7, 8, 7]);
    d.fill([0, 0, 0], [6, 0, 6], &slot("ground"));
    round_curb(&mut d);
    d.fill([2, 0, 2], [4, 0, 4], WATER);
    for x in [1, 5] {
        d.fill([x, 2, 3], [x, 6, 3], &log("y"));
        d.set([x, 4, 2], &log("z"));
        d.set([x, 4, 4], &log("z"));
        d.set([x, 3, 2], "@light[facing=north]");
    }
    d.fill([2, 3, 3], [4, 3, 3], &log("x"));
    gable_roof(&mut d, Ridge::X, [0, 6], [1, 5], 5);
    for x in [1, 5] {
        gable_end(&mut d, Ridge::X, x, [1, 5], [2, 4], 5, &slot("gable"));
    }
    d.done()
}

/// The curb around the well, two blocks high with its corners cut off so it reads round.
fn round_curb(d: &mut Draft) {
    for y in 0..=1 {
        d.fill([1, y, 2], [1, y, 4], &slot("plinth"));
        d.fill([5, y, 2], [5, y, 4], &slot("plinth"));
        d.fill([2, y, 1], [4, y, 1], &slot("plinth"));
        d.fill([2, y, 5], [4, y, 5], &slot("plinth"));
    }
}

/// A square stone basin of water on a paved step, its rim capped with slabs, and in its middle a
/// column with a carved capital and a light on top.
fn market_fountain() -> Structure {
    let mut d = Draft::new([9, 6, 9]);
    d.fill([0, 0, 0], [8, 0, 8], &slot("ground"));
    d.fill([1, 0, 1], [7, 0, 7], &slot("plinth"));
    d.ring(1, [1, 1], [7, 7], &slot("plinth"));
    d.ring(2, [1, 1], [7, 7], &slab("plinth/slab", "bottom"));
    for [x, z] in [[1, 1], [7, 1], [1, 7], [7, 7]] {
        d.fill([x, 1, z], [x, 2, z], &slot("plinth/corner"));
    }
    d.fill([2, 1, 2], [6, 1, 6], WATER);
    d.fill([4, 1, 4], [4, 4, 4], &slot("plinth/corner"));
    for facing in ["north", "south", "east", "west"] {
        let [x, z] = match facing {
            "north" => [4, 5],
            "south" => [4, 3],
            "east" => [3, 4],
            _ => [5, 4],
        };
        d.set([x, 4, z], &stair("plinth/stair", facing, "top"));
    }
    d.set([4, 5, 4], "@light[facing=up]");
    d.done()
}

/// A stall on four log posts: a counter at the front, a workbench and a chest at the sides, and a
/// flat awning striped in white wool and roof slabs.
fn market_stall() -> Structure {
    let mut d = Draft::new([3, 4, 3]);
    stall_frame(&mut d, [2, 2], &[0, 2]);
    d.set([1, 1, 0], &stair("trim/stair", "south", "top"));
    d.set([0, 1, 1], "base:crafting_table");
    d.set([2, 1, 1], "base:chest[facing=west]");
    d.set([0, 2, 1], "@light[facing=up]");
    d.done()
}

/// A stall five wide with a timber back wall holding a chest and an oven, a long counter at the
/// front, a workbench at the side, under a striped awning.
fn market_stall_large() -> Structure {
    let mut d = Draft::new([5, 4, 3]);
    stall_frame(&mut d, [4, 2], &[0, 4]);
    d.fill([1, 1, 0], [3, 1, 0], &stair("trim/stair", "south", "top"));
    d.fill([1, 1, 2], [3, 1, 2], &slot("wall/infill"));
    d.fill([1, 2, 2], [3, 2, 2], &log("x"));
    d.set([0, 1, 1], "base:crafting_table");
    d.set([1, 1, 2], "base:chest[facing=north]");
    d.set([3, 1, 2], "base:furnace[facing=north,lit=false]");
    d.set([2, 2, 1], "@light[facing=north]");
    d.done()
}

/// The ground, the corner posts (and any at `posts` along x) and the striped awning of a stall
/// reaching to `far` (x, z).
fn stall_frame(d: &mut Draft, far: [usize; 2], posts: &[usize]) {
    let [fx, fz] = far;
    d.fill([0, 0, 0], [fx, 0, fz], &slot("ground"));
    for &x in posts {
        for z in [0, fz] {
            d.fill([x, 1, z], [x, 2, z], &log("y"));
        }
    }
    for x in 0..=fx {
        let stripe = if x % 2 == 0 {
            "base:wool".to_string()
        } else {
            slab("roof/slab", "top")
        };
        d.fill([x, 3, 0], [x, 3, fz], &stripe);
    }
}
