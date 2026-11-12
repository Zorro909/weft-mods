//! The town church: a buttressed stone nave with a battlemented tower at the front.

use super::{Room, altar, nave_and_chancel, pew, stone_walls};
use crate::draft::{AIR, Draft};
use crate::kit::{ladder, log, plinth, slab, slot, stair, window};
use crate::structure_file::Structure;

/// An 11 x 13 nave of stone between buttresses, lit by tall pointed windows, a chancel behind
/// it, and a square tower at the front, 25 tall with battlements, the doorway at its foot.
pub fn town_church() -> Structure {
    let mut d = Draft::new([13, 25, 23]);
    d.fill([0, 0, 0], [12, 0, 22], &slot("ground"));
    let nave = Room {
        from: [1, 5],
        to: [11, 17],
        top: 7,
    };
    let chancel = Room {
        from: [3, 17],
        to: [9, 21],
        top: 6,
    };
    nave_and_chancel(&mut d, nave, chancel);
    d.fill([5, 1, 17], [7, 5, 17], AIR);
    for z in [6, 10, 14] {
        pointed_window(&mut d, [1, z], "west");
        pointed_window(&mut d, [11, z], "east");
    }
    for z in [5, 9, 13, 17] {
        buttress(&mut d, [0, z], "east");
        buttress(&mut d, [12, z], "west");
    }
    chancel_windows(&mut d);
    tower(&mut d);
    furnish_town_church(&mut d);
    d.done()
}

/// A window 3 wide and 4 tall in the wall along z at `x`, from `z` on, its middle a block
/// taller so it ends in a point.
fn pointed_window(d: &mut Draft, [x, z]: [usize; 2], outward: &str) {
    for c in z..z + 3 {
        window(d, [x, 2, c], 4, outward);
    }
    d.set([x, 6, z + 1], &slot("window"));
}

/// A stepped stone buttress at (`x`, `z`), leaning on the wall to its `wall` side.
fn buttress(d: &mut Draft, [x, z]: [usize; 2], wall: &str) {
    d.fill([x, 1, z], [x, 5, z], &slot("plinth/corner"));
    d.set([x, 6, z], &stair("plinth/stair", wall, "bottom"));
}

/// Stained glass behind the altar and a plain window either side of it.
fn chancel_windows(d: &mut Draft) {
    for (x, height, glass) in [(5, 4, "red"), (6, 6, "blue"), (7, 4, "red")] {
        d.fill(
            [x, 2, 21],
            [x, 1 + height, 21],
            &format!("base:{glass}_stained_glass"),
        );
    }
    window(d, [3, 2, 19], 3, "west");
    window(d, [9, 2, 19], 3, "east");
}

/// The tower at the front: the porch at its foot, a bell chamber with louvres, battlements with
/// corner pinnacles, and a ladder to the top.
fn tower(d: &mut Draft) {
    plinth(d, [4, 1], [8, 5], 0);
    stone_walls(d, [4, 1], [8, 5], [1, 21]);
    d.fill([5, 1, 2], [7, 20, 4], AIR);
    d.fill([5, 21, 2], [7, 21, 4], &slab("plinth/slab", "top"));
    d.fill([5, 16, 2], [7, 16, 4], &slot("floor"));
    d.fill([5, 1, 1], [7, 3, 1], AIR);
    d.set([6, 4, 1], AIR);
    d.fill([6, 1, 5], [6, 3, 5], AIR);
    corbels(d, 21, [3, 0], [9, 6]);
    d.fill([6, 7, 1], [6, 10, 1], &slot("window"));
    for [x, z] in [[6, 1], [6, 5], [4, 3], [8, 3]] {
        d.fill([x, 17, z], [x, 18, z], AIR);
    }
    d.fill([6, 19, 2], [6, 19, 4], &log("z"));
    d.set([6, 18, 3], "base:gold_block");
    d.fill([5, 1, 3], [5, 21, 3], &ladder("west"));
    battlements(d, 22, [4, 1], [8, 5]);
    for y in [6, 12, 17] {
        d.set([7, y, 3], "@light[facing=west]");
    }
}

/// A ring of upturned `plinth/stair` from `from` to `to` (x, z) at height `y`, carrying the
/// parapet out over the wall below, with `plinth/slab` at its corners.
fn corbels(d: &mut Draft, y: usize, from: [usize; 2], to: [usize; 2]) {
    let corbel = |facing| stair("plinth/stair", facing, "top");
    for x in from[0]..=to[0] {
        d.set([x, y, from[1]], &corbel("south"));
        d.set([x, y, to[1]], &corbel("north"));
    }
    for z in from[1]..=to[1] {
        d.set([from[0], y, z], &corbel("east"));
        d.set([to[0], y, z], &corbel("west"));
    }
    for [x, z] in [from, [to[0], from[1]], [from[0], to[1]], to] {
        d.set([x, y, z], &slab("plinth/slab", "top"));
    }
}

/// Merlons on every other block of the ring from `from` to `to` (x, z) at height `y`, and a
/// pinnacle on each corner.
fn battlements(d: &mut Draft, y: usize, from: [usize; 2], to: [usize; 2]) {
    for x in from[0]..=to[0] {
        for z in from[1]..=to[1] {
            let edge = x == from[0] || x == to[0] || z == from[1] || z == to[1];
            if edge && (x - from[0] + z - from[1]).is_multiple_of(2) {
                d.set([x, y, z], &slot("plinth"));
            }
        }
    }
    for [x, z] in [from, [to[0], from[1]], [from[0], to[1]], to] {
        d.set([x, y + 1, z], &slot("plinth/corner"));
        d.set([x, y + 2, z], &slab("plinth/slab", "bottom"));
    }
}

/// Pews in two blocks down the middle, columns between the nave and its aisles, an altar
/// behind a rail, and torches.
fn furnish_town_church(d: &mut Draft) {
    for z in [7, 9, 11, 13, 15] {
        pew(d, [4, 5], z);
        pew(d, [7, 8], z);
    }
    for z in [9, 13] {
        for x in [3, 9] {
            d.fill([x, 1, z], [x, 7, z], &slot("plinth/corner"));
        }
        d.fill([1, 8, z], [11, 8, z], &log("x"));
        d.set([2, 5, z], "@light[facing=east]");
        d.set([10, 5, z], "@light[facing=west]");
    }
    altar(d, [5, 7], 20);
    for x in [4, 5, 7, 8] {
        d.set([x, 1, 18], &slab("trim/slab", "bottom"));
    }
    for x in [4, 8] {
        d.set([x, 3, 0], "@light[facing=north]");
    }
}
