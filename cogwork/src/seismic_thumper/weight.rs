//! The thumper's weight: an owned entity in its frame, shaped by
//! `assets/models/seismic_thumper_weight.geo.json`, a riveted steel weight on a rod that runs up
//! through the yoke, resting raised. Each drop plays the clip `thump` of
//! `assets/animations/seismic_thumper_weight.animation.json`: it falls onto the strike plate, then
//! is wound up again. The ground booms and dust flies off the plate.

use weft_sdk::{Dir, EntityKind, Output, Particle, Registry, Vec3, World};

use super::SURVEY;
use crate::lib::Pos;

static WEIGHT: EntityKind = EntityKind::named("seismic_thumper_weight");
/// What flies off the strike plate as the weight lands.
static DUST: Particle = Particle::named("core:poof");

/// The model, its texture and the file of its clip, and the clip.
const MODEL: &str = "seismic_thumper_weight";
const THUMP: &str = "thump";
/// How far above the floor of its block the strike plate lies.
const PLATE: f64 = 3.0 / 16.0;

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the frame, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&WEIGHT), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&WEIGHT, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &SURVEY)
        .is_some_and(|survey| survey.weight == id)
}

/// Keeps the weight `id` of the thumper `owner` at `pos`, gearbox to the right of `front`, in its
/// frame; returns its id.
pub fn place(world: &World, owner: u64, pos: Pos, front: Dir, id: u64) -> u64 {
    let yaw = crate::owned::yaw(front);
    crate::owned::place(world, owner, id, &WEIGHT, pos.floor_middle(), yaw)
}

/// Drops the weight `id` of the thumper at `pos` once: the clip, the boom and the dust.
pub fn drop_on(world: &World, id: u64, pos: Pos) {
    world.action_play(id, THUMP, false);
    world.play_sound(&crate::sounds::THUMP, pos.center(), 1.0);
    let plate = pos.floor_middle() + Vec3::new(0.0, PLATE, 0.0);
    let burst = world.spawn_particles(&DUST, plate).count(10);
    drop(
        burst
            .spread(0.4)
            .velocity(Vec3::new(0.0, 0.05, 0.0))
            .speed(0.12),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_weight_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/seismic_thumper_weight.animation.json");
        assert!(
            clips.contains("\"animation.seismic_thumper_weight.thump\""),
            "the clip"
        );
        assert!(
            clips.contains("\"animation_length\": 2"),
            "as long as a thump's turn"
        );
    }
}
