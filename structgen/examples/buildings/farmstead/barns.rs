//! Barns and granaries: where the harvest is threshed and kept.

use structgen_api::Structure;

use crate::draft::{AIR, Draft};
use crate::kit::{
    Ridge, gable_end, gable_roof, ladder, light, log, plinth, slab, slot, stair, timber_wall,
    window,
};

/// 9 x 15 inside its walls: buttressed stone lower walls under oak-boarded upper walls, a tall
/// roof whose ridge runs front to back, a cart doorway at each end and a loft at the back.
pub fn great_barn() -> Structure {
    let mut d = Draft::new([11, 12, 17]);
    d.default("wall/infill", "base:oak_planks");
    plinth(&mut d, [1, 1], [9, 15], 1);
    barn_walls(&mut d);
    gable_roof(&mut d, Ridge::Z, [0, 16], [0, 10], 6);
    for z in [1, 15] {
        gable_end(&mut d, Ridge::Z, z, [0, 10], [1, 9], 6, &slot("gable"));
        d.fill([4, 1, z], [6, 4, z], AIR);
        d.set([5, 8, z], &slot("window"));
    }
    buttresses(&mut d);
    barn_loft(&mut d);
    furnish_barn(&mut d);
    d.done()
}

/// Two courses of stone, then timber up to the eaves, with windows high in the long sides.
fn barn_walls(d: &mut Draft) {
    for y in 1..=2 {
        d.ring(y, [1, 1], [9, 15], &slot("plinth"));
        for [x, z] in [[1, 1], [9, 1], [1, 15], [9, 15]] {
            d.set([x, y, z], &slot("plinth/corner"));
        }
    }
    for (from, to, spacing) in [
        ([1, 1], [9, 1], 4),
        ([1, 15], [9, 15], 4),
        ([1, 1], [1, 15], 7),
        ([9, 1], [9, 15], 7),
    ] {
        timber_wall(d, from, to, [3, 5], spacing);
    }
    for z in [5, 11] {
        window(d, [1, 3, z], 2, "west");
        window(d, [9, 3, z], 2, "east");
    }
}

/// Stone buttresses against the long sides, stepping back at the top.
fn buttresses(d: &mut Draft) {
    for z in [3, 8, 13] {
        for (x, toward) in [(0, "east"), (10, "west")] {
            d.set([x, 1, z], &slot("plinth/corner"));
            d.set([x, 2, z], &stair("plinth/stair", toward, "bottom"));
        }
    }
}

/// A loft over the back of the barn at the height of the eaves, railed at its edge and reached
/// by a ladder against the east wall.
fn barn_loft(d: &mut Draft) {
    d.fill([2, 5, 11], [8, 5, 14], &slot("floor"));
    d.fill([2, 6, 11], [7, 6, 11], &slab("trim/slab", "bottom"));
    d.fill([8, 1, 12], [8, 5, 12], &ladder("east"));
    for x in [3, 7] {
        d.set([x, 6, 14], "base:chest[facing=north]");
    }
    d.set([5, 7, 14], &light("north"));
}

/// Chests and a workbench along the walls, and torches on the posts and by both doorways.
fn furnish_barn(d: &mut Draft) {
    d.set([2, 1, 2], "base:crafting_table");
    d.set([2, 1, 14], "base:chest[facing=east]");
    d.set([8, 1, 2], "base:woodcutter");
    for (x, facing) in [(2, "east"), (8, "west")] {
        d.set([x, 3, 8], &light(facing));
    }
    for x in [3, 7] {
        d.set([x, 3, 0], &light("north"));
        d.set([x, 3, 16], &light("south"));
    }
}

/// 5 x 7: a timber store raised on staddle stones out of the reach of rats, stone steps up to
/// its door in the front gable.
pub fn staddle_granary() -> Structure {
    let mut d = Draft::new([7, 11, 11]);
    d.default("wall/infill", "base:oak_planks");
    d.fill([0, 0, 0], [6, 0, 10], &slot("ground"));
    staddles(&mut d);
    granary_body(&mut d);
    gable_roof(&mut d, Ridge::Z, [2, 10], [0, 6], 7);
    for z in [3, 9] {
        gable_end(&mut d, Ridge::Z, z, [0, 6], [1, 5], 7, &slot("gable"));
    }
    granary_steps(&mut d);
    d.done()
}

/// Stone columns, each under a cap of slabs, carrying the floor two blocks off the ground.
fn staddles(d: &mut Draft) {
    for x in [1, 5] {
        for z in [3, 6, 9] {
            d.fill([x, 0, z], [x, 2, z], &slot("plinth"));
            for [cx, cz] in [[x - 1, z], [x + 1, z], [x, z - 1], [x, z + 1]] {
                d.set([cx, 2, cz], &slab("plinth/slab", "top"));
            }
        }
    }
}

/// The floor on its sill beams, timber walls with the door in the front, and what is kept
/// inside.
fn granary_body(d: &mut Draft) {
    d.fill([1, 3, 3], [5, 3, 9], &slot("floor"));
    for z in [3, 9] {
        d.fill([1, 3, z], [5, 3, z], &log("x"));
    }
    for x in [1, 5] {
        d.fill([x, 3, 3], [x, 3, 9], &log("z"));
    }
    for (from, to) in [
        ([1, 3], [5, 3]),
        ([1, 9], [5, 9]),
        ([1, 3], [1, 9]),
        ([5, 3], [5, 9]),
    ] {
        timber_wall(d, from, to, [4, 6], 2);
    }
    d.fill([3, 4, 3], [3, 5, 3], AIR);
    window(d, [1, 5, 6], 1, "west");
    window(d, [5, 5, 6], 1, "east");
    for x in [2, 4] {
        d.set([x, 4, 8], "base:chest[facing=north]");
    }
    d.set([3, 6, 8], &light("north"));
}

/// Three stone steps up to the door, and a torch beside it.
fn granary_steps(d: &mut Draft) {
    for (y, z) in [(1, 0), (2, 1), (3, 2)] {
        d.fill([3, 1, z], [3, y - 1, z], &slot("plinth"));
        d.set([3, y, z], &stair("plinth/stair", "south", "bottom"));
    }
    d.set([2, 5, 2], &light("north"));
}
