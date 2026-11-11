//! The town hall: the landmark of a town, its belfry and spire seen from every street.

use super::{civic_roof, cross_gable, upper_storey};
use crate::chapels::{spire, stone_walls};
use crate::draft::{AIR, Draft};
use crate::kit::{
    Ridge, chimney, gable_end, gable_roof, ladder, log, plinth, slab, slot, stair, timber_wall,
    window,
};
use crate::structure_file::Structure;

/// 19 x 15, a timber storey jutting over a stone hall: a double stair climbs the front to the council
/// chamber's door over the arched entrance below, and a belfry tower rises from the middle
/// of the red tiled roof to a spire 30 tall.
pub fn town_hall() -> Structure {
    let mut d = Draft::new([21, 30, 18]);
    civic_roof(&mut d);
    d.fill([0, 0, 0], [20, 0, 17], &slot("ground"));
    plinth(&mut d, [1, 3], [19, 16], 0);
    stone_walls(&mut d, [1, 3], [19, 16], [1, 4]);
    upper_storey(&mut d, [1, 2], [19, 16], 5);
    for x in (1..=19).step_by(2) {
        d.set([x, 4, 2], &stair("trim/stair", "south", "top"));
    }
    gable_roof(&mut d, Ridge::X, [0, 20], [1, 17], 9);
    for x in [1, 19] {
        gable_end(&mut d, Ridge::X, x, [1, 17], [2, 16], 9, &slot("gable"));
        for z in [7, 11] {
            d.fill([x, 10, z], [x, 11, z], &slot("window"));
        }
        d.fill([x, 12, 9], [x, 14, 9], &slot("window"));
    }
    for x in [2, 14] {
        cross_gable(&mut d, [x, 1], 9);
    }
    chimney(&mut d, [4, 16], [1, 19]);
    front_stair(&mut d);
    town_hall_windows(&mut d);
    belfry_tower(&mut d);
    furnish_town_hall(&mut d);
    d.done()
}

/// Two flights up the front meeting on a landing before the council chamber's door, the
/// arched way into the hall below it.
fn front_stair(d: &mut Draft) {
    for i in 0..4 {
        let (west, east) = (5 + i, 15 - i);
        for x in [west, east] {
            d.fill([x, 1, 1], [x, i + 1, 1], &slot("plinth"));
        }
        d.set([west, 1 + i, 1], &stair("plinth/stair", "east", "bottom"));
        d.set([east, 1 + i, 1], &stair("plinth/stair", "west", "bottom"));
    }
    d.fill([9, 1, 1], [11, 5, 1], &slot("plinth"));
    d.set([9, 5, 1], &stair("plinth/stair", "east", "bottom"));
    d.set([11, 5, 1], &stair("plinth/stair", "west", "bottom"));
    for x in [9, 11] {
        d.fill([x, 1, 1], [x, 4, 1], &slot("plinth/corner"));
        d.set([x, 6, 1], "@light[facing=north]");
    }
    d.fill([10, 1, 1], [10, 3, 3], AIR);
    d.fill([10, 6, 2], [10, 7, 2], AIR);
}

fn town_hall_windows(d: &mut Draft) {
    for x in [3, 17] {
        window(d, [x, 2, 3], 2, "north");
    }
    for x in [4, 6, 14, 16] {
        window(d, [x, 6, 2], 2, "north");
    }
    for x in [6, 10, 14] {
        window(d, [x, 2, 16], 2, "south");
        window(d, [x, 6, 16], 2, "south");
    }
    for z in [5, 9, 13] {
        window(d, [1, 2, z], 2, "west");
        window(d, [19, 2, z], 2, "east");
        window(d, [1, 6, z], 2, "west");
        window(d, [19, 6, z], 2, "east");
    }
}

/// A stone tower up through the middle of the roof, a ladder inside to the timber belfry on
/// top, its bell, and the spire over it.
fn belfry_tower(d: &mut Draft) {
    let top = 20;
    stone_walls(d, [8, 7], [12, 11], [1, top]);
    d.fill([9, 1, 8], [11, top - 1, 10], AIR);
    for y in [5, top] {
        d.fill([9, y, 8], [11, y, 10], &slot("floor"));
    }
    for y in [1, 6] {
        d.fill([10, y, 7], [10, y + 1, 7], AIR);
    }
    d.fill([10, 1, 10], [10, top, 10], &ladder("south"));
    for y in [3, 8, 13, 18] {
        d.set([9, y, 9], "@light[facing=east]");
    }
    for (from, to) in [
        ([8, 7], [12, 7]),
        ([8, 11], [12, 11]),
        ([8, 7], [8, 11]),
        ([12, 7], [12, 11]),
    ] {
        timber_wall(d, from, to, [top + 1, top + 3], 2);
    }
    for c in [9, 11] {
        for [x, z] in [[c, 7], [c, 11], [8, c - 1], [12, c - 1]] {
            d.fill([x, top + 1, z], [x, top + 2, z], AIR);
        }
    }
    d.fill([9, top + 3, 9], [11, top + 3, 9], &log("x"));
    d.set([10, top + 2, 9], "base:gold_block");
    spire(d, [7, 6], 7, top + 4, 2);
}

/// The court below, with its clerks' tables, bench and chests, and the council's long table
/// by the hearth upstairs.
fn furnish_town_hall(d: &mut Draft) {
    d.fill(
        [4, 1, 13],
        [16, 1, 13],
        &stair("trim/stair", "south", "bottom"),
    );
    for x in [6, 14] {
        d.fill([x, 1, 9], [x, 1, 11], &slab("trim/slab", "top"));
    }
    for z in [8, 15] {
        d.set([2, 1, z], "base:chest[facing=east]");
        d.set([18, 1, z], "base:chest[facing=west]");
    }
    d.fill([5, 6, 13], [15, 6, 13], &slab("trim/slab", "top"));
    for z in [12, 14] {
        d.fill([5, 6, z], [15, 6, z], &slab("trim/slab", "bottom"));
    }
    d.set([4, 6, 15], "base:furnace[facing=north,lit=true]");
    d.set([4, 1, 15], "base:furnace[facing=north,lit=false]");
    for x in [3, 17] {
        d.set([x, 6, 4], "base:crafting_table");
    }
    town_hall_lights(d);
}

/// Torches round the walls and on the tower of both floors, clear of the windows.
fn town_hall_lights(d: &mut Draft) {
    for y in [3, 7] {
        for (x, z, facing) in [
            (4, 15, "north"),
            (16, 15, "north"),
            (7, 9, "west"),
            (13, 9, "east"),
            (10, 12, "south"),
        ] {
            d.set([x, y, z], &format!("@light[facing={facing}]"));
        }
        for z in [7, 11] {
            d.set([2, y, z], "@light[facing=east]");
            d.set([18, y, z], "@light[facing=west]");
        }
    }
    for x in [4, 16] {
        d.set([x, 3, 4], "@light[facing=south]");
    }
    for x in [8, 12] {
        d.set([x, 7, 3], "@light[facing=south]");
    }
}
