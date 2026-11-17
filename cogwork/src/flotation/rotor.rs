//! The flotation cell's rotor: an owned entity hung from the bridge across the tank, shaped by
//! `assets/models/flotation_rotor.geo.json`, a steel shaft with a drive cap above the bridge and
//! two crossed blades down in the pulp. While the cell floats it loops the clip `spin` of
//! `assets/animations/flotation_rotor.animation.json`, a quarter turn at a time, and the froth
//! bubbles now and then.

use weft_sdk::{Output, Registry, World};

use super::{FLOAT, Float, Status};
use crate::lib::Pos;

static ROTOR: weft_sdk::EntityKind = weft_sdk::EntityKind::named("flotation_rotor");

/// The model, its texture and the file of its clip, and the clip.
const MODEL: &str = "flotation_rotor";
const SPIN: &str = "spin";
/// Ticks between bubblings while it floats: about as long as one lasts.
const BUBBLE_EVERY: u64 = 40;

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the tank, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&ROTOR), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&ROTOR, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &FLOAT)
        .is_some_and(|float| float.rotor == id)
}

/// Keeps the rotor of the cell `id` at `pos`, front toward `front`, under its bridge, spinning
/// and bubbling while it floats.
pub fn show(world: &World, id: u64, pos: Pos, front: weft_sdk::Dir, float: &mut Float) {
    let yaw = crate::owned::yaw(front);
    float.rotor = crate::owned::place(world, id, float.rotor, &ROTOR, pos.floor_middle(), yaw);
    let floating = float.status() == Status::Floating;
    crate::owned::keep_looping(world, id, float.rotor, SPIN, floating);
    if floating && world.game_time().is_multiple_of(BUBBLE_EVERY) {
        world.play_sound(&crate::sounds::FLOTATION, pos.center(), 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rotor_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/flotation_rotor.animation.json");
        assert!(
            clips.contains("\"animation.flotation_rotor.spin\""),
            "the clip"
        );
        assert!(clips.contains("\"loop\": true"), "it loops");
    }
}
