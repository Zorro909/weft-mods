//! How deep each oil field lies, noted as its plan lays it out. The engine says where a field
//! starts (`World::structure_near`) but not the random bits that set its depth, so the plan
//! notes its ball here, by its world's seed, its dimension and its start. A field the engine
//! finds has had its plan run in this process: finding it runs the plan, unless the engine laid
//! it out before, which ran it then.
//!
//! A `static` that outlives the world it was filled in, so it is keyed by the seed: another world
//! never reads a field of this one.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use crate::lib::Pos;

/// Where a field's oil lies: its highest and its lowest layer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Depth {
    pub top: i32,
    pub bottom: i32,
}

/// A field's start in a world: the seed, the dimension and the start.
type Key = (u64, u32, Pos);

static KNOWN: Mutex<Option<HashMap<Key, Depth>>> = Mutex::new(None);

/// The fields noted, even after a panic while it was held.
fn lock() -> MutexGuard<'static, Option<HashMap<Key, Depth>>> {
    KNOWN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Notes the ball of oil of `radius` round `middle` of the field that starts at `start`, in the
/// world of `seed` and its dimension `dimension`.
pub fn note(seed: u64, dimension: u32, start: Pos, middle: Pos, radius: i32) {
    let depth = Depth {
        top: middle.y + radius,
        bottom: middle.y - radius,
    };
    let mut known = lock();
    known
        .get_or_insert_with(HashMap::new)
        .insert((seed, dimension, start), depth);
}

/// Where the oil of the field that starts at `start` lies, if its plan ran in this process.
pub fn depth(seed: u64, dimension: u32, start: Pos) -> Option<Depth> {
    let known = lock();
    known.as_ref()?.get(&(seed, dimension, start)).copied()
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
}
