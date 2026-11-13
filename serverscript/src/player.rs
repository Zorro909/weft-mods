//! Who a player is from one visit to the next: a number of the mod's own on the player, which
//! the store keeps their data under.
//!
//! The game gives a player a new entity id at every join and hands mods no name or key, so the
//! number is a component saved with the player. It is the only thing the mod keeps in the
//! world's own files: the data itself is the store's.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};

use weft_sdk::{Component, PLAYER, Registry, World, component_flags};

/// A player's number in the store. Saved by this name, so the name stays.
static PLAYER_ID: Component<u64> = Component::named("serverscript:player_id");

/// What a player who was given no number yet carries.
const NONE: u64 = 0;

pub fn register(r: &mut Registry) {
    let kept = component_flags::SAVE | component_flags::KEEP_ON_RESPAWN;
    r.entity_component(&PLAYER_ID, NONE, kept);
    r.edit(&PLAYER).attach(&PLAYER_ID);
}

/// The number `entity` has; `None` for a player without one, for what is no player, and for a
/// player in another region than the script runs in.
pub fn known(world: &World, entity: u64) -> Option<u64> {
    world.component(entity, &PLAYER_ID).filter(|id| *id != NONE)
}

/// The number of `entity`, given now if it had none; `None` for what is no player here.
pub fn id(world: &World, entity: u64) -> Option<u64> {
    let id = world.component(entity, &PLAYER_ID)?;
    if id != NONE {
        return Some(id);
    }
    let id = fresh();
    world.set_component(entity, &PLAYER_ID, id).then_some(id)
}

/// A number no player has yet: random, so that two worlds sharing one scripts folder don't
/// hand out the same.
fn fresh() -> u64 {
    RandomState::new().build_hasher().finish().max(NONE + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_number_is_never_the_one_for_none() {
        let numbers: Vec<u64> = (0..64).map(|_| fresh()).collect();
        assert!(numbers.iter().all(|number| *number != NONE));
        let mut different = numbers.clone();
        different.sort_unstable();
        different.dedup();
        assert_eq!(different.len(), numbers.len(), "no two are the same");
    }
}
