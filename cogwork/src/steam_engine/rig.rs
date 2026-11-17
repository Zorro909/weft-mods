//! The steam engine's moving parts: an owned entity on its bed plate, shaped by
//! `assets/models/steam_engine_rig.geo.json`, the piston rod coming out of the cylinder, the
//! connecting rod and a flywheel of two discs with the crank pin between them. While the engine
//! runs it loops the clip `run` of `assets/animations/steam_engine_rig.animation.json`, a turn
//! of the flywheel a second, and chuffs, with a puff of steam out of the cylinder, twice a turn.

use weft_sdk::{Dir, EntityKind, Output, Registry, Vec3, World};

use super::{STEAM_ENGINE_STATE, SteamEngine};
use crate::lib::Pos;

static RIG: EntityKind = EntityKind::named("steam_engine_rig");

/// The model, its texture and the file of its clip, and the clip.
const MODEL: &str = "steam_engine_rig";
const RUN: &str = "run";
/// Ticks a turn of the flywheel takes, the clip's length; it chuffs twice a turn.
pub const TURN_TICKS: u64 = 20;
/// Where the cylinder lets out its steam: blocks toward the front from the middle, and up.
const EXHAUST_FORWARD: f64 = 0.25;
const EXHAUST_UP: f64 = 0.85;
/// The crank's throw and the connecting rod's length, in model pixels: the clip moves the piston
/// and tilts the rod by them.
#[cfg(test)]
const THROW: f64 = 2.0;
#[cfg(test)]
const CONROD: f64 = 4.0;

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the block, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&RIG), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&RIG, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &STEAM_ENGINE_STATE)
        .is_some_and(|engine| engine.rig == id)
}

/// Keeps the rig of the steam engine `id` at `pos`, front toward `front`, on its bed plate,
/// running, chuffing and puffing while it runs.
pub fn show(world: &World, id: u64, pos: Pos, front: Dir, engine: &mut SteamEngine, running: bool) {
    let yaw = crate::owned::yaw(front);
    engine.rig = crate::owned::place(world, id, engine.rig, &RIG, pos.floor_middle(), yaw);
    crate::owned::keep_looping(world, id, engine.rig, RUN, running);
    if !running || !world.game_time().is_multiple_of(TURN_TICKS / 2) {
        return;
    }
    let ahead = front.offset();
    let forward = Vec3::new(f64::from(ahead.x), 0.0, f64::from(ahead.z)) * EXHAUST_FORWARD;
    let exhaust = pos.floor_middle() + forward + Vec3::new(0.0, EXHAUST_UP, 0.0);
    world.play_sound(&crate::sounds::STEAM_CHUFF, exhaust, 1.0);
    crate::steam::puff(world, exhaust, 3);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rig_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/steam_engine_rig.animation.json");
        assert!(
            clips.contains("\"animation.steam_engine_rig.run\""),
            "the clip"
        );
        let seconds = TURN_TICKS as f32 / 20.0;
        let length = format!("\"animation_length\": {seconds}");
        assert!(clips.contains(&length), "a turn long");
        assert!(clips.contains("\"loop\": true"), "it loops");
    }

    /// The keyframes of `channel` of the clip's `bone`: each time and its value on `axis`.
    fn keys(bone: &str, channel: &str, axis: usize) -> Vec<(f64, f64)> {
        let file = include_str!("../../assets/animations/steam_engine_rig.animation.json");
        let clips: serde_json::Value = serde_json::from_str(file).expect("the clip is JSON");
        let keys = &clips["animations"]["animation.steam_engine_rig.run"]["bones"][bone][channel];
        let keys = keys.as_object().expect("keyframes");
        let value = |key: &serde_json::Value| key[axis].as_f64().expect("a number");
        keys.iter()
            .map(|(time, key)| (time.parse().expect("a time"), value(key)))
            .collect()
    }

    #[test]
    fn the_rods_follow_the_crank() {
        let turn = |time: f64| time * 20.0 / TURN_TICKS as f64 * std::f64::consts::TAU;
        let wheel = keys("flywheel", "rotation", 0);
        assert_eq!(wheel.len(), 13, "a key every twelfth of a turn");
        for (time, degrees) in wheel {
            assert!(
                (degrees + turn(time).to_degrees()).abs() < 0.05,
                "flywheel at {time}"
            );
        }
        for (time, along) in keys("piston", "position", 2) {
            let (sin, cos) = turn(time).sin_cos();
            let out =
                THROW * (1.0 - cos) + CONROD - (CONROD.powi(2) - (THROW * sin).powi(2)).sqrt();
            assert!(
                (along - out).abs() < 0.01,
                "piston at {time}: {along}, not {out}"
            );
        }
        for (time, degrees) in keys("conrod", "rotation", 0) {
            let tilt = (THROW * turn(time).sin() / CONROD).asin().to_degrees();
            assert!(
                (degrees - tilt).abs() < 0.05,
                "rod at {time}: {degrees}, not {tilt}"
            );
        }
    }
}
