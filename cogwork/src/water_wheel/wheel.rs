//! A water wheel's wheel: an owned entity centered on its hub, shaped by
//! `assets/models/water_wheel.geo.json`, eight spokes, a rim and paddles three blocks across. It
//! turns by the looping clips of `assets/animations/water_wheel.animation.json`, `turn` while the
//! wheel runs and the slow `creep` while its store is full, and creaks now and then as it goes.

use weft_sdk::{Component, Dir, EntityKind, Output, Registry, World, value};

use super::flow::Speed;
use super::{WHEEL, Wheel};
use crate::lib::Pos;

/// Named apart from the block, whose block entity takes the block's name.
static WHEEL_ENTITY: EntityKind = EntityKind::named("water_wheel_paddles");

/// The model, its texture and its clips.
const MODEL: &str = "water_wheel";
const TURN: &str = "turn";
const CREEP: &str = "creep";
/// Ticks between creaks while it turns, and while it creeps.
const CREAK_TURNING: u64 = 60;
const CREAK_CREEPING: u64 = 160;

value! {
    /// What the wheel entity was last shown doing, its `Speed` as a number: never saved, so a
    /// loaded wheel starts its clip again.
    #[derive(Default)]
    pub struct Shown {
        entity: u64,
        speed: u32,
    }
}
pub static SHOWN: Component<Shown> = Component::named("cogwork:water_wheel_shown");

pub fn register(r: &mut Registry) {
    // Not saved: a loaded wheel's clip starts again.
    r.entity_component(&SHOWN, Shown::default(), 0);
    // Small, so aiming at the hub aims at the block.
    crate::owned::kind(r.entity(&WHEEL_ENTITY), claims)
        .texture(MODEL)
        .animations(MODEL)
        .size(0.25, 0.25);
    r.set_model(&WHEEL_ENTITY, MODEL);
}

fn claims(world: &World, owner: u64, id: u64) -> bool {
    world
        .component(owner, &WHEEL)
        .is_some_and(|wheel| wheel.entity == id)
}

/// Keeps the wheel of the water wheel `id` at `pos`, axle toward `out`, on its hub, turning at
/// `speed`.
pub fn show(world: &World, id: u64, pos: Pos, out: Dir, wheel: &mut Wheel, speed: Speed) {
    // The model's axle runs north and south.
    let yaw = if matches!(out, Dir::East | Dir::West) {
        90.0
    } else {
        0.0
    };
    let feet = pos.floor_middle();
    wheel.entity = crate::owned::place(world, id, wheel.entity, &WHEEL_ENTITY, feet, yaw);
    let now = Shown {
        entity: wheel.entity,
        speed: speed as u32,
    };
    if world.component(id, &SHOWN).unwrap_or_default() != now {
        world.action_play(wheel.entity, clip(speed), speed != Speed::Still);
        world.set_component(id, &SHOWN, now);
    }
    if creaks(speed, world.game_time()) {
        world.play_sound(&crate::sounds::WATER_WHEEL, pos.center(), 1.0);
    }
}

/// The clip of a wheel turning at `speed`; none for a still one.
fn clip(speed: Speed) -> &'static str {
    match speed {
        Speed::Still => "",
        Speed::Creep => CREEP,
        Speed::Turn => TURN,
    }
}

/// Whether a wheel turning at `speed` creaks at `time`.
fn creaks(speed: Speed, time: u64) -> bool {
    match speed {
        Speed::Still => false,
        Speed::Creep => time.is_multiple_of(CREAK_CREEPING),
        Speed::Turn => time.is_multiple_of(CREAK_TURNING),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wheel_creaks_more_often_the_faster_it_turns() {
        let count = |speed| (0..480).filter(|&time| creaks(speed, time)).count();
        assert!(count(Speed::Turn) > count(Speed::Creep), "faster, more");
        assert_eq!(count(Speed::Still), 0, "still, silent");
    }

    #[test]
    fn the_clips_turn_it_once_round_in_their_time() {
        let clips = include_str!("../../assets/animations/water_wheel.animation.json");
        for (clip, seconds) in [(TURN, 4), (CREEP, 16)] {
            assert!(
                clips.contains(&format!("\"animation.water_wheel.{clip}\"")),
                "{clip}"
            );
            let length = format!("\"animation_length\": {seconds}");
            assert!(clips.contains(&length), "{clip} is {seconds} seconds");
        }
        assert!(clips.contains("[0, 0, -360]"), "once round");
    }

    #[test]
    fn the_wheel_has_its_model_clips_and_texture() {
        for file in [
            format!("models/{MODEL}.geo.json"),
            format!("animations/{MODEL}.animation.json"),
            format!("textures/entity/{MODEL}.png"),
            "models/block/water_wheel.json".to_string(),
            "textures/item/water_wheel.png".to_string(),
        ] {
            assert!(crate::has_asset(&file), "no {file}");
        }
    }
}
