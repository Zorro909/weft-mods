//! The shaker screen's tray: an owned entity on the frame's springs, shaped by
//! `assets/models/shaker_screen_tray.geo.json`, a framed iron mesh with a heap of Rubble on it.
//! While the screen sifts, it loops the clip `shake` of
//! `assets/animations/shaker_screen_tray.animation.json`, jolting back and forth toward its
//! front, and rattles now and then.

use weft_sdk::{Dir, EntityKind, Output, Registry, World};

use super::{SHAKER, Shaker, Status};
use crate::lib::Pos;

static TRAY: EntityKind = EntityKind::named("shaker_screen_tray");

/// The model, its texture and the file of its clip, and the clip.
const MODEL: &str = "shaker_screen_tray";
const SHAKE: &str = "shake";
/// Ticks between rattles while it sifts: about as long as one lasts.
const RATTLE_EVERY: u64 = 30;

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the frame, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&TRAY), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&TRAY, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &SHAKER)
        .is_some_and(|shaker| shaker.tray == id)
}

/// Keeps the tray of the shaker screen `id` at `pos`, front toward `front`, on its springs,
/// shaking and rattling while it sifts.
pub fn show(world: &World, id: u64, pos: Pos, front: Dir, shaker: &mut Shaker) {
    let yaw = crate::owned::yaw(front);
    shaker.tray = crate::owned::place(world, id, shaker.tray, &TRAY, pos.floor_middle(), yaw);
    let sifting = shaker.status() == Status::Sifting;
    crate::owned::keep_looping(world, id, shaker.tray, SHAKE, sifting);
    if sifting && world.game_time().is_multiple_of(RATTLE_EVERY) {
        world.play_sound(&crate::sounds::SHAKER_SCREEN, pos.center(), 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Seconds one shake of the clip takes, there and back.
    const SHAKE_SECONDS: f32 = 0.3;

    #[test]
    fn the_tray_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/shaker_screen_tray.animation.json");
        assert!(
            clips.contains("\"animation.shaker_screen_tray.shake\""),
            "the clip"
        );
        let length = format!("\"animation_length\": {SHAKE_SECONDS}");
        assert!(clips.contains(&length), "a shake long");
        assert!(clips.contains("\"loop\": true"), "it loops");
    }
}
