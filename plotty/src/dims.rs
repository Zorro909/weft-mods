//! Dimensions as Plotty keeps them: by `dimension_key`, a hash of the name that stays when other
//! mods come or go, and back to a handle `World::teleport` takes.

use std::sync::Mutex;

use weft_sdk::{Dimension, World, abi, dimension_from_key, dimension_key, name_of};

/// Handles made so far, so each dimension's is made once: `Dimension::generated` keeps them.
static HANDLES: Mutex<Vec<(u64, &'static Dimension)>> = Mutex::new(Vec::new());

/// The key of the dimension `world` is in.
pub fn key_of(world: &World) -> u64 {
    dimension_key(world.dimension())
}

/// The dimension a key means, while a mod still registers it.
pub fn handle(key: u64) -> Option<&'static Dimension> {
    let mut handles = HANDLES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((_, handle)) = handles.iter().find(|(known, _)| *known == key) {
        return Some(handle);
    }
    let id = dimension_from_key(key)?;
    let handle = Dimension::generated(name_of(abi::entry_kind::DIMENSION, id).to_string());
    handles.push((key, handle));
    Some(handle)
}
