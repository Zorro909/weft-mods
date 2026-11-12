//! Entities that only show what a block entity is doing (an excavator's gantry, an engine's
//! piston). Each belongs to an owner, moves where its owner puts it, and leaves when the owner
//! is gone or no longer counts it as its own.

use weft_sdk::*;

/// The block entity an owned entity belongs to.
static OWNER: Component<u64> = Component::named("cogwork:owner");

pub fn register(r: &mut Registry) {
    r.entity_component(&OWNER, 0u64, component_flags::SAVE);
}

/// Whether the block entity `owner` still counts the entity `id` as its own.
pub type Claims = fn(world: &World, owner: u64, id: u64) -> bool;

/// Makes `builder` an owned entity: moved only by its owner, never hurt or pushed, with no egg.
///
/// An owner that loads before its entities makes new ones, and the old ones arrive later still
/// moving as they were when saved. `claims` tells those apart, so they leave instead of
/// drifting off forever.
pub fn kind<'a>(builder: EntityBuilder<'a>, claims: Claims) -> EntityBuilder<'a> {
    builder
        .kinematic()
        .invulnerable()
        .unpushable()
        .no_egg()
        .attach(&OWNER)
        .on_tick(move |tick| leave_unclaimed(tick, claims))
}

fn leave_unclaimed(tick: &EntityTick, claims: Claims) {
    let world = tick.world();
    for &id in tick.ids() {
        let owner = world.component(id, &OWNER).unwrap_or(0);
        if owner == 0 || world.entity(owner).is_none() || !claims(&world, owner, id) {
            world.despawn(id);
        }
    }
}

/// Moves `owner`'s entity `id` to `pos`, turned `yaw` degrees; makes it anew if it is gone, of
/// another kind or turned another way. Returns its id.
pub fn place(world: &World, owner: u64, id: u64, kind: &EntityKind, pos: Vec3, yaw: f32) -> u64 {
    let Some(entity) = world.entity(id).filter(|_| id != 0) else {
        return spawn(world, owner, kind, pos, yaw);
    };
    let turn = if entity.kind == kind.id() {
        squaring(entity.yaw, yaw)
    } else {
        Squaring::Lost
    };
    match turn {
        Squaring::Lost => {
            if world.component(id, &OWNER) == Some(owner) {
                world.despawn(id);
            }
            return spawn(world, owner, kind, pos, yaw);
        }
        Squaring::Look(next) => {
            world.entity_modify(id).yaw(next.to_radians());
        }
        Squaring::Square => {}
    }
    // A kinematic entity moves by its velocity after this tick's systems: just that speed lands
    // it on `pos`, and it stands still once there. Placing it as well would overshoot.
    let velocity = (pos - entity.pos) * 20.0;
    if velocity != entity.vel {
        world.entity_modify(id).velocity(velocity);
    }
    id
}

/// Degrees past its yaw a new owned entity looks, then as far short of it, a tick each, so that
/// its body faces its yaw. The models are drawn turned with the body. Weft 26.11.4 starts the
/// body at 0 whatever yaw the entity is made with, and turns one standing still only to within
/// 50 degrees of its yaw (`physics::turn_body`): 50 past brings it onto the yaw, and 50 short
/// leaves it there, or brings one that started at its yaw back onto it.
const SQUARING: f32 = 50.0;
/// Radians two yaws may differ by and still be the same.
const SAME_YAW: f32 = 1e-3;

/// What an owned entity looking `now` radians does next for its body to face `yaw` degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Squaring {
    /// Looks this many degrees.
    Look(f32),
    /// Faces `yaw`, body and all.
    Square,
    /// Turned another way: it is made anew. A body turned later would only come within 50
    /// degrees of its new yaw.
    Lost,
}

fn squaring(now: f32, yaw: f32) -> Squaring {
    let looks = |degrees: f32| apart(now, degrees.to_radians()) < SAME_YAW;
    if looks(yaw) {
        Squaring::Square
    } else if looks(yaw + SQUARING) {
        Squaring::Look(yaw - SQUARING)
    } else if looks(yaw - SQUARING) {
        Squaring::Look(yaw)
    } else {
        Squaring::Lost
    }
}

/// Radians between two yaws in radians, the short way round.
fn apart(a: f32, b: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    ((a - b + PI).rem_euclid(TAU) - PI).abs()
}

/// Makes `owner`'s entity of `kind` at `pos`, looking past `yaw` degrees to square up its body.
fn spawn(world: &World, owner: u64, kind: &EntityKind, pos: Vec3, yaw: f32) -> u64 {
    let looks = (yaw + SQUARING).to_radians();
    let id = world.spawn_with(kind, pos, &Spawn::new().yaw(looks));
    world.set_component(id, &OWNER, owner);
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The body of one standing still after a tick, as Weft 26.11.4 turns it: to within 50
    /// degrees of where it looks. Degrees.
    fn turned_body(body: f32, looks: f32) -> f32 {
        let off = looks - body;
        body + off - off.clamp(-50.0, 50.0)
    }

    /// The body of an entity made to face `yaw` degrees, its body starting at `body`, once it has
    /// squared up; and the ticks that took.
    fn squared_body(yaw: f32, body: f32) -> (f32, u32) {
        let (mut body, mut looks) = (body, yaw + SQUARING);
        for tick in 0..10 {
            body = turned_body(body, looks);
            match squaring(looks.to_radians(), yaw) {
                Squaring::Look(next) => looks = next,
                Squaring::Square => return (body, tick),
                Squaring::Lost => panic!("lost at {looks} degrees"),
            }
        }
        panic!("never squared up to {yaw} degrees")
    }

    #[test]
    fn a_new_body_ends_up_facing_its_yaw() {
        for yaw in [0.0, 90.0, 180.0, -90.0] {
            let (body, ticks) = squared_body(yaw, 0.0);
            assert_eq!(body, yaw, "started at 0, as the engine does");
            assert_eq!(ticks, 2, "two ticks to {yaw}");
            let (body, _) = squared_body(yaw, yaw + SQUARING);
            assert_eq!(body, yaw, "started where it looked");
        }
    }

    #[test]
    fn a_turned_entity_is_made_anew() {
        let quarter = 90f32.to_radians();
        assert_eq!(squaring(quarter, 90.0), Squaring::Square, "as it was");
        assert_eq!(squaring(0.0, 90.0), Squaring::Lost, "turned a quarter");
        assert_eq!(squaring(quarter, 0.0), Squaring::Lost, "turned back");
    }

    #[test]
    fn yaws_a_turn_apart_are_the_same() {
        let (half, whole) = (std::f32::consts::PI, std::f32::consts::TAU);
        assert!(
            apart(-half + 0.0001, half) < SAME_YAW,
            "either side of the back"
        );
        assert!(apart(whole, 0.0) < SAME_YAW, "a whole turn");
        assert!(apart(1.0, 0.0) > SAME_YAW, "a radian");
    }
}
