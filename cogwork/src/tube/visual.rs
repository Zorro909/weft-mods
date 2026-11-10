//! What players see of items in tubes: an ordinary dropped item of the same kind, held in place
//! inside the tube every tick, which nobody may pick up. It is only a picture: the real stack
//! stays in the tube's slot, so a lost picture is drawn again and costs no items.
//!
//! Pictures are saved with the world like any dropped item, and their tube keeps their id, so
//! after loading each tube takes its pictures back. One no tube has shown for a while (its tube
//! was taken away by a command) is removed. Entity ids start over in every world, so whether an
//! item is a picture is asked of the entity (`IS_PICTURE`), never of the id alone.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

use weft_sdk::*;

use super::motion::HUB;
use super::{Motion, TUBES};
use crate::lib::{BlockState, Pos, TypedWorld, point};

/// Marks a picture's stack, a different value for neighbors, so two pictures of the same item
/// never merge into one the way dropped items do.
static PICTURE: Component<u32> = Component::named("cogwork:tube_picture");
/// 1 on the dropped items that are pictures, saved with them.
static IS_PICTURE: Component<u32> = Component::named("cogwork:is_picture");
/// The pictures shown lately, by dimension and entity: the tick a tube last showed each.
type Pictures = HashMap<(u32, u64), u64>;
/// It outlives the world, so an entry may name an entity of another world.
static PICTURES: LazyLock<Mutex<Pictures>> = LazyLock::new(Default::default);
static NEXT_MARK: AtomicU32 = AtomicU32::new(0);
/// Ticks a picture may go unshown before it is taken away.
const STALE: u64 = 40;
/// Blocks a dropped item is drawn above where it is, bobbing: a block 0.15 up, a flat item from
/// 0.15 to 0.65 up.
const BLOCK_BOB: f64 = 0.15;
const ITEM_BOB: f64 = 0.4;

pub fn register(r: &mut Registry) {
    r.stack_component(&PICTURE, 0);
    r.entity_component(&IS_PICTURE, 0, component_flags::SAVE);
    r.edit(&ITEM).attach(&IS_PICTURE);
    // A picture's pickup delay (`spawn`) isn't saved: this keeps loaded ones out of hands too.
    r.before_event(&actions::PICKUP_ITEM, |pickup| {
        if is_picture(&pickup.world(), pickup.query().other) {
            Verdict::Deny
        } else {
            Verdict::Abstain
        }
    });
    r.world_tick(sweep);
}

fn pictures() -> MutexGuard<'static, Pictures> {
    PICTURES.lock().unwrap_or_else(PoisonError::into_inner)
}

fn is_picture(world: &World, id: u64) -> bool {
    world
        .component(id, &IS_PICTURE)
        .is_some_and(|mark| mark != 0)
}

/// Whether a picture last shown at the tick `shown` is left over at `now`: unshown for `STALE`
/// ticks, or shown at a time this world hasn't reached, so in another world.
fn is_stale(shown: u64, now: u64) -> bool {
    shown + STALE < now || shown > now
}

/// Removes the pictures of this region nobody has shown lately.
fn sweep(world: &World) {
    let now = world.game_time();
    if !now.is_multiple_of(STALE) {
        return;
    }
    let dimension = world.dimension();
    // Collected first, so the lock is let go before `remove` takes it.
    let stale: Vec<u64> = pictures()
        .iter()
        .filter(|&(&(of, _), &shown)| of == dimension && is_stale(shown, now))
        .map(|(&(_, id), _)| id)
        .collect();
    // Only what this region holds; another region sweeps its own.
    for id in stale.into_iter().filter(|&id| world.entity(id).is_some()) {
        remove(world, id);
    }
}

/// Where in the world an item at `motion` in the tube at `pos` is, and how fast it goes there
/// (blocks per second).
fn place_of(pos: Pos, motion: &Motion) -> (Vec3, Vec3) {
    let (side, along, sign) = if motion.reached_hub() {
        (motion.leaves_by(), motion.pos - HUB, 1.0)
    } else {
        (motion.came_by(), HUB - motion.pos, -1.0)
    };
    let Some(dir) = side else {
        return (pos.center(), Vec3::ZERO);
    };
    let at = point::toward(pos.center(), dir, along as f64);
    // Ticks are a twentieth of a second.
    let velocity = point::toward(Vec3::ZERO, dir, sign * motion.speed as f64 * 20.0);
    (at, velocity)
}

/// Shows the item at `motion` where it is now, making its picture if it has none.
pub fn show(world: &World, pos: Pos, motion: &mut Motion, stack: ItemStack) {
    let (at, velocity) = place_of(pos, motion);
    let bob = if BlockState::placed_by(stack.item).is_some() {
        BLOCK_BOB
    } else {
        ITEM_BOB
    };
    let at = at.offset(0.0, -bob, 0.0);
    let shown = world
        .entity(motion.visual)
        .is_some_and(|entity| motion.visual != 0 && entity.kind == ITEM.id());
    if shown {
        world
            .entity_modify(motion.visual)
            .pos(at)
            .velocity(velocity);
    } else {
        motion.visual = spawn(world, at, velocity, stack);
    }
    if motion.visual != 0 {
        // Again on every show, for pictures saved before they were marked.
        world.set_component(motion.visual, &IS_PICTURE, 1);
        pictures().insert((world.dimension(), motion.visual), world.game_time());
    }
}

/// A new picture of `stack` at `at` going at `velocity`, 0 if none could be made.
fn spawn(world: &World, at: Vec3, velocity: Vec3, stack: ItemStack) -> u64 {
    let mark = NEXT_MARK.fetch_add(1, Ordering::Relaxed) % 64;
    let picture = ItemStack { count: 1, ..stack }.with(&PICTURE, mark);
    let how = Spawn::new()
        .stack(picture)
        .velocity(velocity)
        .pickup_delay(u32::MAX);
    world.spawn_with(&ITEM, at, &how)
}

/// Takes the picture `id` out of the world.
pub fn remove(world: &World, id: u64) {
    if id != 0 && is_picture(world, id) {
        world.despawn(id);
    }
    pictures().remove(&(world.dimension(), id));
}

/// Removes the pictures left where a tube was broken; the spilled stacks stay.
pub fn clear_broken(world: &World, pos: Pos) {
    if world.block_typed(pos).has(&TUBES) {
        return;
    }
    for id in world.entities_near(pos.center(), 1.5, Some(&ITEM)) {
        // Back up by the bob `show` takes off, to find the cell it was drawn for.
        let inside = world
            .entity(id)
            .is_some_and(|entity| Pos::from(entity.pos.offset(0.0, ITEM_BOB, 0.0).block()) == pos);
        if inside {
            remove(world, id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tube::motion::NONE;

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
    fn a_picture_unshown_or_from_another_world_is_stale() {
        assert!(!is_stale(100, 100 + STALE), "shown lately");
        assert!(is_stale(100, 101 + STALE), "unshown too long");
        assert!(is_stale(5000, 20), "shown in a world further on in time");
    }

    #[test]
    fn an_item_is_drawn_on_its_way_through_the_tube() {
        let (west, up) = (Dir::West.face(), Dir::Up.face());
        let tube = Pos::new(0, 0, 0);
        let (hub, still) = place_of(tube, &at(0.5, NONE, NONE));
        let middle = Vec3::new(0.5, 0.5, 0.5);
        assert_eq!((hub, still), (middle, Vec3::ZERO), "waiting at the hub");
        let (start, velocity) = place_of(tube, &at(0.0, west, NONE));
        assert_eq!(start, Vec3::new(0.0, 0.5, 0.5), "in at the west face");
        assert!(velocity.x > 0.0, "heading east, to the hub");
        let (end, velocity) = place_of(tube, &at(1.0, west, up));
        assert_eq!(end, Vec3::new(0.5, 1.0, 0.5), "out at the top");
        assert!(
            (velocity.y - 1.0).abs() < 1e-6,
            "a block a second, going up"
        );
    }
}
