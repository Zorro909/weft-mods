//! Workyards: the trades that give a town its texture.

use structgen_api::Structure;

use crate::draft::{AIR, Draft};
use crate::kit::{
    Building, Ridge, gable_end, gable_roof, lights, log, plinth, side_windows, slot, stair,
    timber_wall, window,
};

/// A building's file name and how it is drawn.
pub const BUILDINGS: &[Building] = &[("lumberyard", lumberyard), ("storehouse", storehouse)];

const LADDER_ON_SOUTH: &str = "base:ladder[north=false,east=false,south=true,west=false]";

/// An open log shed with a plank back wall under a gabled roof, its logs stacked inside, and in
/// front a gravel yard with a saw pit, a woodpile and chopping blocks.
fn lumberyard() -> Structure {
    let mut d = Draft::new([11, 8, 11]);
    d.default("wall/infill", "base:oak_planks");
    d.fill([0, 0, 0], [10, 0, 10], &slot("ground"));
    log_shed(&mut d);
    gable_roof(&mut d, Ridge::X, [0, 10], [4, 10], 4);
    for x in [1, 9] {
        gable_end(&mut d, Ridge::X, x, [4, 10], [5, 9], 4, &slot("gable"));
    }
    shed_stock(&mut d);
    yard(&mut d);
    d.done()
}

/// Posts on stone feet with beams along the top, open at the front and sides, boarded behind.
fn log_shed(d: &mut Draft) {
    timber_wall(d, [1, 9], [9, 9], [1, 3], 4);
    for x in [1, 5, 9] {
        d.set([x, 0, 5], &slot("plinth/corner"));
        d.set([x, 0, 9], &slot("plinth/corner"));
        d.fill([x, 1, 5], [x, 3, 5], &log("y"));
    }
    d.fill([2, 3, 5], [8, 3, 5], &log("x"));
    for x in [1, 9] {
        d.fill([x, 3, 6], [x, 3, 8], &log("z"));
    }
}

/// Two stacks of seasoning logs, the woodcutter between them, and lights on the posts.
fn shed_stock(d: &mut Draft) {
    d.fill([2, 1, 6], [4, 2, 8], "base:oak_log[axis=x]");
    d.fill([6, 1, 6], [8, 1, 8], "base:spruce_log[axis=x]");
    d.fill([6, 2, 7], [8, 2, 8], "base:spruce_log[axis=x]");
    d.set([5, 1, 7], "base:woodcutter");
    d.set([5, 1, 8], "base:chest[facing=north]");
    for x in [1, 5, 9] {
        d.set([x, 2, 6], "@light[facing=south]");
    }
    d.set([5, 2, 4], "@light[facing=north]");
}

/// The saw pit, a trunk lying across its rails over the sawdust, a woodpile, and chopping
/// blocks.
fn yard(d: &mut Draft) {
    d.fill([2, 0, 2], [4, 0, 2], "base:dirt");
    for z in [1, 3] {
        d.fill([2, 1, z], [4, 1, z], &log("x"));
    }
    d.fill([3, 2, 1], [3, 2, 3], "base:spruce_log[axis=z]");
    d.fill([7, 1, 1], [9, 1, 3], "base:oak_log[axis=z]");
    d.fill([8, 2, 1], [8, 2, 3], "base:oak_log[axis=z]");
    d.set([5, 1, 2], "base:oak_log[axis=y]");
    d.set([6, 1, 3], "base:oak_log[axis=y]");
}

/// 9 x 15, two storeys: a stone ground floor with a wide cart doorway in the north gable, a
/// timber-framed loft jutting out over it with a loft door under a hoist beam out of the gable.
fn storehouse() -> Structure {
    let mut d = Draft::new([11, 14, 18]);
    d.fill([0, 0, 0], [10, 0, 17], &slot("ground"));
    plinth(&mut d, [1, 3], [9, 16], 1);
    stone_ground_floor(&mut d);
    loft_storey(&mut d);
    gable_roof(&mut d, Ridge::Z, [1, 17], [0, 10], 8);
    for z in [2, 16] {
        gable_end(&mut d, Ridge::Z, z, [0, 10], [1, 9], 8, &slot("gable"));
    }
    hoist(&mut d);
    stock_ground_floor(&mut d);
    stock_loft(&mut d);
    d.done()
}

/// Stone walls with stone-brick corners, one block in from the front under the jutting loft,
/// with brackets under it, the cart doorway, and small windows.
fn stone_ground_floor(d: &mut Draft) {
    for y in 1..=3 {
        d.ring(y, [1, 3], [9, 16], &slot("plinth"));
        for [x, z] in [[1, 3], [9, 3], [1, 16], [9, 16]] {
            d.set([x, y, z], &slot("plinth/corner"));
        }
    }
    for x in [1, 3, 7, 9] {
        d.set([x, 3, 2], &stair("trim/stair", "south", "top"));
    }
    d.fill([4, 1, 3], [6, 3, 3], AIR);
    side_windows(d, [1, 9], 2, 1, &[6, 12]);
    window(d, [5, 2, 16], 1, "south");
    d.set([3, 2, 2], "@light[facing=north]");
    d.set([7, 2, 2], "@light[facing=north]");
}

/// The loft's floor on lying logs, its framed walls and windows, and the loft door.
fn loft_storey(d: &mut Draft) {
    d.fill([1, 4, 2], [9, 4, 16], &slot("floor"));
    d.ring(4, [1, 2], [9, 16], &log("z"));
    for z in [2, 16] {
        d.fill([1, 4, z], [9, 4, z], &log("x"));
    }
    for (from, to) in [
        ([1, 2], [9, 2]),
        ([1, 16], [9, 16]),
        ([1, 2], [1, 16]),
        ([9, 2], [9, 16]),
    ] {
        timber_wall(d, from, to, [5, 7], 2);
    }
    side_windows(d, [1, 9], 5, 2, &[5, 9, 13]);
    for x in [2, 8] {
        window(d, [x, 5, 2], 2, "north");
    }
    for x in [4, 6] {
        window(d, [x, 5, 16], 2, "south");
    }
    d.fill([5, 5, 2], [5, 6, 2], AIR);
    d.set([5, 7, 2], &log("x"));
}

/// A beam out of the gable over the loft door, on a bracket, to hoist goods up by.
fn hoist(d: &mut Draft) {
    d.fill([5, 9, 0], [5, 9, 2], &log("z"));
    d.set([5, 8, 1], &stair("trim/stair", "south", "top"));
}

/// Chests along the walls, crates stacked at the back, a ladder to the loft, and lights.
fn stock_ground_floor(d: &mut Draft) {
    for z in [5, 6, 8, 9] {
        d.set([2, 1, z], "base:chest[facing=east]");
        d.set([8, 1, z], "base:chest[facing=west]");
    }
    d.fill([2, 1, 12], [3, 1, 14], "base:oak_planks");
    d.fill([2, 2, 13], [2, 2, 14], "base:oak_planks");
    d.set([8, 1, 11], "base:crafting_table");
    d.fill([5, 1, 15], [5, 5, 15], LADDER_ON_SOUTH);
    lights(
        d,
        &[
            ([2, 2, 4], "east"),
            ([8, 2, 4], "west"),
            ([2, 2, 10], "east"),
            ([8, 2, 13], "west"),
        ],
    );
}

/// Sacks of wool, chests and crates in the loft, and lights.
fn stock_loft(d: &mut Draft) {
    d.fill([7, 5, 6], [8, 5, 7], "base:wool");
    d.set([8, 6, 6], "base:wool");
    d.set([2, 5, 13], "base:chest[facing=east]");
    d.set([2, 5, 14], "base:chest[facing=east]");
    d.fill([8, 5, 12], [8, 5, 14], "base:spruce_planks");
    lights(
        d,
        &[
            ([2, 6, 11], "east"),
            ([8, 6, 11], "west"),
            ([3, 6, 3], "south"),
            ([7, 6, 15], "north"),
        ],
    );
}
