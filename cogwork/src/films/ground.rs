//! Shaping the land for the set so that it looks built there: a pad is a terrace cut into the
//! slope on its high side and filled up on its low side, held by dry-stone walls where it
//! stands above or below the land around, and the land beyond a pad is left as it is. Paths
//! and lines of tubes follow the ground as they find it.

use showcase_api::Set;
use weft_sdk::Block;

use super::parts::{COBBLESTONE, GRASS};
use crate::lib::{BlockState, Pos, TypedWorld};

static DIRT: Block = Block::named("base:dirt");
static TALL_GRASS: Block = Block::named("base:tall_grass");
static MOSSY_COBBLESTONE: Block = Block::named("base:mossy_cobblestone");

/// Heights, from the site, between which the ground is looked for.
const LOWEST: i32 = -24;
const HIGHEST: i32 = 40;
/// Blocks above the ground that a tree may reach, and across from its trunk.
const TREE_HEIGHT: i32 = 12;
const TREE_REACH: i32 = 3;
/// How far from a pad a tree is felled, so that none is left cut in half at its edge.
const FELL_WITHIN: i32 = 5;
/// One wall block in this many is mossy.
const MOSSY_ONE_IN: i32 = 4;
/// The most blocks of wall a pad shows above lower land: below that an earth bank takes over,
/// stepping down a block a block, this many blocks out at most.
const WALL_HEIGHT: i32 = 2;
const BANK_REACH: i32 = 4;

/// A terrace: the columns from (`west`, `north`) to (`east`, `south`), their ground ending in
/// a block of `surface` at the height `top`.
pub struct Pad {
    pub west: i32,
    pub north: i32,
    pub east: i32,
    pub south: i32,
    pub top: i32,
    pub surface: &'static Block,
}

impl Pad {
    fn columns(&self) -> impl Iterator<Item = (i32, i32)> {
        let (west, east) = (self.west, self.east);
        (self.north..=self.south).flat_map(move |z| (west..=east).map(move |x| (x, z)))
    }

    fn contains(&self, x: i32, z: i32) -> bool {
        (self.west..=self.east).contains(&x) && (self.north..=self.south).contains(&z)
    }

    /// How many blocks outside it the column is, counted along x or z, whichever is more.
    fn away(&self, x: i32, z: i32) -> i32 {
        let beyond = |at: i32, low: i32, high: i32| (low - at).max(at - high).max(0);
        beyond(x, self.west, self.east).max(beyond(z, self.north, self.south))
    }

    /// The columns at most `within` blocks outside it.
    fn around(&self, within: i32) -> impl Iterator<Item = (i32, i32)> + '_ {
        let wider = Pad {
            west: self.west - within,
            north: self.north - within,
            east: self.east + within,
            south: self.south + within,
            ..*self
        };
        let outside: Vec<(i32, i32)> = wider.columns().collect();
        outside.into_iter().filter(|&(x, z)| !self.contains(x, z))
    }
}

/// The height of the highest block of ground in the column: trees, plants and liquids are not
/// ground.
pub fn ground_top(set: &Set, x: i32, z: i32) -> i32 {
    let world = set.world();
    let is_ground = |y: &i32| is_ground(world.block_typed(set.at(Pos::new(x, *y, z))));
    (LOWEST..=HIGHEST).rev().find(is_ground).unwrap_or(LOWEST)
}

fn is_ground(state: BlockState) -> bool {
    !state.can_build_into() && !is_tree(state)
}

fn is_tree(state: BlockState) -> bool {
    let name = state.block_name();
    name.ends_with("_log") || name.ends_with("_leaves")
}

/// Makes the pads: each cut down or filled up to its height, the trees on and near it gone;
/// an earth bank where the land falls far below a pad, and walls where a pad's edge still
/// stands above the land or the land above the pad.
pub fn terraces(set: &Set, pads: &[Pad]) {
    for pad in pads {
        fell_around(set, pad);
        for (x, z) in pad.columns() {
            level_column(set, pad, x, z);
        }
    }
    for pad in pads {
        bank(set, pad, pads);
    }
    for pad in pads {
        retain(set, pad, pads);
    }
}

/// How high the bank of a pad at `top` stands `out` blocks from its edge.
fn bank_top(top: i32, out: i32) -> i32 {
    top - WALL_HEIGHT - (out - 1)
}

/// Raises the land around `pad`, outside every pad, to a grassy bank where it lies lower.
fn bank(set: &Set, pad: &Pad, pads: &[Pad]) {
    for (x, z) in pad.around(BANK_REACH) {
        if pads.iter().any(|other| other.contains(x, z)) {
            continue;
        }
        let out = pad.away(x, z);
        let (ground, bank) = (ground_top(set, x, z), bank_top(pad.top, out));
        if ground >= bank {
            continue;
        }
        for y in ground..bank {
            set.place(Pos::new(x, y, z), BlockState::of(&DIRT));
        }
        set.place(Pos::new(x, bank, z), BlockState::of(&GRASS));
    }
}

/// Cuts or fills one column of `pad` to its height.
fn level_column(set: &Set, pad: &Pad, x: i32, z: i32) {
    let ground = ground_top(set, x, z);
    clear(set, x, z, pad.top + 1, ground.max(pad.top) + TREE_HEIGHT);
    for y in ground + 1..pad.top {
        set.place(Pos::new(x, y, z), BlockState::of(&DIRT));
    }
    set.place(Pos::new(x, pad.top, z), BlockState::of(pad.surface));
}

/// Takes away whatever stands in the column from `from` up to `to`.
fn clear(set: &Set, x: i32, z: i32, from: i32, to: i32) {
    let world = set.world();
    for y in from..=to {
        let pos = Pos::new(x, y, z);
        if world.block_typed(set.at(pos)) != BlockState::AIR {
            set.place(pos, BlockState::AIR);
        }
    }
}

/// Fells every tree whose trunk stands on `pad` or within `FELL_WITHIN` blocks of it.
fn fell_around(set: &Set, pad: &Pad) {
    let near: Vec<(i32, i32)> = pad.around(FELL_WITHIN).collect();
    for (x, z) in pad.columns().chain(near) {
        fell_if_trunk(set, x, z);
    }
}

/// Fells every tree whose trunk stands at most `within` blocks from the column, along x and z.
pub fn fell_near(set: &Set, x: i32, z: i32, within: i32) {
    for near_z in z - within..=z + within {
        for near_x in x - within..=x + within {
            fell_if_trunk(set, near_x, near_z);
        }
    }
}

/// Fells the tree whose trunk stands in the column, if one does; grass grows where it stood.
fn fell_if_trunk(set: &Set, x: i32, z: i32) {
    let ground = ground_top(set, x, z);
    let world = set.world();
    let trunk = world.block_typed(set.at(Pos::new(x, ground + 1, z)));
    if !trunk.block_name().ends_with("_log") {
        return;
    }
    fell(set, x, z, ground);
    let foot = Pos::new(x, ground, z);
    if world.block_typed(set.at(foot)).is(&DIRT) {
        set.place(foot, BlockState::of(&GRASS));
    }
}

/// Takes the tall grass off the ground at most `within` blocks from the column, along x and z.
pub fn mow(set: &Set, x: i32, z: i32, within: i32) {
    let world = set.world();
    for near_z in z - within..=z + within {
        for near_x in x - within..=x + within {
            let above = Pos::new(near_x, ground_top(set, near_x, near_z) + 1, near_z);
            if world.block_typed(set.at(above)).is(&TALL_GRASS) {
                set.place(above, BlockState::AIR);
            }
        }
    }
}

/// Takes away the tree whose trunk stands on the ground at `ground` in the column.
fn fell(set: &Set, x: i32, z: i32, ground: i32) {
    let world = set.world();
    for near_z in z - TREE_REACH..=z + TREE_REACH {
        for near_x in x - TREE_REACH..=x + TREE_REACH {
            for y in ground + 1..=ground + TREE_HEIGHT {
                let pos = Pos::new(near_x, y, near_z);
                if is_tree(world.block_typed(set.at(pos))) {
                    set.place(pos, BlockState::AIR);
                }
            }
        }
    }
}

/// Walls along the edge of `pad` wherever the land beside it, outside every pad, is not at
/// its height: the pad's own edge down to lower land, the land's face down to a lower pad.
fn retain(set: &Set, pad: &Pad, pads: &[Pad]) {
    for (x, z) in pad.columns() {
        for (beside_x, beside_z) in [(x - 1, z), (x + 1, z), (x, z - 1), (x, z + 1)] {
            if pads.iter().any(|other| other.contains(beside_x, beside_z)) {
                continue;
            }
            let beside = ground_top(set, beside_x, beside_z);
            if beside < pad.top {
                wall(set, x, z, beside + 1, pad.top);
            } else {
                // Its top block stays: grass over a stone face.
                wall(set, beside_x, beside_z, pad.top + 1, beside - 1);
            }
        }
    }
}

/// Dry stone in the column from `from` up to `to`, a block here and there mossy.
fn wall(set: &Set, x: i32, z: i32, from: i32, to: i32) {
    for y in from..=to {
        let mossy = (x * 7 + y * 13 + z * 31).rem_euclid(MOSSY_ONE_IN) == 0;
        let stone = if mossy {
            &MOSSY_COBBLESTONE
        } else {
            &COBBLESTONE
        };
        set.place(Pos::new(x, y, z), BlockState::of(stone));
    }
}

/// Lays `surface` as the top block of the ground in each of `columns`, as it lies.
pub fn pave(set: &Set, columns: impl Iterator<Item = (i32, i32)>, surface: &Block) {
    for (x, z) in columns {
        let ground = ground_top(set, x, z);
        set.place(Pos::new(x, ground, z), BlockState::of(surface));
        clear(set, x, z, ground + 1, ground + 1);
    }
}

/// The columns of a line through `corners`: from each along x to the next one's x, then along
/// z to it.
pub fn route(corners: &[(i32, i32)]) -> Vec<(i32, i32)> {
    let step = |from: i32, to: i32| (to - from).signum();
    let Some(&(mut x, mut z)) = corners.first() else {
        return Vec::new();
    };
    let mut columns = vec![(x, z)];
    for &(to_x, to_z) in &corners[1..] {
        while x != to_x {
            x += step(x, to_x);
            columns.push((x, z));
        }
        while z != to_z {
            z += step(z, to_z);
            columns.push((x, z));
        }
    }
    columns
}

/// The blocks of a line resting on the ground along `columns`, each next to the one before:
/// one above the ground in every column, and where the ground steps, those between in the
/// higher column's neighbour.
pub fn over_ground(set: &Set, columns: &[(i32, i32)]) -> Vec<Pos> {
    let heights: Vec<i32> = columns
        .iter()
        .map(|&(x, z)| ground_top(set, x, z) + 1)
        .collect();
    joined(columns, &heights)
}

/// The blocks of a line through `columns` at `heights`, with those that join two columns at
/// different heights: they stand in the lower one, up to the higher one's height.
fn joined(columns: &[(i32, i32)], heights: &[i32]) -> Vec<Pos> {
    let mut line = Vec::new();
    for (index, (&(x, z), &height)) in columns.iter().zip(heights).enumerate() {
        let beside = [index.checked_sub(1), Some(index + 1)]
            .into_iter()
            .flatten()
            .filter_map(|other| heights.get(other).copied());
        let reach = beside.fold(height, i32::max);
        line.extend((height..=reach).map(|y| Pos::new(x, y, z)));
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_route_goes_along_x_then_along_z() {
        let columns = route(&[(0, 0), (2, -2)]);
        assert_eq!(columns, [(0, 0), (1, 0), (2, 0), (2, -1), (2, -2)], "an L");
        let two = route(&[(0, 0), (1, 1), (2, 1)]);
        assert_eq!(two, [(0, 0), (1, 0), (1, 1), (2, 1)], "through a corner");
        assert_eq!(route(&[(3, 4)]), [(3, 4)], "a single column");
        assert!(route(&[]).is_empty(), "none");
    }

    #[test]
    fn a_line_over_a_step_is_joined_in_the_lower_column() {
        let columns = [(0, 0), (1, 0), (2, 0)];
        let line = joined(&columns, &[1, 3, 3]);
        let low = [Pos::new(0, 1, 0), Pos::new(0, 2, 0), Pos::new(0, 3, 0)];
        let flat = [Pos::new(1, 3, 0), Pos::new(2, 3, 0)];
        assert_eq!(
            line,
            [low.as_slice(), flat.as_slice()].concat(),
            "up a step"
        );
        let down = joined(&columns, &[2, 2, 1]);
        assert_eq!(down.last(), Some(&Pos::new(2, 2, 0)), "and down one");
        assert_eq!(down.len(), 4, "one block more than columns");
    }

    #[test]
    fn a_pad_knows_its_columns_and_those_around_it() {
        let pad = Pad {
            west: 0,
            north: 0,
            east: 2,
            south: 1,
            top: 0,
            surface: &DIRT,
        };
        assert_eq!(pad.columns().count(), 6, "3 by 2");
        assert!(pad.contains(2, 1) && !pad.contains(3, 1), "its east edge");
        assert_eq!(pad.around(1).count(), 5 * 4 - 6, "a ring one wide");
        assert_eq!(pad.away(1, 0), 0, "on it");
        assert_eq!(
            (pad.away(-2, 0), pad.away(4, 4)),
            (2, 3),
            "the farther way counts"
        );
    }

    #[test]
    fn a_bank_starts_a_wall_below_the_pad_and_steps_down() {
        assert_eq!(bank_top(0, 1), -WALL_HEIGHT, "right outside");
        assert_eq!(bank_top(0, 3), -WALL_HEIGHT - 2, "two further out");
    }
}
