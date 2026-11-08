//! Crofter's cottages: the small dwellings most of a hamlet or village is made of.

use structgen_api::Structure;

use crate::draft::{AIR, Draft};
use crate::kit::{Ridge, chimney, gable_end, gable_roof, plinth, slot, timber_wall, window};

pub const BUILDINGS: &[(&str, fn() -> Structure)] = &[("crofter_cottage", crofter_cottage)];

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
