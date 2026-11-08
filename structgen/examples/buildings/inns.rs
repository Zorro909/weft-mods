//! Inns: one per village or town, on the main road. Also the parts inns and smithies share:
//! stone and timber storeys, open sides on posts, cross gables and furniture.

mod coaching_inn;

use std::cmp::Ordering;

use structgen_api::Structure;

use crate::draft::{AIR, Draft};
use crate::kit::{
    Building, Ridge, chimney, gable_end, gable_roof, ladder_up, light_at, log, opposite, plinth,
    slab, slot, stair, step, timber_wall, window,
};

pub const BUILDINGS: &[Building] = &[
    ("wayside_tavern", wayside_tavern),
    ("coaching_inn", coaching_inn::coaching_inn),
];

/// 9 x 9 and two storeys: a stone taproom under a timber-framed upper floor, a cross gable over
/// the door in the middle of the front, and a beer garden under a log pergola before it.
fn wayside_tavern() -> Structure {
    let mut d = Draft::new([11, 16, 15]);
    d.fill([0, 0, 0], [10, 0, 4], &slot("path"));
    plinth(&mut d, [1, 5], [9, 13], 1);
    stone_walls(&mut d, [1, 5], [9, 13], [1, 3]);
    floor_line(&mut d, [1, 5], [9, 13], 4);
    timber_storey(&mut d, [1, 5], [9, 13], [5, 7], 2);
    tavern_bay(&mut d);
    tavern_roof(&mut d);
    tavern_openings(&mut d);
    chimney(&mut d, [9, 9], [1, 15]);
    beer_garden(&mut d);
    furnish_taproom(&mut d);
    furnish_tavern_rooms(&mut d);
    d.done()
}

/// The middle of the upper floor jutting a block over the door, open to the room behind.
fn tavern_bay(d: &mut Draft) {
    floor_line(d, [3, 4], [7, 5], 4);
    timber_wall(d, [3, 4], [7, 4], [5, 7], 2);
    d.fill([4, 5, 5], [6, 6, 5], AIR);
    for x in [4, 6] {
        window(d, [x, 6, 4], 1, "north");
    }
}

/// A roof gabled east and west, crossed by a gable over the bay.
fn tavern_roof(d: &mut Draft) {
    gable_roof(d, Ridge::X, [0, 10], [4, 14], 8);
    cross_gable(d, [2, 8], 3, 8, [4, 8]);
    for x in [1, 9] {
        gable_end(d, Ridge::X, x, [4, 14], [5, 13], 8, &slot("gable"));
    }
    gable_end(d, Ridge::Z, 4, [2, 8], [3, 7], 8, &slot("gable"));
    d.set([5, 9, 4], &slot("window"));
}

/// The door in the middle of the front between two torches, and windows all round.
fn tavern_openings(d: &mut Draft) {
    d.fill([5, 1, 5], [5, 2, 5], AIR);
    for x in [4, 6] {
        light_at(d, [x, 2, 4], "north");
    }
    for x in [3, 7] {
        window(d, [x, 2, 5], 1, "north");
        window(d, [x, 2, 13], 1, "south");
    }
    for x in [2, 8] {
        window(d, [x, 6, 5], 1, "north");
    }
    for x in [2, 4, 6, 8] {
        window(d, [x, 6, 13], 1, "south");
    }
    for z in [7, 11] {
        window(d, [1, 2, z], 1, "west");
        window(d, [9, 2, z], 1, "east");
    }
    for z in [8, 10] {
        window(d, [1, 6, z], 1, "west");
        window(d, [9, 6, z], 1, "east");
    }
}

/// Before the front: a pergola of logs on four posts over two tables with benches, torches on
/// its posts and beam.
fn beer_garden(d: &mut Draft) {
    for x in [1, 3, 7, 9] {
        d.fill([x, 1, 1], [x, 2, 1], &log("y"));
    }
    d.fill([1, 3, 1], [9, 3, 1], &log("x"));
    for x in [1, 3, 5, 7, 9] {
        d.fill([x, 3, 2], [x, 3, 4], &log("z"));
    }
    for x in [1, 9] {
        light_at(d, [x, 2, 0], "north");
    }
    for x in [3, 7] {
        light_at(d, [x, 4, 1], "up");
    }
    table(d, [2, 3], 1, 3);
    table(d, [7, 8], 1, 3);
}

/// Two tables, a bar counter with chests behind it, the hearth and the ladder up.
fn furnish_taproom(d: &mut Draft) {
    table(d, [2, 3], 1, 7);
    table(d, [7, 8], 1, 7);
    d.fill([2, 1, 10], [4, 1, 10], &stair("trim/stair", "south", "top"));
    for x in [2, 3] {
        d.set([x, 1, 12], "base:chest[facing=north]");
    }
    d.set([4, 1, 12], "base:crafting_table");
    d.set([8, 1, 9], "base:furnace[facing=west,lit=true]");
    ladder_up(d, [8, 12], [1, 5], "east");
    for (pos, facing) in [
        ([3, 2, 6], "south"),
        ([7, 2, 6], "south"),
        ([3, 2, 12], "north"),
    ] {
        light_at(d, pos, facing);
    }
}

/// Two guest rooms either side of a partition, with two beds and a chest each.
fn furnish_tavern_rooms(d: &mut Draft) {
    d.fill([2, 5, 9], [8, 7, 9], &slot("wall/infill"));
    d.fill([5, 5, 9], [5, 6, 9], AIR);
    for x in [2, 8] {
        bed(d, [x, 5, 7], "south");
    }
    for z in [10, 12] {
        bed(d, [3, 5, z], "east");
    }
    d.set([5, 5, 4], "base:chest[facing=south]");
    d.set([5, 5, 12], "base:chest[facing=north]");
    for x in [3, 7] {
        light_at(d, [x, 6, 8], "north");
        light_at(d, [x, 6, 10], "south");
    }
}

/// Stone walls round `from`..`to` (x, z), heights `y[0]` to `y[1]`: `plinth`, with
/// `plinth/corner` quoins.
pub fn stone_walls(d: &mut Draft, from: [usize; 2], to: [usize; 2], y: [usize; 2]) {
    for h in y[0]..=y[1] {
        d.ring(h, from, to, &slot("plinth"));
        for [x, z] in corners(from, to) {
            d.set([x, h, z], &slot("plinth/corner"));
        }
    }
}

/// Timber-framed walls round `from`..`to` (x, z), heights `y[0]` to `y[1]`, posts every
/// `spacing` blocks.
pub fn timber_storey(
    d: &mut Draft,
    from: [usize; 2],
    to: [usize; 2],
    y: [usize; 2],
    spacing: usize,
) {
    let [north_east, south_west] = [[to[0], from[1]], [from[0], to[1]]];
    for (a, b) in [
        (from, north_east),
        (south_west, to),
        (from, south_west),
        (north_east, to),
    ] {
        timber_wall(d, a, b, y, spacing);
    }
}

/// The floor of a storey at height `y` over `from`..`to` (x, z): `floor` within a lying log along
/// each wall, standing at the corners where the posts run through.
pub fn floor_line(d: &mut Draft, from: [usize; 2], to: [usize; 2], y: usize) {
    d.fill([from[0], y, from[1]], [to[0], y, to[1]], &slot("floor"));
    for z in [from[1], to[1]] {
        d.fill([from[0], y, z], [to[0], y, z], &log("x"));
    }
    for x in [from[0], to[0]] {
        d.fill([x, y, from[1]], [x, y, to[1]], &log("z"));
    }
    for [x, z] in corners(from, to) {
        d.set([x, y, z], &log("y"));
    }
}

/// A side left open from `from` to `to` (x, z), heights `y[0]` to `y[1]`: log posts every
/// `spacing` blocks under a lying log.
pub fn open_side(d: &mut Draft, from: [usize; 2], to: [usize; 2], y: [usize; 2], spacing: usize) {
    timber_wall(d, from, to, y, spacing);
    let infill = slot("wall/infill");
    for x in from[0].min(to[0])..=from[0].max(to[0]) {
        for z in from[1].min(to[1])..=from[1].max(to[1]) {
            for h in y[0]..=y[1] {
                if d.get([x, h, z]) == infill {
                    d.set([x, h, z], AIR);
                }
            }
        }
    }
}

/// A gable roof across `x[0]`..`x[1]`, its eaves at height `y`, running from `front` (z) south
/// into the north slope of a roof that rises from height `slope[1]` at `slope[0]` (z). It is
/// drawn wherever it stands at least as high as that slope, which is opened beneath it; where
/// it meets the slope its stairs turn into valley corners and its ridge ends in a full block.
pub fn cross_gable(d: &mut Draft, x: [usize; 2], front: usize, y: usize, slope: [usize; 2]) {
    let middle = (x[0] + x[1]) / 2;
    for z in front.. {
        let main = (z >= slope[0]).then(|| slope[1] + z - slope[0]);
        let mut shown = false;
        for c in x[0]..=x[1] {
            let height = y + (c - x[0]).min(x[1] - c);
            if main.is_some_and(|m| m > height) {
                continue;
            }
            if let Some(m) = main.filter(|&m| m < height) {
                d.set([c, m, z], AIR);
            }
            let meets = main == Some(height);
            let name = match c.cmp(&middle) {
                Ordering::Equal if (x[1] - x[0]).is_multiple_of(2) && meets => slot("roof/block"),
                Ordering::Equal if (x[1] - x[0]).is_multiple_of(2) => slab("roof/slab", "bottom"),
                Ordering::Greater => valley_stair("west", "inner_left", meets),
                _ => valley_stair("east", "inner_right", meets),
            };
            d.set([c, height, z], &name);
            shown = true;
        }
        if !shown {
            return;
        }
    }
}

/// A `roof/stair` facing `facing`, turned into the `corner` shape where it meets a slope.
fn valley_stair(facing: &str, corner: &str, meets: bool) -> String {
    let shape = if meets { corner } else { "straight" };
    format!("@roof/stair[facing={facing},half=bottom,shape={shape}]")
}

/// A table from `x[0]` to `x[1]` at height `y` along row `z`, standing on the legs of upside-down
/// stairs at its ends, with a bench on either long side.
pub fn table(d: &mut Draft, x: [usize; 2], y: usize, z: usize) {
    d.fill([x[0], y, z], [x[1], y, z], &slab("trim/slab", "top"));
    d.set([x[0], y, z], &stair("trim/stair", "west", "top"));
    d.set([x[1], y, z], &stair("trim/stair", "east", "top"));
    d.fill(
        [x[0], y, z - 1],
        [x[1], y, z - 1],
        &stair("trim/stair", "north", "bottom"),
    );
    d.fill(
        [x[0], y, z + 1],
        [x[1], y, z + 1],
        &stair("trim/stair", "south", "bottom"),
    );
}

/// A bed with its foot at `foot`, facing `facing`: the head lies the other way.
pub fn bed(d: &mut Draft, foot: [usize; 3], facing: &str) {
    d.set(foot, &format!("base:bed[facing={facing},part=foot]"));
    if let Some(head) = step(foot, opposite(facing)) {
        d.set(head, &format!("base:bed[facing={facing},part=head]"));
    }
}

fn corners(from: [usize; 2], to: [usize; 2]) -> [[usize; 2]; 4] {
    [from, [to[0], from[1]], [from[0], to[1]], to]
}
