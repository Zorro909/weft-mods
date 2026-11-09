//! The grid's roads, squares and fields laid onto the ground at each column's own height:
//! gravel lanes worn in patches, cobbled streets between stone kerbs, wheat on farmland. Above
//! each, the air is cleared, and the leaves beside it, so no tree the world grew there stands
//! in the way or leaves a crown floating.

use structgen_api::{Plan, PlanError};
use weft_sdk::*;

use super::grid::{Cell, Grid};
use super::shape::noise;

static AIR: Block = Block::named("core:air");
static GRAVEL: Block = Block::named("base:gravel");
static DIRT: Block = Block::named("base:dirt");
static COBBLESTONE: Block = Block::named("base:cobblestone");
static MOSSY: Block = Block::named("base:mossy_cobblestone");
static KERB: Block = Block::named("base:stone_bricks");
static FARMLAND: Block = Block::named("base:farmland");
static WHEAT: Block = Block::named("base:wheat");
static LEAVES: Tag = Tag::named("leaves");

pub fn draw(p: &mut Plan, grid: &Grid) -> Result<(), PlanError> {
    for ([x, z], cell) in grid.columns() {
        let Some(y) = grid.height([x, z]) else {
            continue;
        };
        let (from, to) = ([x, y + 2, z], [x, y + CLEAR, z]);
        if let Some((floor, above)) = surface(cell, patch(x, z)) {
            p.set([x, y, z], floor)?;
            p.set_state([x, y + 1, z], above)?;
            p.fill(from, to, &AIR)?;
        } else if beside(grid, [x, z], cell) {
            // Leaves only: a trunk standing here keeps the crown it still holds up.
            p.replace(from, to, &LEAVES, &AIR)?;
        }
    }
    Ok(())
}

/// The floor and the block above it of a road or field column, `worn` in patches; `None` for
/// a column nothing is laid on.
fn surface(cell: Cell, worn: bool) -> Option<(&'static Block, u32)> {
    Some(match cell {
        Cell::Lane if worn => (&DIRT, AIR.id()),
        Cell::Lane => (&GRAVEL, AIR.id()),
        Cell::Kerb => (&KERB, AIR.id()),
        Cell::Street | Cell::Square if worn => (&MOSSY, AIR.id()),
        Cell::Street | Cell::Square => (&COBBLESTONE, AIR.id()),
        Cell::Furrow => (&DIRT, AIR.id()),
        Cell::Crop(age) => {
            let wheat = state_with(WHEAT.id(), "age", &age.to_string());
            (&FARMLAND, wheat.unwrap_or(AIR.id()))
        }
        Cell::Free | Cell::Green | Cell::Building | Cell::Blocked => return None,
    })
}

/// Whether open ground at `at` is near enough a road or field that the crown of a tree cut
/// off there could hang over it.
fn beside(grid: &Grid, at: [i32; 2], cell: Cell) -> bool {
    matches!(cell, Cell::Free | Cell::Green) && grid.near(at, CROWN, laid)
}

/// Whether a road or field is laid on a column of `cell`.
fn laid(cell: Cell) -> bool {
    cell.is_road() || matches!(cell, Cell::Furrow | Cell::Crop(_))
}

/// How high above a road or field the air is cleared: a tall spruce's height.
const CLEAR: i32 = 11;
/// How far a tree's crown reaches out from its trunk.
const CROWN: i32 = 3;

/// Whether column (`x`, `z`) is worn, in patches of a few blocks rather than single spots.
fn patch(x: i32, z: i32) -> bool {
    noise(x.div_euclid(3), z.div_euclid(3) + 9000) > 0.86 && noise(x, z) > 0.3
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settlement::grid::row;

    #[test]
    fn crowns_are_cleared_beside_a_lane_but_not_off_a_house() {
        let mut grid = row(&[Some(64); 6]);
        grid.put([0, 0], Cell::Lane);
        grid.put([1, 0], Cell::Building);
        assert!(
            !beside(&grid, [0, 0], Cell::Lane),
            "the lane is cleared as a lane"
        );
        assert!(!beside(&grid, [1, 0], Cell::Building), "not the house");
        assert!(beside(&grid, [3, 0], Cell::Free), "three from the lane");
        assert!(!beside(&grid, [4, 0], Cell::Free), "four from it");
    }
}
