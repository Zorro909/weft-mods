//! Chapels and churches: small ones for villages, a large one for towns. Also the stone walls
//! and the stepped spires they share with the town hall.

mod town_church;

use structgen_api::Structure;

use crate::draft::{AIR, Draft};
use crate::kit::{
    Building, Ridge, gable_end, gable_roof, log, plinth, slab, slot, stair, timber_wall, window,
};

pub const BUILDINGS: &[Building] = &[
    ("wayside_chapel", wayside_chapel),
    ("village_church", village_church),
    ("town_church", town_church::town_church),
];

/// 5 x 7, one stone room under a steep roof: a little bell gable stands on the front gable over
/// the doorway, and a tall window lights the altar from the back.
fn wayside_chapel() -> Structure {
    let mut d = Draft::new([7, 11, 9]);
    plinth(&mut d, [1, 1], [5, 7], 1);
    stone_walls(&mut d, [1, 1], [5, 7], [1, 3]);
    gable_roof(&mut d, Ridge::Z, [0, 8], [0, 6], 4);
    for z in [1, 7] {
        gable_end(&mut d, Ridge::Z, z, [0, 6], [1, 5], 4, &slot("plinth"));
    }
    bell_gable(&mut d, [3, 1], 7);
    d.fill([3, 1, 1], [3, 2, 1], AIR);
    for z in [3, 5] {
        window(&mut d, [1, 2, z], 2, "west");
        window(&mut d, [5, 2, z], 2, "east");
    }
    window(&mut d, [3, 2, 7], 3, "south");
    altar(&mut d, [2, 4], 6);
    for z in [3, 4] {
        pew(&mut d, [2, 2], z);
        pew(&mut d, [4, 4], z);
    }
    for x in [2, 4] {
        d.set([x, 2, 0], "@light[facing=north]");
        d.set([x, 3, 2], "@light[facing=south]");
    }
    d.done()
}

/// A 9 x 11 stone nave and a narrower chancel behind it, under steep roofs; a timber belfry
/// with a tall pyramid spire rides the front of the nave's ridge.
fn village_church() -> Structure {
    let mut d = Draft::new([11, 18, 18]);
    d.fill([0, 0, 0], [10, 0, 17], &slot("ground"));
    plinth(&mut d, [1, 1], [9, 11], 0);
    plinth(&mut d, [3, 11], [7, 16], 0);
    stone_walls(&mut d, [1, 1], [9, 11], [1, 5]);
    stone_walls(&mut d, [3, 11], [7, 16], [1, 4]);
    gable_roof(&mut d, Ridge::Z, [0, 12], [0, 10], 6);
    gable_roof(&mut d, Ridge::Z, [12, 17], [2, 8], 5);
    for z in [1, 11] {
        gable_end(&mut d, Ridge::Z, z, [0, 10], [1, 9], 6, &slot("plinth"));
    }
    gable_end(&mut d, Ridge::Z, 16, [2, 8], [3, 7], 5, &slot("plinth"));
    d.fill([4, 1, 11], [6, 4, 11], AIR);
    d.fill([5, 1, 1], [5, 3, 1], AIR);
    d.fill([5, 7, 1], [5, 8, 1], &slot("window"));
    for z in [3, 6, 9] {
        window(&mut d, [1, 2, z], 3, "west");
        window(&mut d, [9, 2, z], 3, "east");
    }
    window(&mut d, [5, 2, 16], 5, "south");
    window(&mut d, [3, 2, 14], 2, "west");
    window(&mut d, [7, 2, 14], 2, "east");
    belfry(&mut d);
    furnish_village_church(&mut d);
    d.done()
}

/// A timber belfry over the front of the nave, standing on two tie beams, with louvres on every
/// side, a bell, and a spire.
fn belfry(d: &mut Draft) {
    for z in [2, 4] {
        d.fill([1, 6, z], [9, 6, z], &log("x"));
    }
    for (from, to) in [
        ([4, 2], [6, 2]),
        ([4, 4], [6, 4]),
        ([4, 2], [4, 4]),
        ([6, 2], [6, 4]),
    ] {
        timber_wall(d, from, to, [11, 14], 2);
    }
    for [x, z] in [[4, 2], [6, 2], [4, 4], [6, 4]] {
        d.fill([x, 7, z], [x, 10, z], &log("y"));
    }
    for [x, z] in [[5, 2], [5, 4], [4, 3], [6, 3]] {
        d.fill([x, 12, z], [x, 13, z], AIR);
    }
    d.set([5, 14, 3], &log("x"));
    d.set([5, 13, 3], "base:gold_block");
    spire(d, [3, 1], 5, 15, 1);
}

/// Pews either side of the aisle, an altar with candles, a rail before it and torches.
fn furnish_village_church(d: &mut Draft) {
    for z in [4, 6, 8] {
        pew(d, [2, 4], z);
        pew(d, [6, 8], z);
    }
    altar(d, [4, 6], 15);
    for x in [4, 6] {
        d.set([x, 1, 13], &slab("trim/slab", "bottom"));
        d.set([x, 3, 0], "@light[facing=north]");
    }
    for z in [4, 8] {
        d.set([2, 4, z], "@light[facing=east]");
        d.set([8, 4, z], "@light[facing=west]");
    }
}

/// Walls of `plinth` around the ring from `from` to `to` (x, z), heights `y[0]` to `y[1]`, with
/// `plinth/corner` quoins up every corner.
pub fn stone_walls(d: &mut Draft, from: [usize; 2], to: [usize; 2], y: [usize; 2]) {
    for h in y[0]..=y[1] {
        d.ring(h, from, to, &slot("plinth"));
    }
    for [x, z] in [from, [to[0], from[1]], [from[0], to[1]], to] {
        d.fill([x, y[0], z], [x, y[1], z], &slot("plinth/corner"));
    }
}

/// A bell gable of stone standing on a gable's apex at (`x`, `z`), from height `y`: two piers, a
/// bell between them and a little pitched top.
fn bell_gable(d: &mut Draft, [x, z]: [usize; 2], y: usize) {
    d.fill([x - 1, y, z], [x + 1, y, z], &slot("plinth"));
    d.set([x - 1, y + 1, z], &slot("plinth/corner"));
    d.set([x + 1, y + 1, z], &slot("plinth/corner"));
    d.set([x, y + 1, z], "base:gold_block");
    d.set([x - 1, y + 2, z], &stair("plinth/stair", "east", "bottom"));
    d.set([x, y + 2, z], &slot("plinth"));
    d.set([x + 1, y + 2, z], &stair("plinth/stair", "west", "bottom"));
    d.set([x, y + 3, z], &slab("plinth/slab", "bottom"));
}

/// An altar of stone from `x[0]` to `x[1]` at `z`, a candle at each end.
pub fn altar(d: &mut Draft, x: [usize; 2], z: usize) {
    d.fill([x[0], 1, z], [x[1], 1, z], "base:stone_bricks");
    for end in x {
        d.set([end, 2, z], "@light[facing=up]");
    }
}

/// A pew from `x[0]` to `x[1]` in row `z`, facing the altar in the south.
pub fn pew(d: &mut Draft, x: [usize; 2], z: usize) {
    d.fill(
        [x[0], 1, z],
        [x[1], 1, z],
        &stair("trim/stair", "north", "bottom"),
    );
}

/// A spire over the square `size` wide from `corner` (x, z), from height `y` up: an eave of
/// `roof/stair`, then rings each one block in and `rise` higher, `roof/block` under a ring of
/// stairs, ending in a cap of stairs round a `roof/block` and a `roof/slab`; `size` is odd.
/// Gives back the height of the slab.
pub fn spire(d: &mut Draft, corner: [usize; 2], size: usize, y: usize, rise: usize) -> usize {
    let mut top = y;
    for k in 0..size / 2 {
        let from = [corner[0] + k, corner[1] + k];
        let to = [corner[0] + size - 1 - k, corner[1] + size - 1 - k];
        if k > 0 {
            for h in top + 1..top + rise {
                d.ring(h, from, to, &slot("roof/block"));
            }
            top += rise;
        }
        stair_ring(d, top, from, to);
    }
    let [x, z] = [corner[0] + size / 2, corner[1] + size / 2];
    d.set([x, top, z], &slot("roof/block"));
    d.set([x, top + 1, z], &slab("roof/slab", "bottom"));
    top + 1
}

/// A ring of `roof/stair` rising inward, its corners outer corners.
fn stair_ring(d: &mut Draft, y: usize, from: [usize; 2], to: [usize; 2]) {
    let roof = |facing| stair("roof/stair", facing, "bottom");
    for x in from[0]..=to[0] {
        d.set([x, y, from[1]], &roof("south"));
        d.set([x, y, to[1]], &roof("north"));
    }
    for z in from[1]..=to[1] {
        d.set([from[0], y, z], &roof("east"));
        d.set([to[0], y, z], &roof("west"));
    }
    let corner = |facing, shape| format!("@roof/stair[facing={facing},half=bottom,shape={shape}]");
    d.set([from[0], y, from[1]], &corner("south", "outer_left"));
    d.set([to[0], y, from[1]], &corner("south", "outer_right"));
    d.set([from[0], y, to[1]], &corner("north", "outer_right"));
    d.set([to[0], y, to[1]], &corner("north", "outer_left"));
}
