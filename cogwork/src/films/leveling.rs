//! Leveling east of the benches: three survey markers in an L round a small hill and two holes.
//! The camera uses the corner marker, a leveler goes down next to it, and the tuning wrench
//! turns the ember engine beside it, which pointed away, into it: it clears the hill and fills
//! the holes.

use showcase_api::Set;
use weft_sdk::*;

use super::parts::{area, engine, fuel, in_world};
use crate::leveler::LEVELER;
use crate::lib::{BlockState, Pos};
use crate::markers::SURVEY_MARKER;

/// The marked rectangle, x 10 to 18 and z 14 to 22: its corner marker, and the two along it.
pub const CORNER: Pos = Pos::new(10, 0, 22);
const MARKERS: [Pos; 3] = [CORNER, Pos::new(18, 0, 22), Pos::new(10, 0, 14)];
/// The leveler, south of the corner marker, and its engine to the east.
pub const LEVELER_AT: Pos = Pos::new(10, 0, 23);
pub const ENGINE: Pos = Pos::new(11, 0, 23);
/// The middle of the rectangle, where the hill stands.
pub const MIDDLE: [f64; 3] = [14.5, 0.5, 18.5];

static DIRT: Block = Block::named("base:dirt");
static GRASS: Block = Block::named("base:grass");

pub fn build(set: &Set) {
    for marker in MARKERS {
        set.place(marker, SURVEY_MARKER.id().into());
    }
    let hill = area(Pos::new(13, 0, 16), Pos::new(16, 0, 19));
    set.fill(hill, BlockState::of(&DIRT));
    let top = area(Pos::new(14, 1, 17), Pos::new(15, 1, 18));
    set.fill(top, BlockState::of(&GRASS));
    let air = BlockState::AIR;
    set.fill(area(Pos::new(11, -2, 15), Pos::new(12, -1, 16)), air);
    set.fill(area(Pos::new(16, -1, 20), Pos::new(17, -1, 21)), air);
    // Pointing away from where the leveler goes, until the wrench turns it.
    engine(set, ENGINE, Dir::East);
}

/// The camera uses the corner marker: the rectangle shows.
pub fn survey(set: &Set) {
    crate::markers::survey(&set.world(), in_world(set, CORNER));
}

/// The leveler goes down next to the corner marker and takes the markers' rectangle.
pub fn place_leveler(set: &Set) {
    set.place(LEVELER_AT, LEVELER.with(&[("facing", "south")]).into());
}

/// Fills the leveler's stock and lights the engine, which the wrench then turns into it.
pub fn turn_engine(set: &Set) {
    let world = set.world();
    if let Some(dirt) = Item::lookup("base:dirt") {
        let stock = world.inventory(in_world(set, LEVELER_AT)).group("input");
        stock.set(0, ItemStack::new(dirt, 64));
    }
    fuel(set, ENGINE);
    crate::engines::turn(&world, in_world(set, ENGINE));
}
