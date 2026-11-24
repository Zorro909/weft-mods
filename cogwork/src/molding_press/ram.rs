//! The molding press's ram: an owned entity under its crosshead, shaped by
//! `assets/models/molding_press_ram.geo.json`, a steel rod and platen resting raised. At each
//! press it plays the clip `press` of `assets/animations/molding_press_ram.animation.json`,
//! coming down onto the mold and rising again, with a hydraulic thump.

use weft_sdk::{Dir, EntityKind, Output, Registry, World};

use super::{PRESSING, Pressing};
use crate::lib::Pos;

static RAM: EntityKind = EntityKind::named("molding_press_ram");

/// The model, its texture and the file of its clip, and the clip.
const MODEL: &str = "molding_press_ram";
const PRESS: &str = "press";

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the block, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&RAM), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&RAM, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &PRESSING)
        .is_some_and(|pressing| pressing.ram == id)
}

/// Keeps the ram of the press `id` at `pos`, front toward `front`, under its crosshead, and
/// brings it down with a thump when it `pressed` a part.
pub fn show(world: &World, id: u64, pos: Pos, front: Dir, pressing: &mut Pressing, pressed: bool) {
    let yaw = crate::owned::yaw(front);
    let feet = pos.floor_middle();
    pressing.ram = crate::owned::place(world, id, pressing.ram, &RAM, feet, yaw);
    if pressed {
        world.action_play(pressing.ram, PRESS, false);
        world.play_sound(&crate::sounds::PRESS, pos.center(), 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ram_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/molding_press_ram.animation.json");
        assert!(
            clips.contains("\"animation.molding_press_ram.press\""),
            "the clip"
        );
        assert!(clips.contains("\"loop\": false"), "once a press");
    }
}
