//! The templates a mod's plans may place: registered by name when the mod loads, resolved to
//! block states, for every turn, at the first cube that needs them.
//!
//! Every mod links its own copy of this crate, so a plan sees only its own mod's templates.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use weft_sdk::*;

use crate::format::Structure;
use crate::turn::{Turn, turn_state};

/// By full name (`mymod:hut`).
static TEMPLATES: RwLock<Option<HashMap<String, Arc<Entry>>>> = RwLock::new(None);

/// Registers `structure` as `template`: its blocks are checked when the game loads,
/// `world.place(&TEMPLATE, pos)` puts it anywhere, and plans place it with `p.structure(..)`.
///
/// ```ignore
/// static HOUSE: Template = Template::named("house");
/// let house = Structure::decode(include_bytes!("../assets/structures/house.wst")).expect("house.wst");
/// structgen_api::template(r, &HOUSE, house);
/// ```
pub fn template(registry: &mut Registry, template: &'static Template, structure: Structure) {
    let mut builder = registry.template(template);
    for (pos, block) in structure.blocks() {
        builder = builder.fill(block, pos, pos);
    }
    drop(builder);
    add(template.name(), structure);
}

/// The template registered as `name`.
pub(crate) fn find(name: &str) -> Option<Arc<Entry>> {
    TEMPLATES.read().ok()?.as_ref()?.get(name).cloned()
}

pub(crate) fn add(name: &str, structure: Structure) -> Arc<Entry> {
    let entry = Arc::new(Entry {
        name: name.to_string(),
        structure,
        resolved: OnceLock::new(),
    });
    let mut templates = TEMPLATES.write().unwrap_or_else(|e| e.into_inner());
    templates
        .get_or_insert_with(HashMap::new)
        .insert(name.to_string(), entry.clone());
    entry
}

/// One registered template.
pub(crate) struct Entry {
    pub name: String,
    structure: Structure,
    /// Ids exist only once the game has loaded, so this is filled at the first cube.
    resolved: OnceLock<Option<Resolved>>,
}

impl Entry {
    /// Unturned, along x, y and z.
    pub fn size(&self) -> [i32; 3] {
        self.structure.size()
    }

    /// `None` (said once) if a block is unknown; the template check at load should have caught it.
    pub fn resolved(&self) -> Option<&Resolved> {
        self.resolved.get_or_init(|| self.resolve()).as_ref()
    }

    fn resolve(&self) -> Option<Resolved> {
        let palette: Option<Vec<u32>> = self
            .structure
            .palette
            .iter()
            .map(|name| Block::lookup(name))
            .collect();
        let (Some(palette), Some(air)) = (palette, Block::lookup("core:air")) else {
            error!(
                "structure {}: a block in it is unknown, so it is not placed",
                self.name
            );
            return None;
        };
        let palettes = Turn::ALL.map(|turn| {
            palette
                .iter()
                .map(|&state| turn_state(state, turn))
                .collect()
        });
        Some(Resolved {
            size: self.size(),
            indices: self.structure.indices.clone(),
            palettes,
            air,
        })
    }
}

/// A template ready to place: block states, known once the game has loaded.
#[derive(Debug)]
pub(crate) struct Resolved {
    /// Unturned, along x, y and z.
    pub size: [i32; 3],
    /// Every block's palette index, unturned: x fastest, then z, then y.
    pub indices: Vec<u16>,
    /// The state of each palette entry under each turn, in `Turn::ALL` order.
    pub palettes: [Vec<u32>; 4],
    pub air: u32,
}

impl Resolved {
    /// The state at `pos` of the template turned by `turn`, `pos` counted in the turned box.
    pub fn state(&self, turn: Turn, pos: [i32; 3]) -> u32 {
        let [x, y, z] = turn.inverse().pos(pos, turn.size(self.size));
        let [sx, _, sz] = self.size;
        let index = self.indices[((y * sz + z) * sx + x) as usize];
        self.palettes[turn as usize][index as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 2 x 1 x 3, each block its own state: 10 + its index.
    fn resolved() -> Resolved {
        let palette: Vec<u32> = (10..16).collect();
        Resolved {
            size: [2, 1, 3],
            indices: (0..6).collect(),
            palettes: [0, 1, 2, 3].map(|_| palette.clone()),
            air: 0,
        }
    }

    #[test]
    fn unturned_blocks_read_in_order() {
        let resolved = resolved();
        assert_eq!(resolved.state(Turn::None, [1, 0, 0]), 11, "x first");
        assert_eq!(resolved.state(Turn::None, [0, 0, 2]), 14, "then z");
    }

    #[test]
    fn turned_blocks_come_from_the_turned_place() {
        let resolved = resolved();
        // Turned right the box is 3 x 1 x 2, and the unturned north-west corner (index 0) is
        // now the north-east one.
        assert_eq!(
            resolved.state(Turn::Right, [2, 0, 0]),
            10,
            "north-west went east"
        );
        assert_eq!(
            resolved.state(Turn::Right, [0, 0, 1]),
            15,
            "the far corner came round"
        );
    }

    #[test]
    fn templates_are_found_by_name() {
        let structure = Structure::from_names([1, 1, 1], ["core:air"]).expect("one name");
        add("test:found", structure);
        let entry = find("test:found").expect("registered");
        assert_eq!(entry.size(), [1, 1, 1], "its size");
        assert!(find("test:missing").is_none(), "an unknown name");
    }
}
