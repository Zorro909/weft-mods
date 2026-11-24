//! The jetty on the lake north of the hall, out from the shore: oak planks on piles, and at its
//! head, in four blocks of water, a pump over a gap in the planks, lowering its hose into the
//! lake. A cuboid loader stands on a stone post at the jetty's landward foot keeping that remote
//! cube loaded. Its line of liquid tubes runs back along the jetty and up the bank into the
//! hall's water tank, branching west at the shore to Pad R's refinery cooling tank.

use showcase_api::{Film, Look, Set, flyby};
use weft_sdk::Dir;

use super::camera::speeds;
use super::engine_bay::WATER_TANK;
use super::ground::{over_ground, route};
use super::parts::{OAK_LOG, OAK_PLANKS, STONE_BRICKS, area, engine, fuel, liquid_tubes, prefill};
use super::site::{WORKS_DOWN, above, lowered, lowered_block};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid_tube::LIQUID_TUBE;
use crate::loading::loader::CUBOID_LOADER;
use crate::pump::PUMP;

/// The jetty's middle row along x, the height of its planks (the lake's surface is the block
/// below, at y 63), and its two ends along z: its head out in the lake and its foot on the sand.
const MIDDLE_X: i32 = 12;
const DECK: i32 = -12;
const HEAD_Z: i32 = -25;
const FOOT_Z: i32 = -15;
/// The pump, a block in from the head, standing over the gap in the planks.
pub const PUMP_AT: Pos = Pos::new(MIDDLE_X, DECK + 1, HEAD_Z + 1);
/// Cuboid loader on a stone post beside the jetty's landward foot (in site coordinates).
pub const SMALL_LOADER: Pos = Pos::new(MIDDLE_X - 2, DECK + 2, FOOT_Z);
/// The sides of the pump its engines stand on; its line leaves it to the south.
const ENGINE_SIDES: [Dir; 3] = [Dir::West, Dir::East, Dir::North];
/// Where its piles stand along z, and how far they go down into the lake.
const PILES_Z: [i32; 3] = [HEAD_Z, -20, FOOT_Z];
const PILE_DEPTH: i32 = 3;
/// The line, from behind the pump to the block against the hall's water tank, and the
/// highest block it climbs to: a block over the hall's terrace.
const LINE: [(i32, i32); 2] = [(MIDDLE_X, HEAD_Z + 2), (MIDDLE_X, -7)];
const LINE_TOP: i32 = 1 - WORKS_DOWN;
/// How full the hall's water tank is when the film starts: two thirds.
const TANK_FULL: [u32; 2] = [2, 3];

/// Look targets and handoff point leaving the jetty toward the oil seep.
pub const JETTY_PUMP_LOOK: [f64; 3] = above(PUMP_AT, 0.0);
pub const SMALL_LOADER_LOOK: [f64; 3] = above(SMALL_LOADER, 0.0);
pub const PAST_JETTY: [f64; 3] = [-7.0, -5.5, -13.5];
const WORKS_LOOK: [f64; 3] = lowered([22.0, 0.0, -8.0]);

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
    set.fill(
        area(SMALL_LOADER.with_y(DECK - 1), SMALL_LOADER.add_y(-1)),
        BlockState::of(&STONE_BRICKS),
    );
    set.place(SMALL_LOADER, BlockState::of(&CUBOID_LOADER));
    liquid_tubes(set, &over_ground(set, &route(&LINE)));
}

/// Lights the pump's engines, with the water line and the hall's tank already holding what it
/// would have pumped by now.
pub fn pump(set: &Set) {
    for side in ENGINE_SIDES {
        fuel(set, PUMP_AT.toward(side));
    }
    let tank = WATER_TANK.blocks().map(lowered_block);
    let blocks = line(set, &LINE).into_iter().chain(tank);
    prefill(set, blocks, "base:water", TANK_FULL);
}

/// The line's tubes along `corners`, found where they stand: the hall's roof is over its end.
fn line(set: &Set, corners: &[(i32, i32)]) -> Vec<Pos> {
    let world = set.world();
    let columns = route(corners);
    let in_column = |(x, z): (i32, i32)| (DECK..=LINE_TOP).map(move |y| Pos::new(x, y, z));
    let blocks = columns.into_iter().flat_map(in_column);
    blocks
        .filter(|&block| world.block_typed(set.at(block)).is(&LIQUID_TUBE))
        .collect()
}

/// Fades into Part 2 high over the lake, sweeps low past the lake pump and pauses smoothly past
/// the shore cuboid loader on the way to the oil seep.
pub fn shots(film: Film) -> Film {
    let approach = [[38.0, 19.0, -78.0], [27.0, 6.0, -56.0], [17.0, -3.0, -39.0]];
    let round_jetty = [[17.0, -3.0, -39.0], [7.5, -7.0, -30.5], [3.5, -6.8, -23.0]];
    let past_loader = [[3.5, -6.8, -23.0], [-1.0, -6.2, -18.0], PAST_JETTY];
    film.cue("noon_2")
        .fade_in(1.5)
        .shot(
            flyby(&approach)
                .look(Look::Pan(WORKS_LOOK, JETTY_PUMP_LOOK))
                .ease(speeds(5.0, 5.0))
                .secs(8.0),
        )
        .title("cogwork.film.water", 5.5)
        .shot(
            flyby(&round_jetty)
                .look_at(JETTY_PUMP_LOOK)
                .ease(speeds(5.0, 3.0))
                .secs(5.5),
        )
        .title("cogwork.film.t4_small_loader", 4.5)
        .menu("cogwork:cuboid_loader", SMALL_LOADER, 3.5)
        .shot(
            flyby(&past_loader)
                .look(Look::Pan(JETTY_PUMP_LOOK, SMALL_LOADER_LOOK))
                .ease(speeds(3.0, 4.5))
                .secs(5.0),
        )
}
