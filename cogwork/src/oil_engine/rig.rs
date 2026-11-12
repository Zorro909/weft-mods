//! An oil engine's rig: two upright cylinders whose pistons drive a crankshaft and a flywheel,
//! standing on the engine's top and inside its footprint, with a chimney at its front left that
//! lets off vapor while the engine runs. It is an owned entity shaped by
//! `assets/models/oil_engine_rig.geo.json`, there while the engine is built; each stroke is a
//! clip of `assets/animations/oil_engine_rig.animation.json`, a turn of the crank, which the
//! server starts, with a boom at each of its two beats, so every engine keeps time with the
//! world's ticks.

use weft_sdk::{Dir, EntityKind, Output, Particle, Registry, Vec3, World};

use super::pattern::Cube;
use super::{OIL_ENGINE, OilEngine};

static RIG: EntityKind = EntityKind::named("oil_engine_rig");
static VAPOR: Particle = Particle::named("oil_engine_vapor");

/// The model, its texture and the file of its clip.
const MODEL: &str = "oil_engine_rig";
/// The clip: a turn of the crank, the pistons going down and up in turn.
const TURN: &str = "turn";
/// Ticks of one stroke: twice a small engine's, for a heavier beat.
pub const STROKE_TICKS: u32 = 36;
/// Blocks from the middle of the engine's cube up to its top, where the rig stands.
const TOP_ABOVE_MIDDLE: f64 = 1.5;
/// Ticks between the wisps of vapor it lets off while it runs, and between the puffs it gives
/// at each beat, as a piston comes up: two a stroke.
const WISP_EVERY: u64 = 4;
const BEAT_EVERY: u64 = STROKE_TICKS as u64 / 2;
/// The top of the chimney in the model, in texels from the middle of its feet: to its right (left
/// for less than none), up, and toward its front.
const CHIMNEY_TOP: (f64, f64, f64) = (-18.0, 17.0, 8.0);

pub fn register(r: &mut Registry) {
    // Small, so aiming at the engine's top aims at its blocks.
    crate::owned::kind(r.entity(&RIG), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&RIG, MODEL);
    r.particle(&VAPOR)
        // Small to big: it billows as it rises and thins out.
        .textures(&[
            "core:generic_3",
            "core:generic_4",
            "core:generic_5",
            "core:generic_6",
            "core:generic_7",
        ])
        .life(2.0, 3.0)
        .size(0.4, 0.6)
        .gravity(-0.4)
        .drag(0.9)
        .color(0xE8E8E8C0, 0xFFFFFF00)
        .translucent();
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &OIL_ENGINE)
        .is_some_and(|engine| engine.rig == id)
}

/// Whether a stroke begins at `time`.
fn begins(time: u64) -> bool {
    time.is_multiple_of(STROKE_TICKS as u64)
}

/// Whether the rig makes a stroke at `time`, having made one the tick before or not (`was`): a
/// stroke begins with every shared one when the engine has run since the last began (`ran`),
/// so one that runs only now and then, its energy hardly used, keeps turning; and one begun goes
/// on to its end.
pub fn stroking(was: bool, ran: bool, time: u64) -> bool {
    if begins(time) { ran } else { was }
}

impl OilEngine {
    /// Keeps its rig's stroke at `time`, the engine running this tick or not.
    pub fn keep_stroke(&mut self, running: bool, time: u64) {
        let ran = self.ran != 0 || running;
        self.stroke = stroking(self.is_stroking(), ran, time) as u32;
        self.ran = (ran && !begins(time)) as u32;
    }
}

/// Keeps the rig of the engine `id`, built as `cube`, on its top facing its front, starts its
/// stroke when one begins at `time`, and while it runs lets off vapor, with a boom and a puff
/// at each beat.
pub fn show(world: &World, id: u64, cube: Cube, engine: &mut OilEngine, time: u64) {
    let top = cube.middle.center().offset(0.0, TOP_ABOVE_MIDDLE, 0.0);
    engine.rig = crate::owned::place(world, id, engine.rig, &RIG, top, yaw(cube.front));
    if !engine.is_stroking() {
        return;
    }
    if begins(time) {
        world.action_play(engine.rig, TURN, false);
    }
    let chimney = turned(top, cube.front, CHIMNEY_TOP);
    if time.is_multiple_of(BEAT_EVERY) {
        world.play_sound(&crate::sounds::OIL_ENGINE_STROKE, top, 1.0);
        vapor(world, chimney, 3, 1.4);
    } else if time.is_multiple_of(WISP_EVERY) {
        vapor(world, chimney, 1, 0.6);
    }
}

/// `count` puffs of vapor out of the chimney's top at `chimney`, rising `speed` blocks a second.
fn vapor(world: &World, chimney: Vec3, count: u32, speed: f64) {
    let burst = world.spawn_particles(&VAPOR, chimney).count(count);
    drop(
        burst
            .spread(0.08)
            .velocity(Vec3::new(0.0, speed, 0.0))
            .speed(0.3),
    );
}

/// Takes the rig of the engine `id` away: it is no longer built.
pub fn remove(world: &World, engine: &mut OilEngine) {
    if engine.rig != 0 {
        world.despawn(engine.rig);
        engine.rig = 0;
    }
}

/// The turn of a rig whose front faces `front`, in degrees: the model's front is its north, and
/// a quarter turn takes it west.
fn yaw(front: Dir) -> f32 {
    match front {
        Dir::West => 90.0,
        Dir::South => 180.0,
        Dir::East => -90.0,
        _ => 0.0,
    }
}

/// The point `(right, up, ahead)` texels from `feet` on a rig whose front faces `front`.
fn turned(feet: Vec3, front: Dir, (right, up, ahead): (f64, f64, f64)) -> Vec3 {
    let (ahead_step, right_step) = (front.offset(), front.clockwise().offset());
    let along =
        |ahead_by: i32, right_by: i32| (ahead_by as f64 * ahead + right_by as f64 * right) / 16.0;
    feet.offset(
        along(ahead_step.x, right_step.x),
        up / 16.0,
        along(ahead_step.z, right_step.z),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chimney_stays_at_the_front_left_however_it_faces() {
        let feet = Vec3::ZERO;
        let north = turned(feet, Dir::North, CHIMNEY_TOP);
        assert!(
            north.z < 0.0 && north.x < 0.0,
            "north: front north, left west {north:?}"
        );
        let east = turned(feet, Dir::East, CHIMNEY_TOP);
        assert!(
            east.x > 0.0 && east.z < 0.0,
            "east: front east, left north {east:?}"
        );
    }

    #[test]
    fn a_stroke_goes_on_to_its_end() {
        assert!(stroking(false, true, 0), "begins when it ran");
        assert!(!stroking(false, false, 0), "not when it did not");
        assert!(stroking(true, false, 5), "goes on when it stops");
        assert!(
            !stroking(true, false, STROKE_TICKS as u64),
            "and rests at the next"
        );
    }

    #[test]
    fn an_engine_running_now_and_then_keeps_turning() {
        let mut engine = OilEngine::default();
        let stroke = STROKE_TICKS as u64;
        for time in 1..3 * stroke {
            // Its energy hardly used: it runs one tick in nine, never as a stroke begins.
            engine.keep_stroke(time % 9 == 4, time);
            if time >= stroke {
                assert!(engine.is_stroking(), "turning at {time}");
            }
        }
    }

    #[test]
    fn the_rig_has_its_model_clip_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
        let clips = include_str!("../../assets/animations/oil_engine_rig.animation.json");
        let length = format!("\"animation_length\": {}", STROKE_TICKS as f32 / 20.0);
        assert!(clips.contains(&length), "a stroke long");
    }
}
