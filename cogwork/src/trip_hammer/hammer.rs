//! The trip hammer's hammer: an owned entity on its frame, shaped by
//! `assets/models/trip_hammer_hammer.geo.json`, an iron helve on a pin with a steel head, resting
//! raised. Just before each strike it plays the clip `strike` of
//! `assets/animations/trip_hammer_hammer.animation.json`, falling onto the bloom and rising again;
//! at the strike it clangs, sparks fly off the bloom and the steam that lifted it hisses out.

use weft_sdk::{Dir, EntityKind, Output, Registry, Vec3, World};

use super::strike::Step;
use super::{HAMMERING, Hammering};
use crate::lib::Pos;
use crate::steel::SPARK;

static HAMMER: EntityKind = EntityKind::named("trip_hammer_hammer");

/// The model, its texture and the file of its clip, and the clip.
const MODEL: &str = "trip_hammer_hammer";
const STRIKE: &str = "strike";
/// Where the bloom lies: blocks toward the front from the middle, and up.
const BLOOM_FORWARD: f64 = 0.19;
const BLOOM_UP: f64 = 0.7;
/// Where its steam cylinder lets out its steam: blocks toward the back, and up.
const EXHAUST_BACK: f64 = 0.3;
const EXHAUST_UP: f64 = 0.6;
const SPARKS: u32 = 10;

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the block, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&HAMMER), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&HAMMER, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &HAMMERING)
        .is_some_and(|hammering| hammering.hammer == id)
}

/// Keeps the hammer of the trip hammer `id` at `pos`, front toward `front`, on its frame, and
/// shows the `step` it took: falling, and striking.
pub fn show(world: &World, id: u64, pos: Pos, front: Dir, hammering: &mut Hammering, step: Step) {
    let yaw = crate::owned::yaw(front);
    let feet = pos.floor_middle();
    hammering.hammer = crate::owned::place(world, id, hammering.hammer, &HAMMER, feet, yaw);
    match step {
        Step::Fell => world.action_play(hammering.hammer, STRIKE, false),
        Step::Struck | Step::Forged => strike(world, feet, front),
        Step::Waited | Step::Lifted => {}
    }
}

/// A strike on the anvil of the trip hammer standing at `feet`: a clang, sparks and a hiss of
/// steam.
fn strike(world: &World, feet: Vec3, front: Dir) {
    let ahead = front.offset();
    let forward = Vec3::new(f64::from(ahead.x), 0.0, f64::from(ahead.z));
    let bloom = feet + forward * BLOOM_FORWARD + Vec3::new(0.0, BLOOM_UP, 0.0);
    world.play_sound(&crate::sounds::TRIP_HAMMER, bloom, 1.0);
    let burst = world.spawn_particles(&SPARK, bloom).count(SPARKS);
    drop(
        burst
            .spread(0.12)
            .velocity(Vec3::new(0.0, 0.2, 0.0))
            .speed(0.25),
    );
    let exhaust = feet - forward * EXHAUST_BACK + Vec3::new(0.0, EXHAUST_UP, 0.0);
    crate::steam::puff(world, exhaust, 2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hammer_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/trip_hammer_hammer.animation.json");
        assert!(
            clips.contains("\"animation.trip_hammer_hammer.strike\""),
            "the clip"
        );
        assert!(clips.contains("\"loop\": false"), "once a strike");
    }
}
