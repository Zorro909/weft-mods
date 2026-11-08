//! Material slots: a structure names a role (`roof/stair`, `smith/furnace`) with a default block
//! in place of a block, and `Materials` swaps the block of a role when it is placed.
//!
//! In a palette, `@roof/stair[facing=north,half=bottom]` is slot `roof/stair` with those
//! properties. They go onto whichever block fills the slot, so the same roof is built of brick
//! stairs as well as of spruce ones; a property that block lacks is left out.

use std::collections::BTreeMap;

use weft_sdk::{Block, error, state_with};

/// What marks a palette entry as a slot.
const SLOT: char = '@';

/// The blocks some slots are filled with instead of their defaults, by slot name. Equal sets
/// compare equal however they were built.
///
/// ```ignore
/// let brick = Materials::new().with("roof/stair", "base:bricks_stairs");
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Materials {
    /// Sorted by slot, each slot once.
    blocks: Vec<(String, String)>,
}

impl Materials {
    /// Every slot filled with its default.
    pub fn new() -> Materials {
        Materials::default()
    }

    /// Fills `slot` with `block` (`base:bricks_stairs`, or a state such as
    /// `base:furnace[lit=true]`), in place of what it was filled with.
    pub fn with(mut self, slot: &str, block: &str) -> Materials {
        match self.blocks.binary_search_by(|(s, _)| s.as_str().cmp(slot)) {
            Ok(i) => self.blocks[i].1 = block.to_string(),
            Err(i) => self.blocks.insert(i, (slot.to_string(), block.to_string())),
        }
        self
    }

    /// The block `slot` is filled with, `None` for its default.
    pub fn get(&self, slot: &str) -> Option<&str> {
        let i = self.blocks.binary_search_by(|(s, _)| s.as_str().cmp(slot));
        i.ok().map(|i| self.blocks[i].1.as_str())
    }
}

/// A palette entry that is a slot: its name and its properties (`facing=north,half=bottom`,
/// empty for none). `None` for a plain block state.
pub(crate) fn slot_of(entry: &str) -> Option<(&str, &str)> {
    let rest = entry.strip_prefix(SLOT)?;
    match rest.split_once('[') {
        Some((slot, props)) => Some((slot, props.strip_suffix(']')?)),
        None => Some((rest, "")),
    }
}

/// Whether `name` may name a slot: lowercase letters, digits, `_` and `/`.
pub(crate) fn valid_slot(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '/')
}

/// `block` with `props` set on top of its own: `base:furnace[lit=false]` and `facing=north`
/// make `base:furnace[lit=false,facing=north]`. A property both name takes the value of `props`.
pub(crate) fn with_props(block: &str, props: &str) -> String {
    let (name, own) = match block.split_once('[') {
        Some((name, own)) => (name, own.strip_suffix(']').unwrap_or(own)),
        None => (block, ""),
    };
    let key = |prop: &str| prop.split('=').next().unwrap_or("").to_string();
    let added: Vec<String> = props
        .split(',')
        .filter(|p| !p.is_empty())
        .map(key)
        .collect();
    let kept = own
        .split(',')
        .filter(|p| !p.is_empty() && !added.contains(&key(p)));
    let all: Vec<&str> = kept
        .chain(props.split(',').filter(|p| !p.is_empty()))
        .collect();
    if all.is_empty() {
        name.to_string()
    } else {
        format!("{name}[{}]", all.join(","))
    }
}

/// Each palette entry of a structure as the state name it is placed as with the slots'
/// defaults: the names the game checks when it loads.
pub(crate) fn default_names(palette: &[String], slots: &BTreeMap<String, String>) -> Vec<String> {
    palette
        .iter()
        .map(|entry| match slot_of(entry) {
            Some((slot, props)) => with_props(&slots[slot], props),
            None => entry.clone(),
        })
        .collect()
}

/// The state of each palette entry with `materials`, `None` if a block is unknown. A block in
/// `materials` the game doesn't have is said and the slot's default used instead.
pub(crate) fn states(
    what: &str,
    palette: &[String],
    slots: &BTreeMap<String, String>,
    materials: &Materials,
) -> Option<Vec<u32>> {
    palette
        .iter()
        .map(|entry| match slot_of(entry) {
            None => Block::lookup(entry),
            Some((slot, props)) => {
                let chosen = materials.get(slot).and_then(|block| {
                    let state = Block::lookup(block);
                    if state.is_none() {
                        error!("{what}: slot {slot} is to be {block}, no block of this game");
                    }
                    state
                });
                let state = chosen.or_else(|| Block::lookup(&slots[slot]))?;
                Some(
                    props
                        .split(',')
                        .filter_map(|p| p.split_once('='))
                        .fold(state, |state, (name, value)| {
                            state_with(state, name, value).unwrap_or(state)
                        }),
                )
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materials_keep_the_last_block_per_slot() {
        let materials = Materials::new()
            .with("roof/stair", "base:oak_planks_stairs")
            .with("wall", "base:bricks")
            .with("roof/stair", "base:bricks_stairs");
        assert_eq!(
            materials.get("roof/stair"),
            Some("base:bricks_stairs"),
            "replaced"
        );
        assert_eq!(materials.get("roof/slab"), None, "the default");
        let other = Materials::new()
            .with("wall", "base:bricks")
            .with("roof/stair", "base:bricks_stairs");
        assert_eq!(materials, other, "built in another order");
    }

    #[test]
    fn slots_are_read_from_the_palette() {
        assert_eq!(
            slot_of("@roof/stair[facing=north,half=top]"),
            Some(("roof/stair", "facing=north,half=top")),
            "with properties"
        );
        assert_eq!(slot_of("@wall"), Some(("wall", "")), "without");
        assert_eq!(slot_of("base:stone"), None, "a plain block");
        assert!(valid_slot("smith/furnace_2"), "a good name");
        assert!(!valid_slot("Roof") && !valid_slot(""), "bad names");
    }

    #[test]
    fn slot_properties_go_on_top_of_the_blocks_own() {
        assert_eq!(
            with_props("base:furnace[lit=false,facing=east]", "facing=north"),
            "base:furnace[lit=false,facing=north]",
            "facing replaced, lit kept"
        );
        assert_eq!(with_props("base:stone", ""), "base:stone", "nothing to add");
        assert_eq!(
            with_props("base:oak_planks_stairs", "half=top"),
            "base:oak_planks_stairs[half=top]",
            "added"
        );
    }

    #[test]
    fn defaults_fill_the_slots() {
        let palette = ["@roof[half=top]".to_string(), "base:glass".to_string()];
        let slots = BTreeMap::from([("roof".to_string(), "base:spruce_planks_stairs".to_string())]);
        assert_eq!(
            default_names(&palette, &slots),
            ["base:spruce_planks_stairs[half=top]", "base:glass"],
            "the slot's default with its properties"
        );
    }
}
