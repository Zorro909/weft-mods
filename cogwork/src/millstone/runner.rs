//! The millstone's runner: an owned entity lying on the bed, shaped by
//! `assets/models/millstone_runner.geo.json`, a round stone with a wooden handle. Each turn the
//! server starts its clip in `assets/animations/millstone_runner.animation.json`, a whole turn
//! about its spindle, and the stone grinds.

use weft_sdk::{EntityKind, Output, Registry, World};

use super::MILL;
use crate::lib::Pos;

static RUNNER: EntityKind = EntityKind::named("millstone_runner");

/// The model, its texture and the file of its clip.
const MODEL: &str = "millstone_runner";
/// The clip: one whole turn.
const TURN: &str = "turn";
/// Ticks a turn takes; the next turn counts only after it.
pub const TURN_TICKS: u64 = 12;

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the stone, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&RUNNER), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&RUNNER, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &MILL)
        .is_some_and(|mill| mill.runner == id)
}

/// Keeps the runner `id` of the millstone `owner` at `pos` on its bed; returns its id.
pub fn place(world: &World, owner: u64, pos: Pos, id: u64) -> u64 {
    crate::owned::place(world, owner, id, &RUNNER, pos.floor_middle(), 0.0)
}

/// Turns the runner `id` of the millstone at `pos` once round.
pub fn turn(world: &World, id: u64, pos: Pos) {
    world.action_play(id, TURN, false);
    world.play_sound(&crate::sounds::GRIND, pos.center(), 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clip_is_a_turn_long() {
        let clips = include_str!("../../assets/animations/millstone_runner.animation.json");
        let length = format!("\"animation_length\": {}", TURN_TICKS as f32 / 20.0);
        assert!(clips.contains(&length), "a turn long");
        assert!(
            clips.contains("\"animation.millstone_runner.turn\""),
            "the clip"
        );
    }

    #[test]
    fn the_runner_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
    }
}
