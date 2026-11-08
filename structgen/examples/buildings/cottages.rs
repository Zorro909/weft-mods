//! Crofter's cottages: the small dwellings most of a hamlet or village is made of.

use structgen_api::Structure;

use crate::draft::{AIR, Draft};
use crate::kit::{
    Building, Ridge, chimney, gable_end, gable_roof, ladder_up, log, plinth, slab, slot, stair,
    timber_wall, window,
};

pub const BUILDINGS: &[Building] = &[
    ("crofter_cottage", crofter_cottage),
    ("longhouse", longhouse),
    ("dormer_cottage", dormer_cottage),
];

/// 7 x 5 inside its walls, one storey: close-studded oak walls on a cobblestone plinth under a
/// spruce roof, the door in the middle of the long front, a brick chimney in the back wall over
/// the hearth.
fn crofter_cottage() -> Structure {
    let mut d = Draft::new([9, 9, 7]);
    d.default("wall/infill", "base:oak_planks");
    plinth(&mut d, [1, 1], [7, 5], 1);
    for (from, to) in [
        ([1, 1], [7, 1]),
        ([1, 5], [7, 5]),
        ([1, 1], [1, 5]),
        ([7, 1], [7, 5]),
    ] {
        timber_wall(&mut d, from, to, [1, 3], 2);
    }
    gable_roof(&mut d, Ridge::X, [0, 8], [0, 6], 4);
    for x in [1, 7] {
        gable_end(&mut d, Ridge::X, x, [0, 6], [1, 5], 4, &slot("gable"));
    }
    d.fill([4, 1, 1], [4, 2, 1], AIR);
    for x in [2, 6] {
        window(&mut d, [x, 2, 1], 1, "north");
        window(&mut d, [x, 2, 5], 1, "south");
    }
    window(&mut d, [1, 2, 2], 1, "west");
    window(&mut d, [7, 2, 4], 1, "east");
    chimney(&mut d, [6, 5], [1, 8]);
    furnish_cottage(&mut d);
    d.done()
}

/// A hearth, bed, workbench and chest, and a torch either side of the door.
fn furnish_cottage(d: &mut Draft) {
    d.set([6, 1, 4], "base:furnace[facing=north,lit=false]");
    d.set([2, 1, 4], "base:bed[facing=west,part=foot]");
    d.set([3, 1, 4], "base:bed[facing=west,part=head]");
    d.set([2, 1, 2], "base:crafting_table");
    d.set([5, 1, 4], "base:chest[facing=north]");
    d.set([3, 2, 0], "@light[facing=north]");
    d.set([5, 2, 0], "@light[facing=north]");
}

/// 12 x 7, one low storey under a deep roof whose eaves hang just over the doorway: oak-framed
/// rooms at the west end, a chimney in the west gable, and a cobblestone byre at the east end
/// with a wide open stall door in the front.
fn longhouse() -> Structure {
    let mut d = Draft::new([14, 10, 9]);
    d.default("wall/infill", "base:oak_planks");
    plinth(&mut d, [1, 1], [12, 7], 1);
    gable_roof(&mut d, Ridge::X, [0, 13], [0, 8], 3);
    for x in [1, 8, 12] {
        gable_end(&mut d, Ridge::X, x, [0, 8], [1, 7], 3, &slot("gable"));
    }
    for (from, to) in [
        ([1, 1], [8, 1]),
        ([1, 7], [8, 7]),
        ([1, 1], [1, 7]),
        ([8, 1], [8, 7]),
    ] {
        timber_wall(&mut d, from, to, [1, 3], 2);
    }
    byre(&mut d);
    d.fill([6, 1, 1], [6, 2, 1], AIR);
    door_hood(&mut d, 6);
    d.fill([8, 1, 4], [8, 2, 4], AIR);
    for x in [2, 4] {
        window(&mut d, [x, 2, 1], 1, "north");
    }
    for x in [2, 4, 6] {
        window(&mut d, [x, 2, 7], 1, "south");
    }
    for z in [2, 6] {
        window(&mut d, [1, 2, z], 1, "west");
    }
    chimney(&mut d, [1, 4], [1, 9]);
    furnish_longhouse(&mut d);
    d.done()
}

/// Rubble walls east of the house, open to the front under a lying log, with a trough of water
/// sunk in its gravel floor.
fn byre(d: &mut Draft) {
    for z in [1, 7] {
        d.fill([9, 1, z], [12, 3, z], &slot("plinth"));
    }
    d.fill([12, 1, 1], [12, 3, 7], &slot("plinth"));
    for z in [1, 7] {
        d.fill([12, 1, z], [12, 3, z], &slot("plinth/corner"));
    }
    d.fill([9, 1, 1], [11, 2, 1], AIR);
    d.fill([9, 3, 1], [11, 3, 1], &log("x"));
    d.fill([9, 0, 2], [11, 0, 6], &slot("path"));
    d.fill([9, 0, 6], [11, 0, 6], "base:water");
    d.fill([9, 1, 5], [11, 1, 5], &slab("plinth/slab", "bottom"));
    d.set([11, 2, 2], "@light[facing=west]");
}

/// A hearth under the chimney, bed, chest, table and stools, workbench, and torches.
fn furnish_longhouse(d: &mut Draft) {
    d.set([2, 1, 4], "base:furnace[facing=east,lit=false]");
    d.set([3, 1, 6], "base:bed[facing=east,part=foot]");
    d.set([2, 1, 6], "base:bed[facing=east,part=head]");
    d.set([2, 1, 2], "base:chest[facing=east]");
    d.set([7, 1, 6], "base:crafting_table");
    d.set([5, 1, 4], &slab("trim/slab", "top"));
    d.set([4, 1, 4], &stair("trim/stair", "west", "bottom"));
    d.set([6, 1, 4], &stair("trim/stair", "east", "bottom"));
    d.set([3, 2, 2], "@light[facing=south]");
    d.set([7, 2, 6], "@light[facing=north]");
}

/// A little gabled hood on two posts over the front door at `x`, where the eaves hang too low
/// to walk under, with a torch under it.
fn door_hood(d: &mut Draft, x: usize) {
    for post in [x - 1, x + 1] {
        d.fill([post, 1, 0], [post, 2, 0], &log("y"));
    }
    d.set([x - 1, 3, 0], &roof_corner("east"));
    d.set([x + 1, 3, 0], &roof_corner("west"));
    d.set([x, 4, 0], &slab("roof/slab", "bottom"));
    d.set([x, 3, 0], "@light[facing=north]");
}

/// 9 x 7, a storey and a half: an oak-framed cottage whose loft bedroom lights through a dormer
/// over the door, a chimney in the east gable and a ladder up from the parlour.
fn dormer_cottage() -> Structure {
    let mut d = Draft::new([11, 11, 9]);
    d.default("wall/infill", "base:oak_planks");
    plinth(&mut d, [1, 1], [9, 7], 1);
    gable_roof(&mut d, Ridge::X, [0, 10], [0, 8], 4);
    for x in [1, 9] {
        gable_end(&mut d, Ridge::X, x, [0, 8], [1, 7], 4, &slot("gable"));
    }
    for (from, to) in [
        ([1, 1], [9, 1]),
        ([1, 7], [9, 7]),
        ([1, 1], [1, 7]),
        ([9, 1], [9, 7]),
    ] {
        timber_wall(&mut d, from, to, [1, 4], 2);
    }
    d.fill([2, 4, 2], [8, 4, 6], &slot("floor"));
    dormer(&mut d, [5, 5, 1]);
    d.fill([5, 1, 1], [5, 2, 1], AIR);
    for x in [3, 7] {
        window(&mut d, [x, 2, 1], 1, "north");
    }
    for x in [3, 5, 7] {
        window(&mut d, [x, 2, 7], 1, "south");
    }
    for z in [2, 6] {
        window(&mut d, [1, 2, z], 1, "west");
        window(&mut d, [9, 2, z], 1, "east");
    }
    window(&mut d, [1, 6, 4], 1, "west");
    chimney(&mut d, [9, 4], [1, 10]);
    ladder_up(&mut d, [2, 4], [1, 4], "west");
    furnish_dormer_cottage(&mut d);
    d.done()
}

/// A parlour with hearth, table and workbench below; a bed and chest in the loft.
fn furnish_dormer_cottage(d: &mut Draft) {
    d.set([8, 1, 4], "base:furnace[facing=west,lit=false]");
    d.set([8, 1, 2], "base:crafting_table");
    d.set([8, 1, 6], "base:chest[facing=west]");
    d.set([5, 1, 5], &slab("trim/slab", "top"));
    d.set([4, 1, 5], &stair("trim/stair", "west", "bottom"));
    d.set([6, 1, 5], &stair("trim/stair", "east", "bottom"));
    d.set([3, 3, 2], "@light[facing=south]");
    d.set([7, 3, 6], "@light[facing=north]");
    d.set([4, 2, 0], "@light[facing=north]");
    d.set([6, 2, 0], "@light[facing=north]");
    d.set([6, 5, 5], "base:bed[facing=west,part=foot]");
    d.set([7, 5, 5], "base:bed[facing=west,part=head]");
    d.set([8, 5, 4], "base:chest[facing=west]");
    d.set([2, 6, 3], "@light[facing=east]");
    d.set([8, 6, 5], "@light[facing=west]");
}

/// A dormer in a roof slope that rises southward one block per block: a window at `pos` between
/// boarded cheeks, under a little gable roof of its own, with the main roof opened behind it.
pub fn dormer(d: &mut Draft, [x, y, z]: [usize; 3]) {
    d.set([x - 1, y, z], &slot("gable"));
    d.set([x + 1, y, z], &slot("gable"));
    d.set([x, y, z], &slot("window"));
    d.set([x, y + 1, z], &slot("gable"));
    d.set([x, y + 1, z + 1], AIR);
    // Each row of its roof reaches back until it meets the main roof in a corner.
    for k in 0..2 {
        for back in z - 1..z + k {
            d.set(
                [x - 2 + k, y + k, back],
                &stair("roof/stair", "east", "bottom"),
            );
            d.set(
                [x + 2 - k, y + k, back],
                &stair("roof/stair", "west", "bottom"),
            );
        }
        d.set([x - 2 + k, y + k, z + k], &roof_corner("east"));
        d.set([x + 2 - k, y + k, z + k], &roof_corner("west"));
    }
    d.fill(
        [x, y + 2, z - 1],
        [x, y + 2, z + 1],
        &slab("roof/slab", "bottom"),
    );
}

/// A `roof/stair` facing `east` or `west` where its slope meets a slope rising southward: an
/// inner corner, high on its facing side and to the south.
fn roof_corner(facing: &str) -> String {
    let shape = if facing == "east" {
        "inner_right"
    } else {
        "inner_left"
    };
    format!("@roof/stair[facing={facing},half=bottom,shape={shape}]")
}
