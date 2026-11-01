//! Finding structures again: their plans are worked out anew from the seed, so this finds them
//! whether their cubes are generated or not.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use weft_sdk::*;

use crate::fit::Ground;
use crate::generator::Generator;
use crate::region;

/// Every structure this mod registered, in order.
static STRUCTURES: RwLock<Vec<Arc<Generator>>> = RwLock::new(Vec::new());
/// The seed each dimension generates with, learned at its first cube: the SDK hands mods the
/// seed only while generating.
static SEEDS: RwLock<Option<HashMap<u32, u64>>> = RwLock::new(None);

pub(crate) fn add(generator: Arc<Generator>) {
    STRUCTURES
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .push(generator);
}

pub(crate) fn remember_seed(dimension: u32, seed: u64) {
    if seed_of(dimension) == Some(seed) {
        return;
    }
    let mut seeds = SEEDS.write().unwrap_or_else(|e| e.into_inner());
    seeds
        .get_or_insert_with(HashMap::new)
        .insert(dimension, seed);
}

fn seed_of(dimension: u32) -> Option<u64> {
    SEEDS.read().ok()?.as_ref()?.get(&dimension).copied()
}

/// The full names of the structures this mod registered with `place` and `plan`.
pub fn names() -> Vec<String> {
    let structures = STRUCTURES.read().unwrap_or_else(|e| e.into_inner());
    structures.iter().map(|g| g.name.clone()).collect()
}

/// The nearest structure `name` (full name) within `radius` blocks of `from` in `world`'s
/// dimension, seen from above: the lowest corner of the first thing its plan placed. Its cubes
/// needn't be generated yet. `None` also for a name this mod didn't register, a dimension the
/// structure isn't in, or a dimension that hasn't generated a cube yet.
pub fn locate(world: &World, name: &str, from: [i32; 3], radius: i32) -> Option<[i32; 3]> {
    let generator = find(name)?;
    let dimension_id = world.dimension();
    let allowed = generator.dimensions.is_empty()
        || generator.dimensions.iter().any(|d| d.id() == dimension_id);
    if !allowed {
        return None;
    }
    let seed = seed_of(dimension_id)?;
    let dimension = Dimension::named(name_of(abi::entry_kind::DIMENSION, dimension_id));
    let ground = |x, z| ground(&dimension, seed, x, z);
    nearest(&generator, seed, dimension_id, from, radius, &ground)
}

fn find(name: &str) -> Option<Arc<Generator>> {
    let structures = STRUCTURES.read().ok()?;
    structures.iter().find(|g| g.name == name).cloned()
}

/// Goes through the regions nearest first, and stops once a region is further away than the
/// best found.
fn nearest(
    generator: &Generator,
    seed: u64,
    dimension: u32,
    from: [i32; 3],
    radius: i32,
    ground: &dyn Fn(i32, i32) -> Option<Ground>,
) -> Option<[i32; 3]> {
    let [x, _, z] = from;
    let mut best: Option<([i32; 3], i64)> = None;
    for (index, distance) in region::around(x, z, radius, generator.region) {
        if best.is_some_and(|(_, found)| distance > found) {
            break;
        }
        let slot = generator.planned(seed, dimension, index, ground);
        let Some(anchor) = slot.get().and_then(Option::as_ref).and_then(|p| p.anchor()) else {
            continue;
        };
        let (dx, dz) = ((anchor[0] - x) as i64, (anchor[2] - z) as i64);
        let distance = dx * dx + dz * dz;
        let closer = best.is_none_or(|(_, found)| distance < found);
        if closer && distance <= radius as i64 * radius as i64 {
            best = Some((anchor, distance));
        }
    }
    best.map(|(anchor, _)| anchor)
}

fn ground(dimension: &Dimension, seed: u64, x: i32, z: i32) -> Option<Ground> {
    let column = dimension.column(seed, x, z);
    (column.height != i32::MIN && !dimension.surface_carved(seed, x, z)).then_some(Ground {
        height: column.height,
        biome: column.biome,
    })
}
