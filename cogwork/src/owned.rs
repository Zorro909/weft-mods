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
pub type Claims = fn(w: &World, owner: u64, id: u64) -> bool;

/// Makes `b` an owned entity: moved only by its owner, never hurt or pushed, with no egg.
///
/// An owner that loads before its entities makes new ones, and the old ones arrive later still
/// moving as they were when saved. `claims` tells those apart, so they leave instead of
/// drifting off forever.
pub fn kind<'a>(b: EntityBuilder<'a>, claims: Claims) -> EntityBuilder<'a> {
    b.kinematic()
        .invulnerable()
        .unpushable()
        .no_egg()
        .attach(&OWNER)
        .on_tick(move |t| leave_unclaimed(t, claims))
}

fn leave_unclaimed(t: &EntityTick, claims: Claims) {
    let w = t.world();
    for &id in t.ids() {
        let owner = w.component(id, &OWNER).unwrap_or(0);
        if owner == 0 || w.entity(owner).is_none() || !claims(&w, owner, id) {
            w.despawn(id);
        }
    }
}

/// Moves `owner`'s entity `id` to `pos`, turned `yaw` degrees; makes it if it is gone or of
/// another kind. Returns its id.
pub fn place(w: &World, owner: u64, id: u64, kind: &EntityKind, pos: Vec3, yaw: f32) -> u64 {
    let current = w.entity(id).filter(|_| id != 0);
    let Some(st) = current.filter(|st| st.kind == kind.id()) else {
        if current.is_some() && w.component(id, &OWNER) == Some(owner) {
            w.despawn(id);
        }
        return spawn(w, owner, kind, pos, yaw);
    };
    // A kinematic entity moves by its velocity after this tick's systems: just that speed lands
    // it on `pos`, and it stands still once there. Placing it as well would overshoot.
    let vel = (pos - st.pos) * 20.0;
    let yaw = yaw.to_radians();
    if vel != st.vel || yaw != st.yaw {
        w.entity_edit(id).velocity(vel).yaw(yaw);
    }
    id
}

fn spawn(w: &World, owner: u64, kind: &EntityKind, pos: Vec3, yaw: f32) -> u64 {
    let id = w.spawn_with(kind, pos, &Spawn::new().yaw(yaw.to_radians()));
    w.set_component(id, &OWNER, owner);
    id
}
