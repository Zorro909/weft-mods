//! Burgher houses: tall town dwellings on stone ground floors, their timber-framed upper storeys
//! jutting over the street. Their east and west walls stay plain, so they can stand in rows.

mod roofs;

use roofs::{cross_roof, fill_up, steep_roof};
use structgen_api::Structure;

use crate::cottages::{dormer, ladder};
use crate::draft::{AIR, Draft};
use crate::kit::{Ridge, chimney, gable_roof, log, plinth, slab, slot, stair, timber_wall, window};

pub const BUILDINGS: &[(&str, fn() -> Structure)] = &[
    ("narrow_gable", narrow_gable),
    ("hall_house", hall_house),
    ("corner_house", corner_house),
];

/// 5 x 9, three storeys with the gable to the street: a stone shop floor with open counters
/// either side of the door, two framed storeys each jutting a block further out, and a steep
/// roof with a chimney at the back of the ridge.
fn narrow_gable() -> Structure {
    let mut d = Draft::new([7, 20, 13]);
    d.fill([0, 0, 0], [6, 0, 12], &slot("path"));
    plinth(&mut d, [1, 3], [5, 11], 1);
    stone_storey(&mut d, [1, 3], [5, 11], 3);
    framed_storey(&mut d, [1, 2], [5, 11], 4);
    framed_storey(&mut d, [1, 1], [5, 11], 8);
    brackets(&mut d, [1, 3, 5].map(|x| [x, 3, 2]), "south");
    brackets(&mut d, [1, 3, 5].map(|x| [x, 7, 1]), "south");
    d.fill([2, 12, 2], [4, 12, 10], &slot("floor"));
    steep_roof(&mut d, [0, 6], [0, 12], 12);
    chimney(&mut d, [3, 11], [4, 19]);
    for x in 2..=4 {
        fill_up(&mut d, [x, 1], 13, &slot("gable"));
        fill_up(&mut d, [x, 11], 13, &slot("gable"));
    }
    d.fill([3, 1, 3], [3, 2, 3], AIR);
    for x in [2, 4] {
        d.set([x, 1, 3], &slab("trim/slab", "top"));
        d.set([x, 2, 3], AIR);
        window(&mut d, [x, 6, 2], 2, "north");
        window(&mut d, [x, 10, 1], 2, "north");
    }
    window(&mut d, [3, 14, 1], 2, "north");
    window(&mut d, [4, 2, 11], 1, "south");
    window(&mut d, [4, 6, 11], 2, "south");
    window(&mut d, [4, 10, 11], 2, "south");
    ladder(&mut d, [2, 10], [1, 12], "west");
    furnish_narrow_gable(&mut d);
    d.done()
}

/// A shop with chests and a workbench, a kitchen over it, a bedroom, and a store in the roof.
fn furnish_narrow_gable(d: &mut Draft) {
    d.set([2, 1, 6], "base:crafting_table");
    d.set([4, 1, 8], "base:chest[facing=west]");
    d.set([4, 1, 9], "base:chest[facing=west]");
    d.set([3, 5, 10], "base:furnace[facing=north,lit=false]");
    d.set([4, 5, 10], "base:chest[facing=north]");
    table(d, [3, 5, 6]);
    d.set([4, 9, 4], "base:bed[facing=south,part=foot]");
    d.set([4, 9, 3], "base:bed[facing=south,part=head]");
    d.set([2, 9, 3], "base:chest[facing=east]");
    d.set([3, 13, 6], "base:chest[facing=north]");
    for (pos, facing) in [
        ([1, 2, 2], "north"),
        ([5, 2, 2], "north"),
        ([3, 3, 4], "south"),
        ([4, 3, 10], "north"),
        ([3, 7, 3], "south"),
        ([4, 7, 9], "west"),
        ([3, 11, 2], "south"),
        ([2, 11, 8], "east"),
        ([3, 13, 9], "up"),
    ] {
        d.set(pos, &format!("@light[facing={facing}]"));
    }
}

/// 9 x 9, two storeys and an attic with the ridge along the street: a stone ground floor with
/// the door in the middle, a framed upper storey jutting over it with a row of four windows,
/// and two dormers in the front slope.
fn hall_house() -> Structure {
    let mut d = Draft::new([11, 16, 11]);
    d.fill([0, 0, 0], [10, 0, 10], &slot("path"));
    plinth(&mut d, [1, 2], [9, 9], 1);
    stone_storey(&mut d, [1, 2], [9, 9], 3);
    framed_storey(&mut d, [1, 1], [9, 9], 4);
    brackets(&mut d, [1, 3, 5, 7, 9].map(|x| [x, 3, 1]), "south");
    d.fill([2, 8, 2], [8, 8, 8], &slot("floor"));
    gable_roof(&mut d, Ridge::X, [0, 10], [0, 10], 8);
    for x in [3, 7] {
        dormer(&mut d, [x, 9, 1]);
    }
    d.fill([5, 9, 0], [5, 9, 1], &slot("roof/block"));
    chimney(&mut d, [9, 5], [1, 15]);
    for z in 2..=8 {
        fill_up(&mut d, [1, z], 9, &slot("gable"));
        fill_up(&mut d, [9, z], 9, &slot("gable"));
    }
    d.fill([5, 1, 2], [5, 2, 2], AIR);
    for x in [3, 7] {
        window(&mut d, [x, 2, 2], 1, "north");
        window(&mut d, [x, 2, 9], 1, "south");
    }
    for x in [2, 4, 6, 8] {
        window(&mut d, [x, 6, 1], 2, "north");
    }
    for x in [4, 6] {
        window(&mut d, [x, 6, 9], 2, "south");
    }
    ladder(&mut d, [2, 5], [1, 8], "west");
    furnish_hall_house(&mut d);
    d.done()
}

/// A hall with a hearth and a long table, two beds upstairs, chests in the attic.
fn furnish_hall_house(d: &mut Draft) {
    d.set([8, 1, 5], "base:furnace[facing=west,lit=false]");
    d.set([8, 1, 3], "base:crafting_table");
    d.set([8, 1, 8], "base:chest[facing=west]");
    for x in 4..=6 {
        d.set([x, 1, 6], &slab("trim/slab", "top"));
        d.set([x, 1, 5], &stair("trim/stair", "north", "bottom"));
        d.set([x, 1, 7], &stair("trim/stair", "south", "bottom"));
    }
    for x in [4, 6] {
        d.set([x, 5, 7], "base:bed[facing=north,part=foot]");
        d.set([x, 5, 8], "base:bed[facing=north,part=head]");
        d.set([x, 9, 5], "base:chest[facing=north]");
    }
    d.set([5, 5, 8], "base:chest[facing=north]");
    for (pos, facing) in [
        ([4, 2, 1], "north"),
        ([6, 2, 1], "north"),
        ([2, 3, 3], "south"),
        ([8, 3, 3], "south"),
        ([5, 3, 8], "north"),
        ([3, 7, 2], "south"),
        ([7, 7, 2], "south"),
        ([5, 7, 8], "north"),
        ([8, 10, 5], "west"),
        ([2, 10, 4], "east"),
    ] {
        d.set(pos, &format!("@light[facing={facing}]"));
    }
}

/// 7 x 7, two storeys and an attic for a street corner: the framed upper storey juts out over
/// both streets on a corner post, and two crossing roofs give it gables to the north and east.
fn corner_house() -> Structure {
    let mut d = Draft::new([9, 14, 9]);
    d.fill([0, 0, 0], [8, 0, 8], &slot("path"));
    plinth(&mut d, [1, 2], [6, 7], 1);
    stone_storey(&mut d, [1, 2], [6, 7], 3);
    framed_storey(&mut d, [1, 1], [7, 7], 4);
    d.fill([7, 1, 1], [7, 3, 1], &log("y"));
    brackets(&mut d, [1, 3, 5].map(|x| [x, 3, 1]), "south");
    brackets(&mut d, [3, 5, 7].map(|z| [7, 3, z]), "west");
    d.fill([2, 8, 2], [6, 8, 6], &slot("floor"));
    cross_roof(&mut d, 4, 8);
    chimney(&mut d, [4, 7], [1, 13]);
    for a in 1..=7 {
        fill_up(&mut d, [a, 1], 9, &slot("gable"));
        fill_up(&mut d, [a, 7], 9, &slot("gable"));
        fill_up(&mut d, [7, a], 9, &slot("gable"));
    }
    d.fill([3, 1, 2], [3, 2, 2], AIR);
    window(&mut d, [5, 2, 2], 1, "north");
    window(&mut d, [6, 2, 4], 1, "east");
    window(&mut d, [2, 2, 7], 1, "south");
    for a in [2, 4, 6] {
        window(&mut d, [a, 6, 1], 2, "north");
        window(&mut d, [7, 6, a], 2, "east");
    }
    for x in [2, 6] {
        window(&mut d, [x, 6, 7], 2, "south");
    }
    window(&mut d, [4, 9, 1], 2, "north");
    window(&mut d, [7, 9, 4], 2, "east");
    ladder(&mut d, [3, 6], [1, 8], "south");
    furnish_corner_house(&mut d);
    d.done()
}

/// A workroom with the hearth below, a parlour and bed above, a second bed in the attic.
fn furnish_corner_house(d: &mut Draft) {
    d.set([4, 1, 6], "base:furnace[facing=north,lit=false]");
    d.set([5, 1, 6], "base:crafting_table");
    d.set([5, 1, 3], "base:chest[facing=west]");
    table(d, [3, 5, 3]);
    d.set([5, 5, 5], "base:bed[facing=north,part=foot]");
    d.set([5, 5, 6], "base:bed[facing=north,part=head]");
    d.set([6, 5, 6], "base:chest[facing=west]");
    d.set([5, 9, 4], "base:bed[facing=west,part=foot]");
    d.set([6, 9, 4], "base:bed[facing=west,part=head]");
    d.set([4, 9, 2], "base:chest[facing=south]");
    for (pos, facing) in [
        ([2, 2, 1], "north"),
        ([4, 2, 1], "north"),
        ([3, 3, 3], "south"),
        ([5, 3, 5], "west"),
        ([3, 7, 2], "south"),
        ([6, 7, 3], "west"),
        ([2, 9, 4], "up"),
    ] {
        d.set(pos, &format!("@light[facing={facing}]"));
    }
}

/// A table with a stool either side of it along x, the table at `pos`.
fn table(d: &mut Draft, [x, y, z]: [usize; 3]) {
    d.set([x, y, z], &slab("trim/slab", "top"));
    d.set([x - 1, y, z], &stair("trim/stair", "west", "bottom"));
    d.set([x + 1, y, z], &stair("trim/stair", "east", "bottom"));
}

/// The four walls of a box from `from` to `to` (x, z), each by its two ends.
fn sides(from: [usize; 2], to: [usize; 2]) -> [([usize; 2], [usize; 2]); 4] {
    [
        (from, [to[0], from[1]]),
        ([from[0], to[1]], to),
        (from, [from[0], to[1]]),
        ([to[0], from[1]], to),
    ]
}

/// A ground floor walled in `plinth` from `from` to `to` (x, z), `height` blocks tall, with
/// `plinth/corner` corners.
fn stone_storey(d: &mut Draft, from: [usize; 2], to: [usize; 2], height: usize) {
    for y in 1..=height {
        d.ring(y, from, to, &slot("plinth"));
    }
    for [x, z] in [from, [to[0], from[1]], [from[0], to[1]], to] {
        d.fill([x, 1, z], [x, height, z], &slot("plinth/corner"));
    }
}

/// A timber-framed storey from `from` to `to` (x, z): a ring of lying logs and a floor at `y`,
/// and walls up to the lying logs at `y + 4` that the next storey or the roof rests on.
fn framed_storey(d: &mut Draft, from: [usize; 2], to: [usize; 2], y: usize) {
    d.fill(
        [from[0] + 1, y, from[1] + 1],
        [to[0] - 1, y, to[1] - 1],
        &slot("floor"),
    );
    for z in [from[1], to[1]] {
        d.fill([from[0], y, z], [to[0], y, z], &log("x"));
    }
    for x in [from[0], to[0]] {
        d.fill([x, y, from[1] + 1], [x, y, to[1] - 1], &log("z"));
    }
    for (a, b) in sides(from, to) {
        timber_wall(d, a, b, [y + 1, y + 4], 2);
    }
}

/// Upside-down stairs under a jetty at each of `cells`, their backs against the wall on their
/// `wall` side.
fn brackets<const N: usize>(d: &mut Draft, cells: [[usize; 3]; N], wall: &str) {
    for cell in cells {
        d.set(cell, &stair("trim/stair", wall, "top"));
    }
}
