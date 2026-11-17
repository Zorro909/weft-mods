//! The grinding mill's runner: an owned entity on the bed stone, shaped by
//! `assets/models/grinding_mill_runner.geo.json`, a granite stone on an iron spindle with a
//! wooden crown wheel on it. While the mill grinds, it loops the clip `turn` of
//! `assets/animations/grinding_mill_runner.animation.json`, a whole turn a second, and the stone
//! rumbles now and then.

use weft_sdk::{EntityKind, Output, Registry, World};

use super::{GRINDER, Grinder, Status};
use crate::lib::Pos;

static RUNNER: EntityKind = EntityKind::named("grinding_mill_runner");

/// The model, its texture and the file of its clip, and the clip.
const MODEL: &str = "grinding_mill_runner";
const TURN: &str = "turn";
/// Ticks between rumbles while it grinds: about as long as one lasts.
const RUMBLE_EVERY: u64 = 34;

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the block, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&RUNNER), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&RUNNER, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &GRINDER)
        .is_some_and(|grinder| grinder.runner == id)
}

/// Keeps the runner of the grinding mill `id` at `pos` on its bed stone, turning and rumbling while
/// it grinds.
pub fn show(world: &World, id: u64, pos: Pos, grinder: &mut Grinder) {
    let feet = pos.floor_middle();
    grinder.runner = crate::owned::place(world, id, grinder.runner, &RUNNER, feet, 0.0);
    let grinding = grinder.status() == Status::Grinding;
    crate::owned::keep_looping(world, id, grinder.runner, TURN, grinding);
    if grinding && world.game_time().is_multiple_of(RUMBLE_EVERY) {
        world.play_sound(&crate::sounds::GRINDING_MILL, pos.center(), 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Seconds one turn of the clip takes.
    const TURN_SECONDS: f32 = 1.0;

    #[test]
    fn the_runner_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/grinding_mill_runner.animation.json");
        assert!(
            clips.contains("\"animation.grinding_mill_runner.turn\""),
            "the clip"
        );
        let length = format!("\"animation_length\": {TURN_SECONDS}");
        assert!(clips.contains(&length), "a turn long");
        assert!(clips.contains("\"loop\": true"), "it loops");
    }
}
