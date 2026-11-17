//! The clarifier's rake: an owned entity on the post in the middle of the tank, shaped by
//! `assets/models/clarifier_rake.geo.json`, two arms with tines dragging over the floor and a
//! drive head on top. While the clarifier settles it loops the clip `turn` of
//! `assets/animations/clarifier_rake.animation.json`, slowly, half a turn every four seconds.

use weft_sdk::{EntityKind, Registry, World};

use super::{SETTLE, Settle, Status};
use crate::lib::Pos;

static RAKE: EntityKind = EntityKind::named("clarifier_rake");

/// The model, its texture and the file of its clip, and the clip.
const MODEL: &str = "clarifier_rake";
const TURN: &str = "turn";

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the tank, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&RAKE), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&RAKE, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &SETTLE)
        .is_some_and(|settle| settle.rake == id)
}

/// Keeps the rake of the clarifier `id` at `pos` on its post, turning while it settles.
pub fn show(world: &World, id: u64, pos: Pos, settle: &mut Settle) {
    settle.rake = crate::owned::place(world, id, settle.rake, &RAKE, pos.floor_middle(), 0.0);
    let settling = settle.status() == Status::Settling;
    crate::owned::keep_looping(world, id, settle.rake, TURN, settling);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rake_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/clarifier_rake.animation.json");
        assert!(
            clips.contains("\"animation.clarifier_rake.turn\""),
            "the clip"
        );
        assert!(clips.contains("\"loop\": true"), "it loops");
    }
}
