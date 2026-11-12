//! A timber liquid tube's paddle wheel: an owned entity in the middle of its pump body, shaped by
//! `assets/models/timber_pump_wheel.geo.json`. Its paddles reach across the body from window to
//! window, so their ends show through the windows in front of the liquid. Each stroke after one
//! that pulled liquid in, the server starts its clip in `assets/animations/timber_pump_wheel.
//! animation.json`, a half turn, and the pump creaks and glugs.
//!
//! Its axle runs across the tube, so the windows facing the axle show the wheel's face.

use weft_sdk::{Dir, EntityKind, Output, Registry, World};

use super::{DRIVE, Drive};
use crate::lib::Pos;
use crate::liquid::{LiquidKey, NO_LIQUID};

static WHEEL: EntityKind = EntityKind::named("timber_pump_wheel");

/// The model, its texture and the file of its clip.
const MODEL: &str = "timber_pump_wheel";
/// The clip: a half turn about the axle, as long as a stroke.
const TURN: &str = "turn";

/// Ticks of a stroke, an engine's.
const STROKE: u32 = 18;
/// Texels from the entity's feet up to the axle, the middle of the block.
const AXLE_HEIGHT: f64 = 8.0;

pub fn register(r: &mut Registry) {
    // Small enough to stay inside the body, so aiming at it aims at the block.
    crate::owned::kind(r.entity(&WHEEL), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&WHEEL, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &DRIVE)
        .is_some_and(|drive| drive.wheel == id)
}

/// Whether a stroke begins at `time`: all wheels keep time with each other and the engines.
fn begins(time: u64) -> bool {
    time.is_multiple_of(STROKE as u64)
}

/// Keeps the wheel of the tube `id` at `pos` in place, and turns it through a stroke beginning
/// at `time` when it pulled liquid through the last one.
pub fn show(world: &World, id: u64, pos: Pos, drive: &mut Drive, time: u64) {
    let yaw = if axle_along_x(|dir| world.prop(pos, dir.name()) == Some("true")) {
        90.0
    } else {
        0.0
    };
    let feet = pos.center().offset(0.0, -AXLE_HEIGHT / 16.0, 0.0);
    drive.wheel = crate::owned::place(world, id, drive.wheel, &WHEEL, feet, yaw);
    if !begins(time) {
        return;
    }
    drive.pumping = pumping(drive.pulled, drive.liquid);
    if drive.pumping != NO_LIQUID {
        world.action_play(drive.wheel, TURN, false);
        world.play_sound(&crate::sounds::TIMBER_PUMP, pos.center(), 1.0);
    }
    drive.pulled = 0;
}

/// The liquid a wheel turns in through a stroke after one that `pulled` millibuckets of
/// `liquid`: none when it pulled none.
fn pumping(pulled: u32, liquid: LiquidKey) -> LiquidKey {
    if pulled > 0 { liquid } else { NO_LIQUID }
}

/// Whether the axle of a tube with arms toward `has_arm` runs east and west: when the tube goes
/// north or south and not east or west. The model's axle runs north and south.
fn axle_along_x(has_arm: impl Fn(Dir) -> bool) -> bool {
    let along = |a: Dir, b: Dir| has_arm(a) || has_arm(b);
    along(Dir::North, Dir::South) && !along(Dir::East, Dir::West)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_axle_runs_across_the_tube() {
        let arms = |dirs: &'static [Dir]| move |dir| dirs.contains(&dir);
        assert!(
            !axle_along_x(arms(&[Dir::East, Dir::West])),
            "east-west: along z"
        );
        assert!(
            axle_along_x(arms(&[Dir::North, Dir::Up])),
            "north and up: along x"
        );
        assert!(
            !axle_along_x(arms(&[Dir::North, Dir::East])),
            "a bend: along z"
        );
        assert!(
            !axle_along_x(arms(&[Dir::Up, Dir::Down])),
            "upright: along z"
        );
    }

    #[test]
    fn a_wheel_turns_in_what_it_pulled_the_stroke_before() {
        assert_eq!(pumping(10, 7), 7, "after a stroke it pulled in");
        assert_eq!(pumping(0, 7), NO_LIQUID, "after one it pulled nothing in");
    }

    #[test]
    fn a_stroke_begins_once_every_stroke() {
        let begun = (0..3 * STROKE as u64).filter(|&time| begins(time)).count();
        assert_eq!(begun, 3, "three strokes");
    }

    #[test]
    fn the_clip_is_a_stroke_long() {
        let clips = include_str!("../../assets/animations/timber_pump_wheel.animation.json");
        let length = format!("\"animation_length\": {}", STROKE as f32 / 20.0);
        assert!(clips.contains(&length), "a stroke long");
        assert!(
            clips.contains("\"animation.timber_pump_wheel.turn\""),
            "the clip"
        );
    }

    #[test]
    fn the_wheel_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
    }
}
