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
    let current = world.entity(id).filter(|_| id != 0);
    let Some(entity) = current.filter(|entity| still_fits(entity.kind, entity.yaw, kind.id(), yaw))
    else {
        if current.is_some() && world.component(id, &OWNER) == Some(owner) {
            world.despawn(id);
        }
        return spawn(world, owner, kind, pos, yaw);
    };
    // A kinematic entity moves by its velocity after this tick's systems: just that speed lands
    // it on `pos`, and it stands still once there. Placing it as well would overshoot.
    let velocity = (pos - entity.pos) * 20.0;
    if velocity != entity.vel {
        world.entity_modify(id).velocity(velocity);
    }
    id
}

/// Whether an entity of `kind` turned `yaw` radians can stay to be of `wanted` turned
/// `wanted_yaw` degrees. A turned one can't: clients turn its body after its yaw like a mob's,
/// which stops 50 degrees short when it stands still.
fn still_fits(kind: u32, yaw: f32, wanted: u32, wanted_yaw: f32) -> bool {
    kind == wanted && yaw == wanted_yaw.to_radians()
}

fn spawn(world: &World, owner: u64, kind: &EntityKind, pos: Vec3, yaw: f32) -> u64 {
    let id = world.spawn_with(kind, pos, &Spawn::new().yaw(yaw.to_radians()));
    world.set_component(id, &OWNER, owner);
    id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_turned_entity_is_made_anew() {
        assert!(still_fits(3, 90f32.to_radians(), 3, 90.0), "as it was");
        assert!(!still_fits(3, 0.0, 3, 90.0), "turned a quarter");
        assert!(!still_fits(3, 0.0, 4, 0.0), "another kind");
    }
}
