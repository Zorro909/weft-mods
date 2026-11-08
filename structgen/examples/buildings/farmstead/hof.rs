//! The hof: house and barn under one roof, as farms in the hills are built.

use structgen_api::Structure;

use super::{ladder, light};
use crate::draft::{AIR, Draft};
use crate::kit::{chimney, plinth, slab, slot, stair, timber_wall, window};

/// 11 x 15 inside its walls: house and barn under one huge hipped roof whose eaves come down to
/// the top of the ground floor. The house end faces the front with its door and windows, the
/// barn end has a wide doorway at the back.
pub fn hof() -> Structure {
    let mut d = Draft::new([13, 10, 17]);
    d.default("wall/infill", "base:oak_planks");
    plinth(&mut d, [1, 1], [11, 15], 1);
    hof_walls(&mut d);
    hip_roof(&mut d, [0, 0], [12, 16], 3);
    d.fill([2, 4, 2], [10, 4, 14], &slot("floor"));
    d.fill([4, 1, 6], [4, 4, 6], &ladder("south"));
    chimney(&mut d, [10, 4], [2, 7]);
    furnish_house(&mut d);
    furnish_byre(&mut d);
    d.done()
}

/// Timber walls a storey high, the house cut off from the byre by a wall with a doorway, the
/// front door and windows in the house end and the barn doorway in the back.
fn hof_walls(d: &mut Draft) {
    for (from, to) in [
        ([1, 1], [11, 1]),
        ([1, 15], [11, 15]),
        ([1, 1], [1, 15]),
        ([11, 1], [11, 15]),
        ([1, 7], [11, 7]),
    ] {
        timber_wall(d, from, to, [1, 3], 2);
    }
    d.fill([6, 1, 1], [6, 2, 1], AIR);
    d.fill([6, 1, 7], [6, 2, 7], AIR);
    d.fill([5, 1, 15], [7, 3, 15], AIR);
    for x in [4, 8] {
        window(d, [x, 2, 1], 1, "north");
    }
    for z in [2, 4, 10, 12] {
        window(d, [1, 2, z], 1, "west");
    }
    window(d, [11, 2, 2], 1, "east");
    window(d, [11, 2, 12], 1, "east");
}

/// A hipped roof of `roof/stair` from `from` to `to` (x, z, eaves included), its eaves at height
/// `y`: every side slopes up, the hips on corner stairs, to a `roof/slab` ridge when the narrow
/// side is odd.
fn hip_roof(d: &mut Draft, from: [usize; 2], to: [usize; 2], y: usize) {
    let mut k = 0;
    while from[0] + k <= to[0] - k && from[1] + k <= to[1] - k {
        let (low, high) = ([from[0] + k, from[1] + k], [to[0] - k, to[1] - k]);
        if low[0] == high[0] || low[1] == high[1] {
            let ridge = slab("roof/slab", "bottom");
            d.fill([low[0], y + k, low[1]], [high[0], y + k, high[1]], &ridge);
            return;
        }
        hip_ring(d, low, high, y + k);
        k += 1;
    }
}

/// One course of a hipped roof: stairs along the four edges from `low` to `high` (x, z), their
/// backs to the middle, and an outer corner stair on each hip.
fn hip_ring(d: &mut Draft, low: [usize; 2], high: [usize; 2], y: usize) {
    let edges = [
        ([low[0], low[1]], [high[0], low[1]], "south"),
        ([low[0], high[1]], [high[0], high[1]], "north"),
        ([low[0], low[1]], [low[0], high[1]], "east"),
        ([high[0], low[1]], [high[0], high[1]], "west"),
    ];
    for (from, to, facing) in edges {
        let edge = stair("roof/stair", facing, "bottom");
        d.fill([from[0], y, from[1]], [to[0], y, to[1]], &edge);
    }
    // A corner's high quarter points into the roof: `outer_left` turns it from the stair's
    // facing toward its counter-clockwise side.
    let corners = [
        ([low[0], low[1]], "south", "outer_left"),
        ([high[0], low[1]], "south", "outer_right"),
        ([low[0], high[1]], "north", "outer_right"),
        ([high[0], high[1]], "north", "outer_left"),
    ];
    for ([x, z], facing, shape) in corners {
        let corner = format!("@roof/stair[facing={facing},half=bottom,shape={shape}]");
        d.set([x, y, z], &corner);
    }
}

/// The house: a hearth under the chimney, table, workbench and chest below, beds in the roof.
fn furnish_house(d: &mut Draft) {
    d.set([10, 1, 4], "base:furnace[facing=west,lit=false]");
    d.set([2, 1, 2], "base:crafting_table");
    d.set([9, 1, 2], "base:chest[facing=south]");
    d.set([3, 1, 4], &slab("trim/slab", "top"));
    d.set([2, 1, 4], &stair("trim/stair", "west", "bottom"));
    for x in [6, 8] {
        d.set([x, 5, 3], "base:bed[facing=south,part=head]");
        d.set([x, 5, 4], "base:bed[facing=south,part=foot]");
    }
    d.set([5, 2, 6], &light("north"));
    d.set([4, 5, 4], &light("up"));
    for x in [5, 7] {
        d.set([x, 2, 0], &light("north"));
    }
}

/// The byre: a trough in the floor, feed chests, and torches by the doorway and in the loft.
fn furnish_byre(d: &mut Draft) {
    d.fill([2, 0, 9], [2, 0, 13], "base:water");
    for z in [9, 13] {
        d.set([10, 1, z], "base:chest[facing=west]");
    }
    d.set([7, 2, 8], &light("south"));
    d.set([6, 5, 12], &light("up"));
    for x in [4, 8] {
        d.set([x, 2, 16], &light("south"));
    }
}
