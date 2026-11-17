//! The core drill's crank and rod: an owned entity standing in the tripod, shaped by
//! `assets/models/core_drill_rod.geo.json`, a crank on top of the gearbox and the drill rod going
//! down into the ground. Each turn the server starts its clip in
//! `assets/animations/core_drill_rod.animation.json`, a whole turn of the crank and the rod, and
//! the bit grinds into the rock.

use weft_sdk::{EntityKind, Output, Registry, World};

use super::CORE;
use crate::lib::Pos;

static ROD: EntityKind = EntityKind::named("core_drill_rod");

/// The model, its texture and the file of its clip.
const MODEL: &str = "core_drill_rod";
/// The clip: one whole turn.
const TURN: &str = "turn";
/// Ticks a turn takes; the next turn counts only after it.
pub const TURN_TICKS: u64 = 12;

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the tripod, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&ROD), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&ROD, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &CORE)
        .is_some_and(|core| core.rod == id)
}

/// Keeps the rod `id` of the core drill `owner` at `pos` in its tripod; returns its id.
pub fn place(world: &World, owner: u64, pos: Pos, id: u64) -> u64 {
    crate::owned::place(world, owner, id, &ROD, pos.floor_middle(), 0.0)
}

/// Turns the crank and rod `id` of the core drill at `pos` once round.
pub fn turn(world: &World, id: u64, pos: Pos) {
    world.action_play(id, TURN, false);
    world.play_sound(&crate::sounds::CORE_DRILL, pos.center(), 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clip_is_a_turn_long() {
        let clips = include_str!("../../assets/animations/core_drill_rod.animation.json");
        let length = format!("\"animation_length\": {}", TURN_TICKS as f32 / 20.0);
        assert!(clips.contains(&length), "a turn long");
        assert!(
            clips.contains("\"animation.core_drill_rod.turn\""),
            "the clip"
        );
    }

    #[test]
    fn the_rod_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
    }
}
