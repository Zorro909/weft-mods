//! The coaching inn: a town's great inn on the high street, with stabling round a yard behind.

use structgen_api::Structure;

use super::{bed, cross_gable, floor_line, open_side, stone_walls, table, timber_storey};
use crate::draft::{AIR, Draft};
use crate::kit::{
    Ridge, chimney, gable_end, gable_roof, ladder_up, light_at, plinth, slab, slot, stair,
    timber_wall, window,
};

/// 15 x 15 in an L: a range of three storeys on the street, jettied twice, with three dormers in
/// its roof and a carriage arch at its west end into a walled yard, and a stable wing with a
/// loft along the yard's east side.
pub fn coaching_inn() -> Structure {
    let mut d = Draft::new([17, 19, 17]);
    yard(&mut d);
    stable_wing(&mut d);
    furnish_stable(&mut d);
    range_storeys(&mut d);
    range_roof(&mut d);
    carriage_arch(&mut d);
    range_openings(&mut d);
    furnish_taproom(&mut d);
    furnish_first_floor(&mut d);
    furnish_second_floor(&mut d);
    furnish_attic(&mut d);
    d.done()
}

/// The ground before, under and behind the inn, the yard's wall with a back gate, and a
/// trough.
fn yard(d: &mut Draft) {
    d.fill([0, 0, 0], [16, 0, 16], &slot("ground"));
    d.fill([1, 1, 15], [10, 2, 15], &slot("plinth"));
    d.fill([1, 1, 10], [1, 2, 15], &slot("plinth"));
    d.fill([6, 1, 15], [7, 2, 15], AIR);
    d.fill([7, 0, 12], [8, 0, 12], "base:water");
    for pos in [[1, 3, 12], [5, 3, 15], [8, 3, 15]] {
        light_at(d, pos, "up");
    }
}

/// Behind the east end: a stone stable open to the yard on log posts, with a groom's loft above
/// under a roof running south.
fn stable_wing(d: &mut Draft) {
    stone_walls(d, [11, 9], [15, 15], [0, 3]);
    open_side(d, [11, 10], [11, 14], [1, 3], 2);
    floor_line(d, [11, 9], [15, 15], 4);
    for (from, to) in [
        ([11, 9], [11, 15]),
        ([15, 9], [15, 15]),
        ([11, 15], [15, 15]),
    ] {
        timber_wall(d, from, to, [5, 7], 2);
    }
    gable_roof(d, Ridge::Z, [10, 16], [10, 16], 8);
    gable_end(d, Ridge::Z, 15, [10, 16], [11, 15], 8, &slot("gable"));
    for a in [12, 14] {
        window(d, [a, 6, 15], 1, "south");
        window(d, [11, 6, a], 1, "west");
        window(d, [15, 6, a], 1, "east");
    }
}

/// Two stalls with sunken troughs, a rail between them, and a loft with beds for the grooms.
fn furnish_stable(d: &mut Draft) {
    for z in [11, 14] {
        d.set([14, 0, z], "base:water");
    }
    d.fill([12, 1, 12], [14, 1, 12], &slab("trim/slab", "top"));
    light_at(d, [14, 2, 13], "west");
    for z in [11, 13] {
        bed(d, [13, 5, z], "west");
    }
    d.set([12, 5, 14], "base:chest[facing=east]");
    light_at(d, [12, 6, 10], "south");
}

/// Stone below and two timber-framed storeys above, each jutting a block further over the street
/// on brackets, and the attic's floor.
fn range_storeys(d: &mut Draft) {
    plinth(d, [1, 3], [15, 9], 0);
    stone_walls(d, [1, 3], [15, 9], [1, 3]);
    floor_line(d, [1, 2], [15, 9], 4);
    timber_storey(d, [1, 2], [15, 9], [5, 7], 2);
    floor_line(d, [1, 1], [15, 9], 8);
    timber_storey(d, [1, 1], [15, 9], [9, 11], 2);
    d.fill([1, 12, 1], [15, 12, 9], &slot("floor"));
    let bracket = stair("trim/stair", "south", "top");
    for x in [1, 5, 11, 15] {
        d.set([x, 3, 2], &bracket);
        d.set([x, 7, 1], &bracket);
    }
}

/// A roof gabled east and west with three dormers over the street and a chimney in each gable.
fn range_roof(d: &mut Draft) {
    gable_roof(d, Ridge::X, [0, 16], [0, 10], 12);
    for x in [1, 15] {
        gable_end(d, Ridge::X, x, [0, 10], [1, 9], 12, &slot("gable"));
    }
    for x in [4, 8, 12] {
        dormer(d, x, 2, [0, 12]);
    }
    chimney(d, [15, 6], [1, 18]);
    chimney(d, [1, 6], [5, 18]);
}

/// A dormer at column `x`, its window at `z` in the north slope of a roof that rises from height
/// `slope[1]` at `slope[0]` (z).
fn dormer(d: &mut Draft, x: usize, z: usize, slope: [usize; 2]) {
    let y = slope[1] + z - slope[0];
    cross_gable(d, [x - 1, x + 1], z, y + 1, slope);
    // Its roof hangs a block over the window.
    d.set(
        [x - 1, y + 1, z - 1],
        &stair("roof/stair", "east", "bottom"),
    );
    d.set(
        [x + 1, y + 1, z - 1],
        &stair("roof/stair", "west", "bottom"),
    );
    d.set([x, y + 2, z - 1], &slab("roof/slab", "bottom"));
    for pos in [[x - 1, y, z], [x + 1, y, z], [x, y + 1, z]] {
        d.set(pos, &slot("gable"));
    }
    d.set([x, y, z], &slot("window"));
}

/// A carriage arch through the west end of the range into the yard, walled off from the taproom.
fn carriage_arch(d: &mut Draft) {
    d.fill([2, 0, 3], [4, 0, 9], &slot("ground"));
    d.fill([2, 1, 3], [4, 3, 9], AIR);
    d.fill([5, 1, 3], [5, 3, 9], &slot("plinth"));
    for z in [3, 9] {
        d.set([2, 3, z], &stair("plinth/stair", "west", "top"));
        d.set([4, 3, z], &stair("plinth/stair", "east", "top"));
    }
    light_at(d, [4, 2, 6], "west");
}

/// The door in the middle of the front between two torches, windows in every storey, and a
/// doorway from the first floor into the stable loft.
fn range_openings(d: &mut Draft) {
    d.fill([9, 1, 3], [9, 2, 3], AIR);
    for x in [8, 10] {
        light_at(d, [x, 2, 2], "north");
    }
    for x in [7, 11, 13] {
        window(d, [x, 2, 3], 1, "north");
    }
    for x in [2, 4, 8, 12, 14] {
        window(d, [x, 6, 2], 1, "north");
        window(d, [x, 10, 1], 1, "north");
    }
    for x in [7, 9] {
        window(d, [x, 2, 9], 1, "south");
    }
    for x in [4, 6, 8] {
        window(d, [x, 6, 9], 1, "south");
        window(d, [x, 10, 9], 1, "south");
    }
    for (x, outward) in [(1, "west"), (15, "east")] {
        for z in [3, 5] {
            window(d, [x, 6, z], 1, outward);
        }
        window(d, [x, 10, 4], 1, outward);
        window(d, [x, 2, 4], 1, outward);
    }
    d.fill([13, 5, 9], [13, 6, 9], AIR);
}

/// Two tables, a bar counter before shelves of chests, the hearth and the ladder to every floor.
fn furnish_taproom(d: &mut Draft) {
    table(d, [10, 11], 1, 5);
    table(d, [12, 13], 1, 7);
    d.fill([6, 1, 6], [8, 1, 6], &stair("trim/stair", "south", "top"));
    for x in [6, 8] {
        d.set([x, 1, 8], "base:chest[facing=north]");
    }
    d.set([7, 1, 8], "base:crafting_table");
    d.set([14, 1, 6], "base:furnace[facing=west,lit=true]");
    ladder_up(d, [14, 7], [1, 12], "east");
    for (pos, facing) in [
        ([6, 2, 4], "south"),
        ([12, 2, 4], "south"),
        ([8, 2, 8], "north"),
    ] {
        light_at(d, pos, facing);
    }
}

/// A dining room by the west hearth and a guest room with two beds.
fn furnish_first_floor(d: &mut Draft) {
    d.set([2, 5, 6], "base:furnace[facing=east,lit=true]");
    table(d, [5, 6], 5, 5);
    partition(d, 9, [3, 8], [5, 7]);
    for x in [11, 13] {
        bed(d, [x, 5, 4], "south");
    }
    d.set([12, 5, 3], "base:chest[facing=south]");
    for (pos, facing) in [
        ([2, 6, 4], "east"),
        ([8, 6, 4], "west"),
        ([10, 6, 8], "north"),
    ] {
        light_at(d, pos, facing);
    }
}

/// Three guest rooms in a row, through doorways in the partitions.
fn furnish_second_floor(d: &mut Draft) {
    for x in [5, 10] {
        partition(d, x, [2, 8], [9, 11]);
        light_at(d, [x - 1, 10, 6], "west");
        light_at(d, [x + 1, 10, 6], "east");
    }
    for x in [3, 7, 9, 12] {
        bed(d, [x, 9, 3], "south");
    }
    d.set([7, 9, 8], "base:chest[facing=north]");
}

/// A bed under each dormer and chests along the back, lit by torches on the floor.
fn furnish_attic(d: &mut Draft) {
    for x in [4, 8, 12] {
        bed(d, [x, 13, 4], "south");
        d.set([x + 1, 13, 7], "base:chest[facing=north]");
    }
    for x in [6, 10] {
        light_at(d, [x, 13, 6], "up");
    }
}

/// An infill wall across the range at `x`, from `z[0]` to `z[1]` and heights `y[0]` to `y[1]`,
/// with a doorway at z 5.
fn partition(d: &mut Draft, x: usize, z: [usize; 2], y: [usize; 2]) {
    d.fill([x, y[0], z[0]], [x, y[1], z[1]], &slot("wall/infill"));
    d.fill([x, y[0], 5], [x, y[0] + 1, 5], AIR);
}
