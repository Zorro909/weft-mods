//! Entities that only show what a block entity is doing (an excavator's gantry, an engine's
//! piston). Each belongs to an owner, moves where its owner puts it, and leaves when the owner
//! is gone.

use weft_sdk::*;

/// The block entity an owned entity belongs to.
static OWNER: Component<u64> = Component::named("cogwork:owner");

pub fn register(r: &mut Registry) {
    r.entity_component(&OWNER, 0u64, component_flags::SAVE);
}

/// Makes `b` an owned entity: moved only by its owner, never hurt or pushed, with no egg.
pub fn kind<'a>(b: EntityBuilder<'a>) -> EntityBuilder<'a> {
    b.kinematic()
        .invulnerable()
        .unpushable()
        .no_egg()
        .attach(&OWNER)
        .on_tick(leave_without_owner)
}

fn leave_without_owner(t: &EntityTick) {
    let w = t.world();
    for &id in t.ids() {
        let owner = w.component(id, &OWNER).copied().unwrap_or(0);
        if owner == 0 || w.entity(owner).is_none() {
            w.remove(id);
        }
    }
}

/// Moves `owner`'s entity `id` to `pos`, turned `yaw` degrees; makes it if it is gone or of
/// another kind. Returns its id.
pub fn place(w: &World, owner: u64, id: u64, kind: &Entity, pos: [f64; 3], yaw: f32) -> u64 {
    let current = w.entity(id).filter(|_| id != 0);
    let Some(mut st) = current.filter(|st| st.kind == kind.id()) else {
        if current.is_some() && w.component(id, &OWNER) == Some(&owner) {
            w.remove(id);
        }
        return spawn(w, owner, kind, pos, yaw);
    };
    // A kinematic entity moves by its velocity after this tick's systems: just that speed lands
    // it on `pos`, and it stands still once there. Placing it as well would overshoot.
    let vel: [f64; 3] = std::array::from_fn(|i| (pos[i] - st.pos[i]) * 20.0);
    let yaw = yaw.to_radians();
    if vel != st.vel || yaw != st.yaw {
        st.vel = vel;
        st.yaw = yaw;
        w.set_entity(id, &st);
    }
    id
}

fn spawn(w: &World, owner: u64, kind: &Entity, pos: [f64; 3], yaw: f32) -> u64 {
    let id = w.spawn(kind, pos);
    if let Some(o) = w.component_mut(id, &OWNER) {
        *o = owner;
    }
    if let Some(mut st) = w.entity(id) {
        st.yaw = yaw.to_radians();
        w.set_entity(id, &st);
    }
    id
}
