//! What other mods need to work with redstone: names and value layouts only, never the mod's
//! code (the statics compile into whoever uses them, so they carry full names).

use weft_sdk::*;

pub use redstone_core::{FULL, Range};

/// The world's signal range: 16, 32 or 64 levels. Change with `/rule redstone:signal_range <0|1|2>`.
pub static RANGE: Config<u32> = Config::named("redstone:signal_range");

/// The range of the world that is running now.
pub fn range() -> Range {
    Range::from_index(RANGE.get())
}
