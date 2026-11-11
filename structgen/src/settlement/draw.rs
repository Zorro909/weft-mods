//! The grid's roads, squares and fields laid onto the ground at each column's own height:
//! gravel lanes worn in patches, cobbled streets between stone kerbs, wheat on farmland, with
//! the air above each cleared. The world grows no trees where a settlement stands: the engine
//! keeps them out of a structure's box.

use weft_sdk::*;
use weftkit::{BlockState, Pos};

use super::grid::{Cell, Grid};
use super::plan::Plan;
use super::shape::noise;

static AIR: Block = Block::named("core:air");
static GRAVEL: Block = Block::named("base:gravel");
static DIRT: Block = Block::named("base:dirt");
static COBBLESTONE: Block = Block::named("base:cobblestone");
static MOSSY: Block = Block::named("base:mossy_cobblestone");
static KERB: Block = Block::named("base:stone_bricks");
static FARMLAND: Block = Block::named("base:farmland");
static WHEAT: Block = Block::named("base:wheat");

pub fn draw(p: &mut Plan, grid: &Grid) {
    for ([x, z], cell) in grid.columns() {
        let (Some(y), Some((floor, above))) = (grid.height([x, z]), surface(cell, patch(x, z)))
        else {
            continue;
        };
        p.set(Pos::new(x, y, z), floor);
        p.set_state(Pos::new(x, y + 1, z), above);
        p.fill(Pos::new(x, y + 2, z), Pos::new(x, y + CLEAR, z), &AIR);
    }
}

/// The floor and the block above it of a road or field column, `worn` in patches; `None` for
/// a column nothing is laid on.
fn surface(cell: Cell, worn: bool) -> Option<(&'static Block, BlockState)> {
    Some(match cell {
        Cell::Lane if worn => (&DIRT, BlockState::AIR),
        Cell::Lane => (&GRAVEL, BlockState::AIR),
        Cell::Kerb => (&KERB, BlockState::AIR),
        Cell::Street | Cell::Square if worn => (&MOSSY, BlockState::AIR),
        Cell::Street | Cell::Square => (&COBBLESTONE, BlockState::AIR),
        Cell::Furrow => (&DIRT, BlockState::AIR),
        Cell::Crop(age) => {
            let wheat = BlockState::of(&WHEAT).with_prop("age", &age.to_string());
            (&FARMLAND, wheat.unwrap_or(BlockState::AIR))
        }
        Cell::Free | Cell::Green | Cell::Building | Cell::Blocked => return None,
    })
}

/// How high above a road or field the air is cleared: a hill it cuts through.
const CLEAR: i32 = 11;

/// Whether column (`x`, `z`) is worn, in patches of a few blocks rather than single spots.
fn patch(x: i32, z: i32) -> bool {
    noise(x.div_euclid(3), z.div_euclid(3) + 9000) > 0.86 && noise(x, z) > 0.3
}
