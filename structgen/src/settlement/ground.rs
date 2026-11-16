//! Which columns a settlement may build on: read from the blocks generation leaves before
//! structures, so it holds in any dimension, whatever its sea level, and on every client alike.

use weft_sdk::abi::block_flags::{LIQUID, REPLACEABLE};
use weft_sdk::state_flags;
use weft_sdk::terrain::Column;
use weftkit::BlockState;

/// What a block is to something built on the ground below it.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cover {
    /// Air, or what a building simply takes the place of: grass, flowers, a layer of snow.
    Open,
    /// Water or another liquid: a lake, the sea, a river.
    Liquid,
    /// Anything else: ice on a pond, rock overhanging.
    Solid,
}

impl Cover {
    /// What `state` is, as the engine's flags for it say.
    pub fn of(state: BlockState) -> Cover {
        if state == BlockState::AIR {
            return Cover::Open;
        }
        let flags = state_flags(state.id());
        if flags & LIQUID != 0 {
            Cover::Liquid
        } else if flags & REPLACEABLE != 0 {
            Cover::Open
        } else {
            Cover::Solid
        }
    }
}

/// `column`'s ground, if something may stand on it: there is terrain, its top block `top` is
/// still there and no liquid (no cave took it, no sea covers it), and what lies on it, `above`,
/// is open. `cover` says what a block is.
pub fn dry(
    column: Column,
    top: BlockState,
    above: BlockState,
    cover: impl Fn(BlockState) -> Cover,
) -> Option<Column> {
    let has_ground = column.height != i32::MIN && top != BlockState::AIR;
    let dry = has_ground && cover(top) != Cover::Liquid && cover(above) == Cover::Open;
    dry.then_some(column)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WATER: u32 = 9;
    const ICE: u32 = 10;
    const STONE: u32 = 11;
    const SNOW: u32 = 12;

    fn cover(state: BlockState) -> Cover {
        match state.id() {
            0 | SNOW => Cover::Open,
            WATER => Cover::Liquid,
            _ => Cover::Solid,
        }
    }

    fn at(height: i32) -> Column {
        Column { height, biome: 1 }
    }

    fn dry_at(height: i32, top: u32, above: u32) -> bool {
        dry(at(height), top.into(), above.into(), cover).is_some()
    }

    #[test]
    fn open_ground_is_built_on() {
        assert!(dry_at(70, STONE, 0), "nothing on it");
        assert!(dry_at(70, STONE, SNOW), "a layer of snow on it");
        assert!(dry_at(20, STONE, 0), "low ground with no water on it");
    }

    #[test]
    fn ground_under_water_is_not() {
        assert!(!dry_at(58, STONE, WATER), "the sea floor");
        assert!(!dry_at(80, STONE, WATER), "a lake in the hills");
        assert!(!dry_at(62, WATER, WATER), "where the top is water");
    }

    #[test]
    fn ground_under_ice_is_not() {
        assert!(!dry_at(63, STONE, ICE), "a frozen pond");
    }

    #[test]
    fn a_column_without_ground_is_not() {
        assert!(!dry_at(i32::MIN, STONE, 0), "no terrain");
        assert!(!dry_at(70, 0, 0), "a cave mouth took the top block");
    }
}
