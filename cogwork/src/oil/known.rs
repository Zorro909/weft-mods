//! How deep each oil field lies, noted as its plan lays it out. The engine says where a field
//! starts (`World::structure_near`) but not the random bits that set its depth, so the plan
//! notes its ball here, by its world's seed, its dimension and its start. A field the engine
//! finds has had its plan run in this process: finding it runs the plan, unless the engine laid
//! it out before, which ran it then.
//!
//! A `static` that outlives the world it was filled in, so it is keyed by the seed: another world
//! never reads a field of this one. It keeps the `MOST` fields noted last, so a long-running
//! server or one world after another never fills memory with them; a field noted before those
//! has an unknown depth until its plan runs again.

use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, MutexGuard};

use crate::lib::Pos;

/// Fields it keeps the depth of: under a megabyte.
const MOST: usize = 8_192;

/// Where a field's oil lies: its highest and its lowest layer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Depth {
    pub top: i32,
    pub bottom: i32,
}

/// A field's start in a world: the seed, the dimension and the start.
type Key = (u64, u32, Pos);

/// The depths of the fields noted last, with their keys oldest first.
#[derive(Default)]
struct Known {
    depths: HashMap<Key, Depth>,
    order: VecDeque<Key>,
}

impl Known {
    /// Notes the depth of the field `key`, letting go of the oldest past `MOST`.
    fn note(&mut self, key: Key, depth: Depth) {
        if self.depths.insert(key, depth).is_none() {
            self.order.push_back(key);
        }
        while self.order.len() > MOST {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            self.depths.remove(&oldest);
        }
    }

    fn depth(&self, key: Key) -> Option<Depth> {
        self.depths.get(&key).copied()
    }
}

static KNOWN: Mutex<Option<Known>> = Mutex::new(None);

/// The fields noted, even after a panic while it was held.
fn lock() -> MutexGuard<'static, Option<Known>> {
    KNOWN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Notes the ball of oil of `radius` round `middle` of the field that starts at `start`, in the
/// world of `seed` and its dimension `dimension`.
pub fn note(seed: u64, dimension: u32, start: Pos, middle: Pos, radius: i32) {
    let depth = Depth {
        top: middle.y.saturating_add(radius),
        bottom: middle.y.saturating_sub(radius),
    };
    let mut known = lock();
    known
        .get_or_insert_with(Known::default)
        .note((seed, dimension, start), depth);
}

/// Where the oil of the field that starts at `start` lies, if its plan ran in this process lately.
pub fn depth(seed: u64, dimension: u32, start: Pos) -> Option<Depth> {
    let known = lock();
    known.as_ref()?.depth((seed, dimension, start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_is_known_only_in_its_own_world() {
        let start = Pos::new(4_000_001, 70, -9);
        note(42, 0, start, Pos::new(4_000_001, 10, -9), 5);
        let known = Depth { top: 15, bottom: 5 };
        assert_eq!(
            depth(42, 0, start),
            Some(known),
            "from its top to its bottom"
        );
        assert_eq!(depth(43, 0, start), None, "another seed");
        assert_eq!(depth(42, 1, start), None, "another dimension");
    }

    #[test]
    fn past_the_most_it_keeps_the_oldest_field_goes() {
        let mut known = Known::default();
        let field = |x| (7, 0, Pos::new(x, 70, 0));
        let deep = Depth { top: 9, bottom: 1 };
        for x in 0..=MOST as i32 {
            known.note(field(x), deep);
        }
        assert_eq!(known.depth(field(0)), None, "the first noted went");
        assert_eq!(known.depth(field(1)), Some(deep), "the next stays");
        assert_eq!(known.depth(field(MOST as i32)), Some(deep), "the newest");
        known.note(field(1), deep);
        assert_eq!(known.depths.len(), MOST, "noting one again adds none");
    }
}
