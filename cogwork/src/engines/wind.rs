//! Winding a spring engine: each use with an empty hand turns its crank once and winds energy
//! into its spring, which then runs down into the engine's store a little every tick. A redstone
//! signal holds it still, a brake; nothing else stops it but an empty spring or a full store.

use weft_sdk::{Arg, Output, World};

use super::{ENGINE, Kind, piston};
use crate::lib::Pos;

/// Energy one turn of the crank winds in, and the most the spring holds.
const PER_TURN: u32 = 200;
const FULL: u32 = 10_000;
/// Energy a tick the spring gives while it runs down.
const PER_TICK: u32 = 10;

/// `player` uses the engine at `pos`: with an empty hand, a spring engine's crank turns once.
/// False passes the click on.
pub fn wind(world: &World, player: u64, pos: Pos) -> bool {
    if !world
        .inventory(player)
        .get(world.selected_slot(player))
        .is_empty()
    {
        return false;
    }
    let id = world.block_entity(pos);
    let Some(mut engine) = world.component(id, &ENGINE) else {
        return false;
    };
    if engine.kind() != Kind::Spring {
        return false;
    }
    let Some(wound) = turned(engine.wound) else {
        world.hint("cogwork.engine.fully_wound", &[]).to(player);
        return true;
    };
    engine.wound = wound;
    piston::wind(world, engine.piston);
    world.play_sound(&crate::sounds::WIND, pos.center(), 1.0);
    let args = [Arg::Int(i64::from(wound)), Arg::Int(i64::from(FULL))];
    world.hint("cogwork.engine.wound", &args).to(player);
    world.set_component(id, &ENGINE, engine);
    true
}

/// The spring after one more turn of a crank wound to `wound`; none when it is full.
fn turned(wound: u32) -> Option<u32> {
    (wound < FULL).then(|| (wound + PER_TURN).min(FULL))
}

/// What a spring wound to `wound` gives over `ticks`, into a store with `room` left: nothing while
/// `braked`.
pub fn release(wound: u32, ticks: u32, room: i64, braked: bool) -> u32 {
    if braked {
        return 0;
    }
    let room = u32::try_from(room.max(0)).unwrap_or(u32::MAX);
    wound.min(PER_TICK.saturating_mul(ticks)).min(room)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_turn_winds_in_two_hundred_up_to_ten_thousand() {
        assert_eq!(turned(0), Some(200), "the first turn");
        assert_eq!(turned(9_900), Some(10_000), "the last, cut short");
        assert_eq!(turned(10_000), None, "full: no more");
    }

    #[test]
    fn a_wound_spring_gives_ten_a_tick() {
        assert_eq!(release(10_000, 1, 1_000, false), 10, "a tick");
        assert_eq!(release(10_000, 20, 1_000, false), 200, "a catch-up of 20");
        assert_eq!(release(4, 1, 1_000, false), 4, "the last of it");
        assert_eq!(release(0, 1, 1_000, false), 0, "run down");
    }

    #[test]
    fn a_signal_brakes_it_and_a_full_store_holds_it() {
        assert_eq!(release(10_000, 1, 1_000, true), 0, "braked");
        assert_eq!(release(10_000, 1, 3, false), 3, "only what fits");
        assert_eq!(release(10_000, 1, 0, false), 0, "full");
    }

    #[test]
    fn its_messages_have_text() {
        for key in ["cogwork.engine.wound", "cogwork.engine.fully_wound"] {
            assert!(crate::translated(key), "{key} in every lang file");
        }
    }
}
