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

/// Moves `owner`'s entity `id` to `pos`, turned `yaw` degrees; makes it anew if it is gone, of
/// another kind or turned another way. Returns its id.
pub fn place(w: &World, owner: u64, id: u64, kind: &EntityKind, pos: Vec3, yaw: f32) -> u64 {
    let current = w.entity(id).filter(|_| id != 0);
    let Some(st) = current.filter(|st| still_fits(st.kind, st.yaw, kind.id(), yaw)) else {
        if current.is_some() && w.component(id, &OWNER) == Some(owner) {
            w.despawn(id);
        }
        return spawn(w, owner, kind, pos, yaw);
    };
    // A kinematic entity moves by its velocity after this tick's systems: just that speed lands
    // it on `pos`, and it stands still once there. Placing it as well would overshoot.
    let vel = (pos - st.pos) * 20.0;
    if vel != st.vel {
        w.entity_modify(id).velocity(vel);
    }
    id
}

/// Whether an entity of `kind` turned `yaw` radians can stay to be of `wanted` turned
/// `wanted_yaw` degrees. A turned one can't: clients turn its body after its yaw like a mob's,
/// which stops 50 degrees short when it stands still.
fn still_fits(kind: u32, yaw: f32, wanted: u32, wanted_yaw: f32) -> bool {
    kind == wanted && yaw == wanted_yaw.to_radians()
}

fn spawn(w: &World, owner: u64, kind: &EntityKind, pos: Vec3, yaw: f32) -> u64 {
    let id = w.spawn_with(kind, pos, &Spawn::new().yaw(yaw.to_radians()));
    w.set_component(id, &OWNER, owner);
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
