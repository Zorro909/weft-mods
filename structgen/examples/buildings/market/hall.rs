//! The market hall: a timber-framed hall raised on log posts over an open, paved market floor.

use crate::draft::{AIR, Draft};
use crate::kit::{Ridge, gable_end, gable_roof, lights, log, slot, stair, timber_wall, window};
use crate::structure_file::Structure;

/// The hall's east wall; the west wall is at x = 1.
const EAST: usize = 17;

/// 17 x 9, a hall of close timber framing jutting out over a row of log posts on each long
/// side, its open ground floor paved in stone with stalls under it, a stone stair up the east
/// end to the hall's door, a long roof running east to west.
pub fn market_hall() -> Structure {
    let mut d = Draft::new([EAST + 2, 14, 11]);
    arcade(&mut d);
    upper_hall(&mut d);
    gable_roof(&mut d, Ridge::X, [0, EAST + 1], [0, 10], 8);
    for x in [1, EAST] {
        gable_end(&mut d, Ridge::X, x, [0, 10], [1, 9], 8, &slot("gable"));
    }
    outside_stair(&mut d);
    market_floor(&mut d);
    furnish_hall(&mut d);
    d.done()
}

/// The paving, under the eaves too, and the posts that carry the hall on stone feet: set one
/// block in on the long sides, with brackets under the jutting floor.
fn arcade(d: &mut Draft) {
    d.fill([0, 0, 0], [EAST + 1, 0, 10], &slot("ground"));
    d.fill([1, 0, 1], [EAST, 0, 9], &slot("plinth"));
    for x in (1..=EAST).step_by(4) {
        post(d, [x, 2]);
        post(d, [x, 8]);
        d.set([x, 3, 1], &stair("trim/stair", "south", "top"));
        d.set([x, 3, 9], &stair("trim/stair", "north", "top"));
    }
    for x in [1, EAST] {
        post(d, [x, 5]);
    }
}

fn post(d: &mut Draft, [x, z]: [usize; 2]) {
    d.set([x, 0, z], &slot("plinth/corner"));
    d.fill([x, 1, z], [x, 3, z], &log("y"));
}

/// The hall's floor on lying logs, its framed walls and windows, and the door at the east end.
fn upper_hall(d: &mut Draft) {
    d.fill([1, 4, 1], [EAST, 4, 9], &slot("floor"));
    for z in [1, 9] {
        d.fill([1, 4, z], [EAST, 4, z], &log("x"));
    }
    for x in [1, EAST] {
        d.fill([x, 4, 2], [x, 4, 8], &log("z"));
    }
    for (from, to) in [
        ([1, 1], [EAST, 1]),
        ([1, 9], [EAST, 9]),
        ([1, 1], [1, 9]),
        ([EAST, 1], [EAST, 9]),
    ] {
        timber_wall(d, from, to, [5, 7], 2);
    }
    for x in [4, 8, 10, 14] {
        window(d, [x, 5, 1], 2, "north");
        window(d, [x, 5, 9], 2, "south");
    }
    for z in [4, 6] {
        window(d, [1, 5, z], 2, "west");
    }
    d.fill([EAST, 5, 4], [EAST, 6, 4], AIR);
}

/// Stone steps up the east end, climbing north to a landing at the hall's door.
fn outside_stair(d: &mut Draft) {
    let x = EAST + 1;
    for (step, z) in [(1, 8), (2, 7), (3, 6), (4, 5)] {
        d.fill([x, 1, z], [x, step - 1, z], &slot("plinth"));
        d.set([x, step, z], &stair("plinth/stair", "north", "bottom"));
    }
    d.fill([x, 1, 4], [x, 4, 4], &slot("plinth"));
    d.set([x, 6, 3], "@light[facing=east]");
}

/// Two stalls under the hall, and lights on the posts.
fn market_floor(d: &mut Draft) {
    d.fill([4, 1, 4], [6, 1, 4], &stair("trim/stair", "north", "top"));
    d.set([5, 1, 3], "base:chest[facing=south]");
    d.fill([12, 1, 6], [14, 1, 6], &stair("trim/stair", "south", "top"));
    d.set([12, 1, 7], "base:chest[facing=north]");
    d.set([13, 1, 7], "base:crafting_table");
    lights(
        d,
        &[
            ([2, 3, 5], "east"),
            ([EAST - 1, 3, 5], "west"),
            ([9, 3, 3], "south"),
            ([9, 3, 7], "north"),
        ],
    );
}

/// A long table with benches down the hall, chests and a workbench, and lights on the walls.
fn furnish_hall(d: &mut Draft) {
    d.fill([5, 5, 5], [13, 5, 5], "@trim/slab[type=top]");
    d.fill(
        [5, 5, 4],
        [13, 5, 4],
        &stair("trim/stair", "north", "bottom"),
    );
    d.fill(
        [5, 5, 6],
        [13, 5, 6],
        &stair("trim/stair", "south", "bottom"),
    );
    d.set([2, 5, 2], "base:crafting_table");
    d.set([EAST - 1, 5, 8], "base:chest[facing=west]");
    d.set([EAST - 1, 5, 2], "base:chest[facing=west]");
    lights(
        d,
        &[
            ([2, 6, 5], "east"),
            ([9, 6, 2], "south"),
            ([9, 6, 8], "north"),
            ([EAST - 1, 6, 6], "west"),
        ],
    );
}
