//! The washer's paddle: an owned entity on the trough's bearings, shaped by
//! `assets/models/washer_paddle.geo.json`, six oak blades on an iron axle, the lowest dipping
//! into the water. While the washer washes, it loops the clip `turn` of
//! `assets/animations/washer_paddle.animation.json`, a blade on at a time, and the water
//! sloshes now and then.

use weft_sdk::{Dir, EntityKind, Output, Registry, World};

use super::{Status, WASH, Wash};
use crate::lib::Pos;

static PADDLE: EntityKind = EntityKind::named("washer_paddle");

/// The model, its texture and the file of its clip, and the clip.
const MODEL: &str = "washer_paddle";
const TURN: &str = "turn";
/// Ticks between sloshes while it washes: about as long as one lasts.
const SLOSH_EVERY: u64 = 30;

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the trough, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&PADDLE), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&PADDLE, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &WASH)
        .is_some_and(|wash| wash.paddle == id)
}

/// Keeps the paddle of the washer `id` at `pos`, front toward `front`, on its bearings, turning
/// and sloshing while it washes.
pub fn show(world: &World, id: u64, pos: Pos, front: Dir, wash: &mut Wash) {
    let yaw = crate::owned::yaw(front);
    wash.paddle = crate::owned::place(world, id, wash.paddle, &PADDLE, pos.floor_middle(), yaw);
    let washing = wash.status() == Status::Washing;
    crate::owned::keep_looping(world, id, wash.paddle, TURN, washing);
    if washing && world.game_time().is_multiple_of(SLOSH_EVERY) {
        world.play_sound(&crate::sounds::WASHER, pos.center(), 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Seconds the clip takes to turn the paddle a blade on.
    const TURN_SECONDS: f32 = 0.5;

    #[test]
    fn the_paddle_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/washer_paddle.animation.json");
        assert!(
            clips.contains("\"animation.washer_paddle.turn\""),
            "the clip"
        );
        let length = format!("\"animation_length\": {TURN_SECONDS}");
        assert!(clips.contains(&length), "a blade on");
        assert!(clips.contains("\"loop\": true"), "it loops");
    }
}
