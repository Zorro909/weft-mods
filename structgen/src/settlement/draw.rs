//! The grid's roads, squares and fields laid onto the ground at each column's own height:
//! gravel lanes worn in patches, cobbled streets between stone kerbs, wheat on farmland.

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

pub fn draw(p: &mut Plan, grid: &Grid) -> Result<(), PlanError> {
    for ([x, z], cell) in grid.columns() {
        let Some(y) = grid.height([x, z]) else {
            continue;
        };
        let worn = patch(x, z);
        let (floor, above) = match cell {
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
            Cell::Free | Cell::Green | Cell::Building | Cell::Blocked => continue,
        };
        p.set([x, y, z], floor)?;
        p.set_state([x, y + 1, z], above)?;
    }
    Ok(())
}

/// Whether column (`x`, `z`) is worn, in patches of a few blocks rather than single spots.
fn patch(x: i32, z: i32) -> bool {
    noise(x.div_euclid(3), z.div_euclid(3) + 9000) > 0.86 && noise(x, z) > 0.3
}
