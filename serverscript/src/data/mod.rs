//! Data scripts keep on an entity: a component saved with it, holding values by key. Players
//! carry it, and keep it through death.
//!
//! Scripts can't declare components of their own: a server's clients must have registered the
//! same ones to join, and they don't have its scripts. So this one component holds them all.

mod pack;

use weft_sdk::{Component, PLAYER, Registry, World, component_flags, value};

pub use self::pack::Entries;
use crate::value::Value;

/// Bytes an entity's entries may take, packed as `pack` says.
pub const DATA_BYTES: usize = 2040;

/// The layout `ScriptData::bytes` is written in. Another number reads as no data.
const VERSION: u32 = 1;

value! {
    /// An entity's script data. Saved as its bytes, so the layout is a file format: new fields
    /// go last.
    pub struct ScriptData {
        pub version: u32,
        /// How many of `bytes` hold entries.
        pub used: u32,
        pub bytes: [u8; DATA_BYTES],
    }
}

pub static DATA: Component<ScriptData> = Component::named("serverscript:data");

impl ScriptData {
    const EMPTY: ScriptData = ScriptData {
        version: VERSION,
        used: 0,
        bytes: [0; DATA_BYTES],
    };

    /// What it holds; nothing if another version wrote it or the bytes are not entries.
    pub fn entries(&self) -> Entries {
        if self.version != VERSION {
            return Entries::new();
        }
        let used = self.bytes.get(..self.used as usize);
        used.and_then(pack::unpack).unwrap_or_default()
    }

    /// `entries` as the component's value; `None` when they need more than `DATA_BYTES`.
    pub fn holding(entries: &Entries) -> Option<ScriptData> {
        let packed = pack::pack(entries)?;
        let mut data = ScriptData::EMPTY;
        data.bytes.get_mut(..packed.len())?.copy_from_slice(&packed);
        data.used = packed.len() as u32;
        Some(data)
    }
}

pub fn register(r: &mut Registry) {
    let kept = component_flags::SAVE | component_flags::KEEP_ON_RESPAWN;
    r.entity_component(&DATA, ScriptData::EMPTY, kept);
    r.edit(&PLAYER).attach(&DATA);
}

/// The value `entity` has under `key`; `None` also for an entity that carries no data, or one
/// in another region than the script runs in.
pub fn get(world: &World, entity: u64, key: &str) -> Option<Value> {
    world.component(entity, &DATA)?.entries().remove(key)
}

/// Sets `key` on `entity`; false if it carries no data or has no room left for it.
pub fn set(world: &World, entity: u64, key: &str, value: Value) -> bool {
    change(world, entity, |entries| {
        entries.insert(key.to_string(), value);
    })
}

/// Takes `key` off `entity`; false if it carries no data.
pub fn remove(world: &World, entity: u64, key: &str) -> bool {
    change(world, entity, |entries| {
        entries.remove(key);
    })
}

fn change(world: &World, entity: u64, edit: impl FnOnce(&mut Entries)) -> bool {
    let Some(data) = world.component(entity, &DATA) else {
        return false;
    };
    let mut entries = data.entries();
    edit(&mut entries);
    let Some(changed) = ScriptData::holding(&entries) else {
        return false;
    };
    world.set_component(entity, &DATA, changed)
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use super::*;

    fn entries() -> Entries {
        Entries::from([(
            "home".to_string(),
            Value::List(vec![Value::Int(3), Value::Int(70)]),
        )])
    }

    #[test]
    fn the_layout_is_the_one_saves_hold() {
        assert_eq!(offset_of!(ScriptData, version), 0);
        assert_eq!(offset_of!(ScriptData, used), 4);
        assert_eq!(offset_of!(ScriptData, bytes), 8);
        assert_eq!(size_of::<ScriptData>(), 8 + DATA_BYTES);
    }

    #[test]
    fn entries_come_back_from_the_component() {
        let data = ScriptData::holding(&entries()).expect("one entry fits");
        assert_eq!(data.entries(), entries());
        assert_eq!(
            ScriptData::EMPTY.entries(),
            Entries::new(),
            "a new entity has none"
        );
    }

    #[test]
    fn another_version_reads_as_no_data() {
        let mut data = ScriptData::holding(&entries()).expect("one entry fits");
        data.version = VERSION + 1;
        assert_eq!(data.entries(), Entries::new());
    }

    #[test]
    fn a_length_past_the_end_reads_as_no_data() {
        let mut data = ScriptData::holding(&entries()).expect("one entry fits");
        data.used = DATA_BYTES as u32 + 1;
        assert_eq!(data.entries(), Entries::new());
    }

    #[test]
    fn more_than_the_component_holds_is_refused() {
        let full = Entries::from([("text".to_string(), Value::Text("x".repeat(DATA_BYTES)))]);
        assert!(
            ScriptData::holding(&full).is_none(),
            "the text alone fills it"
        );
        let fits = Entries::from([("text".to_string(), Value::Text("x".repeat(DATA_BYTES - 8)))]);
        assert!(
            ScriptData::holding(&fits).is_some(),
            "eight bytes go to key, tag and length"
        );
    }
}
