//! What players see of items in tubes: an ordinary dropped item of the same kind, held in place
//! inside the tube every tick, which nobody may pick up. It is only a picture: the real stack
//! stays in the tube's slot, so a lost picture is drawn again and costs no items.
//!
//! Pictures are saved with the world like any dropped item, and their tube keeps their id, so
//! after loading each tube takes its pictures back. One no tube has shown for a while (its tube
//! was taken away by a command) is removed.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use weft_sdk::*;

use super::{Motion, NONE, TUBES};

static ITEM_ENTITY: Entity = Entity::named("core:item");
/// Marks a picture's stack, a different value for neighbors, so two pictures of the same item
/// never merge into one the way dropped items do.
static PICTURE: Component<u32> = Component::named("cogwork:tube_picture");
/// The pictures in the world, by dimension and entity: the tick a tube last showed each.
static PICTURES: Mutex<Option<HashMap<(u32, u64), u64>>> = Mutex::new(None);
static NEXT_MARK: AtomicU32 = AtomicU32::new(0);
/// Ticks a picture may go unshown before it is taken away.
const STALE: u64 = 40;

pub fn register(r: &mut Registry) {
    r.stack_component(&PICTURE, 0);
    r.intercept(&question::PICKUP_ITEM, |q| {
        if is_picture(q.world().dimension(), q.query().other) {
            Verdict::Deny
        } else {
            Verdict::Abstain
        }
    });
    r.world_tick(sweep);
}

fn pictures<R>(f: impl FnOnce(&mut HashMap<(u32, u64), u64>) -> R) -> R {
    let mut guard = PICTURES.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(HashMap::new))
}

fn is_picture(dim: u32, id: u64) -> bool {
    pictures(|p| p.contains_key(&(dim, id)))
}

/// Removes the pictures of this region nobody has shown lately.
fn sweep(w: &World) {
    let now = w.time();
    if !now.is_multiple_of(STALE) {
        return;
    }
    let dim = w.dimension();
    let stale: Vec<u64> = pictures(|p| {
        p.iter()
            .filter(|((d, _), shown)| *d == dim && **shown + STALE < now)
            .map(|((_, id), _)| *id)
            .collect()
    });
    for id in stale {
        // Only what this region holds; another region sweeps its own.
        if w.entity(id).is_some() {
            w.remove(id);
            pictures(|p| p.remove(&(dim, id)));
        }
    }
}

/// Where in the world an item at `m` in the tube at `pos` is, and how fast it goes there
/// (blocks per second).
fn place_of(pos: [i32; 3], m: &Motion) -> ([f64; 3], [f64; 3]) {
    let (face, along, sign) = if m.pos < 0.5 {
        (m.from, 0.5 - m.pos, -1.0)
    } else {
        (m.to, m.pos - 0.5, 1.0)
    };
    let mut at = pos.map(|v| v as f64 + 0.5);
    let mut vel = [0.0; 3];
    if face != NONE {
        let o = Dir::from_face(face).offset();
        for i in 0..3 {
            at[i] += o[i] as f64 * along as f64;
            vel[i] = o[i] as f64 * sign * m.speed as f64 * 20.0;
        }
    }
    (at, vel)
}

/// Shows the item at `m` where it is now, making its picture if it has none.
pub fn show(w: &World, pos: [i32; 3], m: &mut Motion, stack: ItemStack) {
    let (mut at, vel) = place_of(pos, m);
    // A dropped item is drawn bobbing above where it is: a block 0.15 up, a flat item from
    // 0.15 to 0.65 up.
    at[1] -= if item_block(stack.item).is_some() {
        0.15
    } else {
        0.4
    };
    let mut state = w
        .entity(m.visual)
        .filter(|st| m.visual != 0 && st.kind == ITEM_ENTITY.id());
    if state.is_none() {
        m.visual = spawn(w, at, stack);
        state = w.entity(m.visual);
    }
    let Some(mut st) = state else { return };
    pictures(|p| p.insert((w.dimension(), m.visual), w.time()));
    st.pos = at;
    st.vel = vel;
    w.set_entity(m.visual, &st);
}

/// A new picture of `stack` at `at`, or 0 if it couldn't be found after spawning.
fn spawn(w: &World, at: [f64; 3], stack: ItemStack) -> u64 {
    let mark = NEXT_MARK.fetch_add(1, Ordering::Relaxed) % 64;
    let before = w.near(at, 0.5, Some(&ITEM_ENTITY));
    w.spawn_item(at, ItemStack { count: 1, ..stack }.with(&PICTURE, mark));
    let id = w
        .near(at, 0.5, Some(&ITEM_ENTITY))
        .into_iter()
        .find(|id| !before.contains(id))
        .unwrap_or(0);
    if id != 0 {
        pictures(|p| p.insert((w.dimension(), id), w.time()));
    }
    id
}

/// Takes the picture `id` out of the world.
pub fn remove(w: &World, id: u64) {
    if id != 0 && is_picture(w.dimension(), id) {
        w.remove(id);
        pictures(|p| p.remove(&(w.dimension(), id)));
    }
}

/// Removes the pictures left where a tube was broken; the spilled stacks stay.
pub fn clear_broken(w: &World, pos: [i32; 3]) {
    if TUBES.on_block(w.get(pos)) {
        return;
    }
    let center = pos.map(|v| v as f64 + 0.5);
    for id in w.near(center, 1.5, Some(&ITEM_ENTITY)) {
        // Back up by the bob `show` takes off, to find the cell it was drawn for.
        let inside = w.entity(id).is_some_and(|st| {
            let cell = [st.pos[0], st.pos[1] + 0.4, st.pos[2]].map(|v| v.floor() as i32);
            cell == pos
        });
        if inside {
            remove(w, id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(pos: f32, from: u32, to: u32) -> Motion {
        Motion {
            item: 1,
            pos,
            speed: 0.05,
            from,
            to,
            ..Motion::default()
        }
    }

    #[test]
    fn an_item_is_drawn_on_its_way_through_the_tube() {
        let (west, up) = (Dir::West.face(), Dir::Up.face());
        let (hub, still) = place_of([0, 0, 0], &at(0.5, NONE, NONE));
        assert_eq!((hub, still), ([0.5; 3], [0.0; 3]), "waiting at the hub");
        let (start, vel) = place_of([0, 0, 0], &at(0.0, west, NONE));
        assert_eq!(start, [0.0, 0.5, 0.5], "in at the west face");
        assert!(vel[0] > 0.0, "heading east, to the hub");
        let (end, vel) = place_of([0, 0, 0], &at(1.0, west, up));
        assert_eq!(end, [0.5, 1.0, 0.5], "out at the top");
        assert!((vel[1] - 1.0).abs() < 1e-6, "a block a second, going up");
    }
}
