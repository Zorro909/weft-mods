//! Mills: landmarks on the edge of a village, seen from far away.

use structgen_api::Structure;

use crate::draft::{AIR, Draft};
use crate::kit::{
    Building, Ridge, gable_end, gable_roof, ladder, light, log, slab, slot, timber_wall, window,
};

pub const BUILDINGS: &[Building] = &[("post_mill", post_mill), ("tower_mill", tower_mill)];

const SAILCLOTH: &str = "base:wool";
const MILLSTONE: &str = "base:stone_slab[type=bottom]";

/// A timber body raised on its post over a round stone roundhouse, four great sails on its
/// front. The door is in the roundhouse, under the sails; a ladder up the post leads into the
/// body.
fn post_mill() -> Structure {
    let mut d = Draft::new([15, 20, 9]);
    d.default("wall/infill", "base:oak_planks");
    d.fill([0, 0, 0], [14, 0, 8], &slot("ground"));
    roundhouse(&mut d);
    mill_body(&mut d);
    gable_roof(&mut d, Ridge::Z, [1, 8], [4, 10], 12);
    for z in [2, 7] {
        gable_end(&mut d, Ridge::Z, z, [4, 10], [5, 9], 12, &slot("gable"));
    }
    sails(&mut d, [7, 12, 0], 7);
    d.fill([7, 12, 1], [7, 12, 2], &log("z"));
    d.done()
}

/// The round stone house under the body: the post in its middle with a ladder up it, a roof
/// closed under the body with slab eaves, its door to the front.
fn roundhouse(d: &mut Draft) {
    let centre = [7, 4];
    round_course(d, centre, 0, 12, &slot("plinth/corner"), &slot("floor"));
    for y in 1..=3 {
        round_course(d, centre, y, 12, &slot("plinth"), AIR);
    }
    round_course(
        d,
        centre,
        4,
        12,
        &slab("roof/slab", "bottom"),
        &slot("roof/block"),
    );
    d.fill([7, 1, 1], [7, 2, 1], AIR);
    d.fill([7, 1, 4], [7, 4, 4], &log("y"));
    d.fill([7, 1, 5], [7, 5, 5], &ladder("north"));
    d.set([8, 1, 6], "base:chest[facing=north]");
    d.set([5, 2, 4], &light("east"));
    d.set([6, 2, 0], &light("north"));
}

/// The boarded body: its floor on the post, timber walls with windows, a millstone and a chest
/// inside.
fn mill_body(d: &mut Draft) {
    d.fill([5, 5, 2], [9, 5, 7], &slot("floor"));
    d.set([7, 5, 5], &ladder("north"));
    for (from, to) in [
        ([5, 2], [9, 2]),
        ([5, 7], [9, 7]),
        ([5, 2], [5, 7]),
        ([9, 2], [9, 7]),
    ] {
        timber_wall(d, from, to, [6, 11], 4);
    }
    window(d, [5, 8, 4], 2, "west");
    window(d, [9, 8, 4], 2, "east");
    window(d, [7, 8, 7], 1, "south");
    for x in [6, 7] {
        d.set([x, 6, 3], MILLSTONE);
    }
    d.set([6, 6, 6], "base:chest[facing=east]");
    d.set([8, 8, 3], &light("south"));
}

/// A round stone tower narrowing at a ledge two thirds of the way up, under a timber cap that
/// carries four sails on its front. The door faces front; a ladder against the back wall climbs
/// two floors.
fn tower_mill() -> Structure {
    let mut d = Draft::new([13, 22, 9]);
    d.fill([0, 0, 0], [12, 0, 8], &slot("ground"));
    tower(&mut d);
    tower_cap(&mut d);
    sails(&mut d, [6, 15, 0], 6);
    d.fill([6, 15, 1], [6, 15, 2], &log("z"));
    furnish_tower(&mut d);
    d.done()
}

/// The stone courses, 7 across up to the ledge and 5 above it, floors at two heights, and the
/// door, windows and ladder.
fn tower(d: &mut Draft) {
    let centre = [6, 4];
    round_course(d, centre, 0, 12, &slot("plinth/corner"), &slot("floor"));
    for y in 1..=14 {
        let reach = if y <= 9 { 12 } else { 6 };
        let floor = if y == 5 || y == 9 {
            slot("floor")
        } else {
            AIR.to_string()
        };
        round_course(d, centre, y, reach, &slot("plinth"), &floor);
    }
    for ([x, z], _) in disk(centre, 12).filter(|(cell, _)| !in_disk(centre, *cell, 6)) {
        d.set([x, 10, z], &slab("plinth/slab", "bottom"));
    }
    d.fill([6, 1, 1], [6, 2, 1], AIR);
    // Off the wall on a stone pier, so its top comes out inside the narrower top storey.
    d.fill([6, 1, 6], [6, 9, 6], &slot("plinth"));
    d.fill([6, 1, 5], [6, 9, 5], &ladder("south"));
    for (y, x) in [(3, 3), (7, 9), (12, 8)] {
        window(d, [x, y, 4], 1, if x < 6 { "west" } else { "east" });
    }
}

/// A boarded cap with a ridge running front to back, sitting on the top of the tower.
fn tower_cap(d: &mut Draft) {
    d.ring(15, [4, 2], [8, 6], &slot("gable"));
    gable_roof(d, Ridge::Z, [1, 7], [3, 9], 15);
    for z in [2, 6] {
        gable_end(d, Ridge::Z, z, [3, 9], [4, 8], 15, &slot("gable"));
    }
}

/// Sacks and tools at the bottom, the millstone at the top, torches on every floor.
fn furnish_tower(d: &mut Draft) {
    d.set([4, 1, 5], "base:crafting_table");
    d.set([8, 1, 5], "base:chest[facing=west]");
    d.set([8, 6, 4], "base:chest[facing=west]");
    for x in [5, 6] {
        d.set([x, 10, 3], MILLSTONE);
    }
    d.set([4, 2, 4], &light("east"));
    d.set([4, 7, 4], &light("east"));
    d.set([5, 12, 4], &light("east"));
    d.set([5, 2, 0], &light("north"));
}

/// Four sails turning about `hub` in the front (north) face: a log stock `length` blocks out
/// each way, and sailcloth two blocks wide along one side of each arm, all four the same way
/// round.
fn sails(d: &mut Draft, [x, y, z]: [usize; 3], length: usize) {
    d.set([x, y, z], &log("z"));
    for i in 1..=length {
        d.set([x, y + i, z], &log("y"));
        d.set([x, y - i, z], &log("y"));
        d.set([x + i, y, z], &log("x"));
        d.set([x - i, y, z], &log("x"));
        if i < 2 {
            continue;
        }
        for w in 1..=2 {
            d.set([x + w, y + i, z], SAILCLOTH);
            d.set([x + i, y - w, z], SAILCLOTH);
            d.set([x - w, y - i, z], SAILCLOTH);
            d.set([x - i, y + w, z], SAILCLOTH);
        }
    }
}

/// Layer `y` of a round tower about `centre` (x, z): `wall` on the edge of the disk `reach`
/// wide (see `in_disk`), `inside` within it.
fn round_course(
    d: &mut Draft,
    centre: [usize; 2],
    y: usize,
    reach: usize,
    wall: &str,
    inside: &str,
) {
    for ([x, z], edge) in disk(centre, reach) {
        d.set([x, y, z], if edge { wall } else { inside });
    }
}

/// The cells of the disk `reach` wide about `centre`, each with whether it is on the edge.
fn disk(centre: [usize; 2], reach: usize) -> impl Iterator<Item = ([usize; 2], bool)> {
    let span = 4;
    let cells = (centre[0] - span..=centre[0] + span)
        .flat_map(move |x| (centre[1] - span..=centre[1] + span).map(move |z| [x, z]));
    cells
        .filter(move |&cell| in_disk(centre, cell, reach))
        .map(move |[x, z]| {
            let neighbours = [[x - 1, z], [x + 1, z], [x, z - 1], [x, z + 1]];
            let edge = neighbours.iter().any(|&n| !in_disk(centre, n, reach));
            ([x, z], edge)
        })
}

/// Whether `cell` is in the disk about `centre` whose squared radius is at most `reach`: 12 is
/// a round 7 blocks across, 9 a narrower 7, 6 a round 5.
fn in_disk(centre: [usize; 2], cell: [usize; 2], reach: usize) -> bool {
    let dx = cell[0].abs_diff(centre[0]);
    let dz = cell[1].abs_diff(centre[1]);
    dx * dx + dz * dz <= reach
}
