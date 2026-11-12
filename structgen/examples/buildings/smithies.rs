//! Smithies: the village workshop, with furnaces and a tall chimney.

use crate::draft::{AIR, Draft};
use crate::inns::{floor_line, open_side, stone_walls, table, timber_storey};
use crate::kit::{
    Building, Ridge, bed, chimney, gable_end, gable_roof, ladder_up, light_at, lights, opposite,
    plinth, slab, slot, stair, step, window,
};
use crate::structure_file::Structure;

pub const BUILDINGS: &[Building] = &[("open_forge", open_forge), ("smithy_house", smithy_house)];

/// 7 x 7: a smith's shed on a cobbled floor, open to the north and west on log posts; its stone
/// back walls hold a lava hearth under a brick hood with a tall chimney, and the tools.
fn open_forge() -> Structure {
    let mut d = Draft::new([9, 12, 9]);
    plinth(&mut d, [1, 1], [7, 7], 1);
    d.fill([2, 0, 2], [6, 0, 6], &slot("plinth"));
    open_side(&mut d, [1, 1], [7, 1], [1, 3], 3);
    open_side(&mut d, [1, 1], [1, 7], [1, 3], 3);
    for (from, to) in [([7, 1], [7, 7]), ([1, 7], [7, 7])] {
        d.fill([from[0], 1, from[1]], [to[0], 3, to[1]], &slot("plinth"));
    }
    for [x, z] in [[7, 1], [7, 7], [1, 7]] {
        d.fill([x, 1, z], [x, 3, z], &slot("plinth/corner"));
    }
    gable_roof(&mut d, Ridge::X, [0, 8], [0, 8], 4);
    for x in [1, 7] {
        gable_end(&mut d, Ridge::X, x, [0, 8], [1, 7], 4, &slot("gable"));
    }
    forge_hearth(&mut d, [5, 1, 6], "south", 11);
    furnish_forge(&mut d);
    d.done()
}

/// An anvil, a quenching trough sunk in the floor, a furnace, a rack of tools over a chest and a
/// workbench, and torches.
fn furnish_forge(d: &mut Draft) {
    d.set([4, 1, 3], "base:iron_block");
    d.set([2, 0, 4], "base:water");
    d.set([6, 1, 2], "base:furnace[facing=west,lit=true]");
    d.fill([2, 2, 6], [3, 2, 6], &stair("trim/stair", "south", "top"));
    d.set([2, 1, 6], "base:chest[facing=north]");
    d.set([3, 1, 6], "base:crafting_table");
    light_at(d, [6, 2, 3], "west");
    light_at(d, [2, 2, 4], "east");
}

/// 10 x 13: a stone forge open to the street and a cobbled work yard beside it, and behind both
/// the smith's timber-framed house of two storeys, entered from the yard.
fn smithy_house() -> Structure {
    let mut d = Draft::new([12, 14, 15]);
    plinth(&mut d, [1, 7], [10, 13], 1);
    d.fill([0, 0, 0], [11, 0, 5], &slot("ground"));
    d.fill([1, 0, 1], [10, 0, 6], &slot("plinth"));
    forge_wing(&mut d);
    smith_house(&mut d);
    work_yard(&mut d);
    furnish_smith_house(&mut d);
    d.done()
}

/// A stone wing under a roof gabled to the street, its front an open arch, with the lava hearth
/// against its west wall, an anvil, a quenching trough and a tool rack.
fn forge_wing(d: &mut Draft) {
    stone_walls(d, [1, 1], [7, 7], [1, 3]);
    d.fill([2, 1, 1], [6, 2, 1], AIR);
    d.set([2, 2, 1], &stair("plinth/stair", "west", "top"));
    d.set([6, 2, 1], &stair("plinth/stair", "east", "top"));
    d.fill([7, 1, 4], [7, 2, 4], AIR);
    gable_roof(d, Ridge::Z, [0, 6], [0, 8], 4);
    gable_end(d, Ridge::Z, 1, [0, 8], [1, 7], 4, &slot("gable"));
    forge_hearth(d, [2, 1, 4], "west", 13);
    d.set([4, 1, 4], "base:iron_block");
    d.set([5, 0, 5], "base:water");
    d.set([6, 1, 6], "base:furnace[facing=west,lit=true]");
    d.fill([6, 2, 2], [6, 2, 3], &stair("trim/stair", "east", "top"));
    d.set([6, 1, 2], "base:chest[facing=west]");
    light_at(d, [6, 2, 5], "west");
}

/// Two timber-framed storeys under a roof gabled east and west, a chimney in the east gable.
fn smith_house(d: &mut Draft) {
    timber_storey(d, [1, 7], [10, 13], [1, 3], 3);
    floor_line(d, [1, 7], [10, 13], 4);
    timber_storey(d, [1, 7], [10, 13], [5, 7], 3);
    gable_roof(d, Ridge::X, [0, 11], [6, 14], 8);
    for x in [1, 10] {
        gable_end(d, Ridge::X, x, [6, 14], [7, 13], 8, &slot("gable"));
    }
    chimney(d, [10, 11], [1, 13]);
    for x in [5, 9] {
        d.fill([x, 1, 7], [x, 2, 7], AIR);
    }
    for (pos, outward) in [
        ([8, 2, 7], "north"),
        ([9, 6, 7], "north"),
        ([3, 2, 13], "south"),
        ([8, 2, 13], "south"),
        ([2, 6, 13], "south"),
        ([5, 6, 13], "south"),
        ([8, 6, 13], "south"),
        ([1, 2, 9], "west"),
        ([1, 2, 12], "west"),
        ([1, 6, 9], "west"),
        ([1, 6, 12], "west"),
        ([10, 2, 9], "east"),
        ([10, 6, 9], "east"),
    ] {
        window(d, pos, 1, outward);
    }
}

/// A woodpile and a heap of coal along the yard's east side, and torches by the house door and
/// on the forge.
fn work_yard(d: &mut Draft) {
    d.fill([10, 1, 1], [10, 2, 3], "base:oak_log[axis=z]");
    d.set([10, 2, 1], AIR);
    d.fill([10, 1, 5], [10, 1, 6], "base:coal_block");
    for x in [8, 10] {
        light_at(d, [x, 2, 6], "north");
    }
    light_at(d, [8, 2, 2], "east");
}

/// A kitchen below with a table, the cooking hearth and a ladder; a bedroom above.
fn furnish_smith_house(d: &mut Draft) {
    d.set([9, 1, 11], "base:furnace[facing=west,lit=true]");
    table(d, [4, 5], 1, 10);
    d.set([2, 1, 12], "base:chest[facing=north]");
    d.set([3, 1, 12], "base:crafting_table");
    ladder_up(d, [2, 8], [1, 5], "west");
    for x in [6, 8] {
        bed(d, [x, 5, 11], "north");
    }
    d.set([4, 5, 12], "base:chest[facing=north]");
    lights(
        d,
        &[
            ([3, 2, 8], "south"),
            ([7, 2, 12], "north"),
            ([5, 6, 8], "south"),
            ([7, 6, 12], "north"),
        ],
    );
}

/// Lava sunk in a 3 x 3 block of stone at `lava`, seen over a slab in front, the brick
/// `chimney` behind it on its `back` side rising to `top`, under a brick hood. The lava has
/// stone on every side, which it cannot flow into, and nothing wooden within a block.
fn forge_hearth(d: &mut Draft, lava: [usize; 3], back: &str, top: usize) {
    let [x, y, z] = lava;
    d.fill([x - 1, y, z - 1], [x + 1, y, z + 1], &slot("plinth/corner"));
    d.set(lava, "base:lava");
    if let Some(lip) = step(lava, opposite(back)) {
        d.set(lip, &slab("plinth/slab", "bottom"));
    }
    let flue = step(lava, back).expect("a hearth has room behind it for the chimney");
    chimney(d, [flue[0], flue[2]], [y, top]);
    let sides = match back {
        "north" | "south" => ["east", "west"],
        _ => ["north", "south"],
    };
    for side in sides {
        let [cx, _, cz] = step(lava, side).expect("a hearth's sides are within the box");
        d.fill([cx, y + 1, cz], [cx, y + 2, cz], &slot("chimney"));
    }
    d.set([x, y + 2, z], &slot("chimney"));
}
