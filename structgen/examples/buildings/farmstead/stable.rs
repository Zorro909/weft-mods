//! The stable: stalls under a lean-to roof, open to a paddock in front.

use crate::draft::{AIR, Draft};
use crate::kit::{light, log, plinth, slab, slot, stair, timber_wall, window};
use crate::structure_file::Structure;

/// 15 x 4 under a single roof sloping down to the front, its three stalls open to a paddock
/// walled with low stone walls, a tack room with its own door at the east end. The paddock's
/// gateway is in the middle of its front wall.
pub fn stable() -> Structure {
    let mut d = Draft::new([17, 10, 9]);
    d.default("wall/infill", "base:oak_planks");
    d.fill([0, 0, 0], [16, 0, 8], "base:grass");
    plinth(&mut d, [1, 4], [15, 7], 1);
    paddock(&mut d);
    stable_walls(&mut d);
    for z in 3..=8 {
        d.fill(
            [0, z + 1, z],
            [16, z + 1, z],
            &stair("roof/stair", "south", "bottom"),
        );
    }
    stalls(&mut d);
    tack_room(&mut d);
    d.done()
}

/// Low stone walls capped with slabs around the front and sides of the paddock.
fn paddock(d: &mut Draft) {
    for (from, to) in [([1, 0], [15, 0]), ([1, 0], [1, 3]), ([15, 0], [15, 3])] {
        d.fill([from[0], 1, from[1]], [to[0], 1, to[1]], &slot("plinth"));
        d.fill(
            [from[0], 2, from[1]],
            [to[0], 2, to[1]],
            &slab("plinth/slab", "bottom"),
        );
    }
    for x in [1, 15] {
        d.set([x, 1, 0], &slot("plinth/corner"));
    }
    d.fill([7, 1, 0], [9, 2, 0], AIR);
    d.fill([7, 0, 0], [9, 0, 3], &slot("ground"));
}

/// Posts along the open front under a lying beam, timber end walls following the slope, and the
/// tall back wall.
fn stable_walls(d: &mut Draft) {
    for x in [1, 4, 8, 11, 15] {
        d.fill([x, 1, 4], [x, 3, 4], &log("y"));
    }
    d.fill([1, 4, 4], [15, 4, 4], &log("x"));
    timber_wall(d, [1, 7], [15, 7], [1, 7], 2);
    for x in [1, 15] {
        timber_wall(d, [x, 4], [x, 7], [1, 4], 3);
        d.set([x, 5, 5], &slot("gable"));
        d.fill([x, 5, 6], [x, 6, 6], &slot("gable"));
    }
    for x in [2, 6, 10] {
        window(d, [x, 4, 7], 2, "south");
    }
}

/// Plank dividers between the stalls, and a trough sunk in the floor of each.
fn stalls(d: &mut Draft) {
    for x in [4, 8] {
        d.fill([x, 1, 5], [x, 2, 6], &slot("wall/infill"));
        d.fill([x, 3, 5], [x, 3, 6], &slab("trim/slab", "bottom"));
    }
    for x in [2, 6, 10] {
        d.set([x, 0, 6], "base:water");
    }
    for x in [3, 6, 9] {
        d.set([x, 3, 6], &light("north"));
    }
    for x in [4, 8] {
        d.set([x, 2, 3], &light("north"));
    }
}

/// The tack room at the east end, walled off from the last stall, its door to the front.
fn tack_room(d: &mut Draft) {
    timber_wall(d, [11, 4], [15, 4], [1, 4], 4);
    timber_wall(d, [11, 4], [11, 7], [1, 4], 3);
    d.fill([13, 1, 4], [13, 2, 4], AIR);
    d.set([12, 1, 6], "base:chest[facing=north]");
    d.set([14, 1, 6], "base:crafting_table");
    d.set([14, 1, 5], "base:chest[facing=west]");
    d.set([13, 3, 6], &light("north"));
}
