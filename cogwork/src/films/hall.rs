//! The hall the engine and the benches stand in: an open pole barn, 25 blocks from west to
//! east and 15 across. Oak posts on stone footings carry a low roof of oak slabs with glass
//! let into it; both long sides are open, and so are both ends under their brick gables, so
//! the camera flies in at the west end and out at the east.

use showcase_api::Set;
use weft_sdk::Block;

use super::parts::{GLOWSTONE, OAK_LOG, STONE_BRICKS, area};
use crate::lib::{BlockState, Pos};

/// The hall's ends along x, and how far its post rows stand from its axis along z.
pub const WEST: i32 = -2;
pub const EAST: i32 = 22;
const HALF_WIDTH: i32 = 7;
/// Blocks from one post to the next along a row.
const POST_EVERY: usize = 4;
/// The height of the beams the posts carry; the roof starts a block above.
const BEAM: i32 = 6;
/// How far the roof reaches past the posts, on all four sides.
const OVERHANG: i32 = 1;
/// The roof lights: how far from the ridge each strip of glass lies, and from where to where
/// along the hall.
const LIGHT_ACROSS: [i32; 2] = [3, 4];
const LIGHT_ALONG: [[i32; 2]; 2] = [[1, 8], [12, 19]];

static OAK_SLAB: Block = Block::named("base:oak_planks_slab");
static BRICKS: Block = Block::named("base:bricks");
static GLASS: Block = Block::named("base:glass");

pub fn build(set: &Set) {
    let floor = area(
        Pos::new(WEST, -1, -HALF_WIDTH),
        Pos::new(EAST, -1, HALF_WIDTH),
    );
    set.fill(floor, BlockState::of(&STONE_BRICKS));
    for (index, x) in posts().enumerate() {
        frame(set, x, index % 2 == 1);
    }
    plates(set);
    roof(set);
    for end in [WEST, EAST] {
        gable(set, end);
    }
}

/// Where along the hall a pair of posts stands.
fn posts() -> impl Iterator<Item = i32> {
    (WEST..=EAST).step_by(POST_EVERY)
}

/// A pair of posts on their footings and the beam across them, with a lamp in its middle if
/// `lit`.
fn frame(set: &Set, x: i32, lit: bool) {
    for z in [-HALF_WIDTH, HALF_WIDTH] {
        set.place(Pos::new(x, 0, z), BlockState::of(&STONE_BRICKS));
        let post = area(Pos::new(x, 1, z), Pos::new(x, BEAM - 1, z));
        set.fill(post, BlockState::of(&OAK_LOG));
    }
    let across = area(
        Pos::new(x, BEAM, 1 - HALF_WIDTH),
        Pos::new(x, BEAM, HALF_WIDTH - 1),
    );
    set.fill(across, OAK_LOG.with(&[("axis", "z")]).into());
    if lit {
        set.place(Pos::new(x, BEAM, 0), BlockState::of(&GLOWSTONE));
    }
}

/// The beams along the tops of both post rows.
fn plates(set: &Set) {
    for z in [-HALF_WIDTH, HALF_WIDTH] {
        let plate = area(Pos::new(WEST, BEAM, z), Pos::new(EAST, BEAM, z));
        set.fill(plate, OAK_LOG.with(&[("axis", "x")]).into());
    }
}

/// The height of the roof's slab `across` blocks from the ridge, and whether it is the upper
/// half of its block: the roof climbs half a block with every block toward the ridge.
fn roof_slab(across: i32) -> (i32, bool) {
    let steps = HALF_WIDTH + OVERHANG - across.abs();
    (BEAM + 1 + steps / 2, steps % 2 == 1)
}

/// The roof from eaves to ridge, with its two strips of glass on either side.
fn roof(set: &Set) {
    let reach = HALF_WIDTH + OVERHANG;
    for z in -reach..=reach {
        let (y, upper) = roof_slab(z);
        let half = if upper { "top" } else { "bottom" };
        let row = area(
            Pos::new(WEST - OVERHANG, y, z),
            Pos::new(EAST + OVERHANG, y, z),
        );
        set.fill(row, OAK_SLAB.with(&[("type", half)]).into());
        if !LIGHT_ACROSS.contains(&z.abs()) {
            continue;
        }
        for [from, to] in LIGHT_ALONG {
            let light = area(Pos::new(from, y, z), Pos::new(to, y, z));
            set.fill(light, BlockState::of(&GLASS));
        }
    }
}

/// The brick gable over the end beam at `x`, up to the roof, a lamp in its middle.
fn gable(set: &Set, x: i32) {
    for z in 1 - HALF_WIDTH..HALF_WIDTH {
        let (roof, _) = roof_slab(z);
        let bricks = area(Pos::new(x, BEAM + 1, z), Pos::new(x, roof - 1, z));
        set.fill(bricks, BlockState::of(&BRICKS));
    }
    set.place(Pos::new(x, BEAM + 2, 0), BlockState::of(&GLOWSTONE));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_roof_climbs_half_a_block_a_block_to_the_ridge() {
        assert_eq!(roof_slab(8), (7, false), "the eaves, a lower half");
        assert_eq!(roof_slab(-7), (7, true), "then the upper half");
        assert_eq!(roof_slab(6), (8, false), "then a block up");
        assert_eq!(roof_slab(0), (11, false), "the ridge");
    }

    #[test]
    fn a_post_stands_at_each_end() {
        let posts: Vec<i32> = posts().collect();
        assert_eq!(posts.first(), Some(&WEST), "the west end");
        assert_eq!(posts.last(), Some(&EAST), "the east end");
    }
}
