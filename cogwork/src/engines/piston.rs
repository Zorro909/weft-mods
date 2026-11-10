//! An engine's piston: a plate riding up and down its trunk while it runs. It is an owned entity
//! standing at the foot of the trunk, shaped by a Blockbench model (`assets/models/piston.geo.json`,
//! or `piston_standing.geo.json` on an engine lying on its side), and each stroke is a clip of
//! `assets/animations/piston.animation.json` that the server starts, so every engine's piston
//! keeps time with the world's ticks.

use weft_sdk::*;

use super::{ENGINE, Engine, Kind, model};
use crate::lib::{Pos, point};

/// The pistons of one kind of engine: lying flat (engines facing up or down) and standing
/// (facing sideways), both in one texture.
struct Pistons {
    flat: EntityKind,
    standing: EntityKind,
    texture: &'static str,
}

static SPRING: Pistons = Pistons {
    flat: EntityKind::named("spring_piston"),
    standing: EntityKind::named("spring_piston_standing"),
    texture: "spring_piston",
};
static EMBER: Pistons = Pistons {
    flat: EntityKind::named("ember_piston"),
    standing: EntityKind::named("ember_piston_standing"),
    texture: "ember_piston",
};

fn pistons(kind: Kind) -> &'static Pistons {
    match kind {
        Kind::Spring => &SPRING,
        Kind::Ember => &EMBER,
    }
}

/// The model files, and the file of the clips both play.
const FLAT_MODEL: &str = "piston";
const STANDING_MODEL: &str = "piston_standing";
const CLIPS: &str = "piston";
/// The clips: the plate going 8 texels up the model and back (toward the south on a standing
/// one), and the same the other way.
const STROKE: &str = "stroke";
const STROKE_BACK: &str = "stroke_back";

/// Ticks the clips take to rise or to fall, and to rest at each end.
const TRAVEL: u32 = 6;
const REST: u32 = 3;
/// Ticks of one stroke, up and back down: as long as a clip.
pub const STROKE_TICKS: u32 = 2 * (TRAVEL + REST);
/// Texels from the middle of the block to the middle of the piston at rest, toward the base
/// plate: the plate is 4 thick and lies on the base plate (4..8 of the block's 16).
const REST_BELOW_MIDDLE: f64 = 2.0;

pub fn register(r: &mut Registry) {
    for pistons in [&SPRING, &EMBER] {
        let models = [
            (&pistons.flat, FLAT_MODEL),
            (&pistons.standing, STANDING_MODEL),
        ];
        for (entity, model) in models {
            // Small enough to stay inside the engine, so aiming at it aims at the block.
            crate::owned::kind(r.entity(entity), claims)
                .texture(pistons.texture)
                .animations(CLIPS)
                .size(0.25, 0.25);
            r.set_model(entity, model);
        }
    }
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &ENGINE)
        .is_some_and(|engine| engine.piston == id)
}

/// Ticks into the stroke all pistons keep time with, at the world's `time`.
fn shared(time: u64) -> u32 {
    (time % STROKE_TICKS as u64) as u32
}

/// Whether a stroke begins at `time`.
fn begins(time: u64) -> bool {
    shared(time) == 0
}

/// Whether a piston making a stroke has just reached the top at `time`: once a stroke.
pub fn at_top(time: u64) -> bool {
    shared(time) == TRAVEL
}

/// Whether the piston makes a stroke at `time`, having made one the tick before or not (`was`).
/// A stroke begins with every shared one while the engine runs, so all pistons rise and fall
/// together, and one begun goes on to its end: a stopped engine finishes its stroke and rests.
pub fn stroking(was: bool, running: bool, time: u64) -> bool {
    if begins(time) { running } else { was }
}

/// The piston of an engine whose trunk points `out`: which of the two it is, its turn in
/// degrees, and the clip that takes it up the trunk.
fn toward(pistons: &'static Pistons, out: Dir) -> (&'static EntityKind, f32, &'static str) {
    match out {
        Dir::Up => (&pistons.flat, 0.0, STROKE),
        Dir::Down => (&pistons.flat, 0.0, STROKE_BACK),
        Dir::South => (&pistons.standing, 0.0, STROKE),
        Dir::North => (&pistons.standing, 0.0, STROKE_BACK),
        // A quarter turn takes the standing model's south to the east.
        Dir::East => (&pistons.standing, 90.0, STROKE),
        Dir::West => (&pistons.standing, 90.0, STROKE_BACK),
    }
}

/// Keeps the piston of the engine `id` at the foot of its trunk, and starts its stroke when one
/// begins at `time`.
pub fn show(world: &World, id: u64, pos: Pos, engine: &mut Engine, time: u64) {
    let Some(out) = model::output(world, pos) else {
        return;
    };
    let (entity, yaw, clip) = toward(pistons(engine.kind()), out);
    // The model's pivot, the middle of the plate, is 2 texels above the entity's feet.
    let middle = point::toward(pos.center(), out, -REST_BELOW_MIDDLE / 16.0);
    let feet = middle.offset(0.0, -2.0 / 16.0, 0.0);
    engine.piston = crate::owned::place(world, id, engine.piston, entity, feet, yaw);
    if engine.is_stroking() && begins(time) {
        world.action_play(engine.piston, clip, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stroke_begins_only_with_the_shared_one() {
        let start = 5 * STROKE_TICKS as u64;
        assert!(stroking(false, true, start), "running at its start");
        assert!(!stroking(false, true, start + 1), "started late: waits");
        assert!(!stroking(false, false, start), "not running");
        let begun: Vec<u64> = (start..start + 3 * STROKE_TICKS as u64)
            .filter(|&time| begins(time))
            .collect();
        assert_eq!(begun.len(), 3, "once a stroke: {begun:?}");
    }

    #[test]
    fn a_stopped_engine_finishes_its_stroke_and_rests() {
        let start = 2 * STROKE_TICKS as u64;
        let mut stroke = stroking(false, true, start);
        for time in start + 1..start + STROKE_TICKS as u64 {
            stroke = stroking(stroke, false, time);
            assert!(stroke, "still on its way at {time}");
        }
        stroke = stroking(stroke, false, start + STROKE_TICKS as u64);
        assert!(!stroke, "at rest when the next would begin");
    }

    #[test]
    fn the_piston_reaches_the_top_once_a_stroke() {
        let tops: Vec<u64> = (0..2 * STROKE_TICKS as u64)
            .filter(|&time| at_top(time))
            .collect();
        assert_eq!(tops.len(), 2, "once in each of two strokes: {tops:?}");
        assert_eq!(tops[0], TRAVEL as u64, "after rising");
    }

    #[test]
    fn the_clips_keep_the_time_of_a_stroke() {
        let clips = include_str!("../../assets/animations/piston.animation.json");
        // Seconds as the file writes them: a tick is a twentieth.
        let secs = |ticks: u32| format!("{}", ticks as f32 / 20.0);
        let length = format!("\"animation_length\": {}", secs(STROKE_TICKS));
        assert_eq!(clips.matches(&length).count(), 2, "both a stroke long");
        for ticks in [TRAVEL, TRAVEL + REST, 2 * TRAVEL + REST] {
            let key = format!("\"{}\": [", secs(ticks));
            assert_eq!(clips.matches(&key).count(), 2, "a key at {key} in both");
        }
        for clip in [STROKE, STROKE_BACK] {
            let name = format!("\"animation.piston.{clip}\"");
            assert!(clips.contains(&name), "the clip {clip}");
        }
    }

    #[test]
    fn the_stroke_goes_up_the_trunk_whichever_way_it_points() {
        let up_the_model = [Dir::Up, Dir::South, Dir::East];
        for out in Dir::ALL {
            let (entity, yaw, clip) = toward(&SPRING, out);
            let flat = matches!(out, Dir::Up | Dir::Down);
            assert_eq!(std::ptr::eq(entity, &SPRING.flat), flat, "{out:?}: lying");
            let turned = matches!(out, Dir::East | Dir::West);
            assert_eq!(yaw == 90.0, turned, "{out:?}: turned");
            let forth = up_the_model.contains(&out);
            assert_eq!(clip == STROKE, forth, "{out:?}: the clip");
            assert_eq!(toward(&SPRING, out.opposite()).2 == STROKE, !forth);
        }
    }

    #[test]
    fn every_piston_has_its_model_clips_and_texture() {
        for file in [FLAT_MODEL, STANDING_MODEL] {
            let model = format!("models/{file}.geo.json");
            assert!(crate::has_asset(&model), "no {model}");
        }
        let clips = format!("animations/{CLIPS}.animation.json");
        assert!(crate::has_asset(&clips), "no {clips}");
        for pistons in [&SPRING, &EMBER] {
            let texture = format!("textures/entity/{}.png", pistons.texture);
            assert!(crate::has_asset(&texture), "no {texture}");
        }
    }
}
