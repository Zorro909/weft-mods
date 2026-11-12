//! The jetty on the lake north of the hall: oak planks on piles, and at its head a pump over
//! a gap in the planks, lowering its hose into the lake. Its line of liquid tubes runs back
//! along the jetty and up the bank into the hall's water tank. Water climbs a line only as
//! fast as it fills it, a block every two seconds, so the film starts as if this pump had been
//! at work a while: its line full and the tank two thirds full, and it goes on pumping.

use showcase_api::Set;
use weft_sdk::{Block, Dir};

use super::engine_bay::WATER_TANK;
use super::ground::{over_ground, route};
use super::parts::{OAK_LOG, area, engine, fuel, liquid_tubes, prefill};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid_tube::LIQUID_TUBE;
use crate::pump::PUMP;

/// The jetty's middle row along x, the height of its planks (the lake's surface is the block
/// below), and its two ends along z: its head out in the lake and its foot on the sand.
const MIDDLE_X: i32 = 12;
const DECK: i32 = -12;
const HEAD_Z: i32 = -47;
const FOOT_Z: i32 = -37;
/// The pump, a block in from the head, standing over the gap in the planks.
pub const PUMP_AT: Pos = Pos::new(MIDDLE_X, DECK + 1, HEAD_Z + 1);
/// The sides of the pump its engines stand on; its line leaves it to the south.
const ENGINE_SIDES: [Dir; 3] = [Dir::West, Dir::East, Dir::North];
/// Where its piles stand along z, and how far they go down into the lake.
const PILES_Z: [i32; 3] = [HEAD_Z, -42, FOOT_Z];
const PILE_DEPTH: i32 = 3;
/// The line, from behind the pump to the block against the hall's water tank, and the
/// highest block it climbs to.
const LINE: [(i32, i32); 2] = [(MIDDLE_X, HEAD_Z + 2), (MIDDLE_X, -7)];
const LINE_TOP: i32 = 1;
/// How full the hall's water tank is when the film starts: two thirds.
const TANK_FULL: [u32; 2] = [2, 3];

static OAK_PLANKS: Block = Block::named("base:oak_planks");

pub fn build(set: &Set) {
    let deck = area(
        Pos::new(MIDDLE_X - 1, DECK, HEAD_Z),
        Pos::new(MIDDLE_X + 1, DECK, FOOT_Z),
    );
    set.fill(deck, BlockState::of(&OAK_PLANKS));
    for z in PILES_Z {
        for x in [MIDDLE_X - 1, MIDDLE_X + 1] {
            let pile = area(Pos::new(x, DECK - PILE_DEPTH, z), Pos::new(x, DECK, z));
            set.fill(pile, BlockState::of(&OAK_LOG));
        }
    }
    set.place(PUMP_AT.toward(Dir::Down), BlockState::AIR);
    set.place(PUMP_AT, BlockState::of(&PUMP));
    for side in ENGINE_SIDES {
        engine(set, PUMP_AT.toward(side), side.opposite());
    }
    liquid_tubes(set, &over_ground(set, &route(&LINE)));
}

/// Lights the pump's engines, with the line and the tank already holding what it would have
/// pumped by now.
pub fn pump(set: &Set) {
    for side in ENGINE_SIDES {
        fuel(set, PUMP_AT.toward(side));
    }
    let blocks = line(set).into_iter().chain(WATER_TANK.blocks());
    prefill(set, blocks, "base:water", TANK_FULL);
}

/// The line's tubes, found where they stand: the hall's roof is over its end by now.
fn line(set: &Set) -> Vec<Pos> {
    let world = set.world();
    let columns = route(&LINE);
    let in_column = |(x, z): (i32, i32)| (DECK..=LINE_TOP).map(move |y| Pos::new(x, y, z));
    let blocks = columns.into_iter().flat_map(in_column);
    blocks
        .filter(|&block| world.block_typed(set.at(block)).is(&LIQUID_TUBE))
        .collect()
}
