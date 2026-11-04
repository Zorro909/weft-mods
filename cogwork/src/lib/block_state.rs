//! A block as it stands in the world: which block, and the values of its properties (a tube's
//! arms, an engine's facing). The SDK passes these around as bare `u32` states; here they are a
//! `BlockState` that answers what cogwork asks of a block.

use weft_sdk::abi::block_flags::{LIQUID, OPAQUE, REPLACEABLE, SOLID};
use weft_sdk::*;

use super::Pos;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct BlockState(u32);

impl BlockState {
    pub const AIR: BlockState = BlockState(0);

    /// The block at `pos`.
    pub fn at(w: &World, pos: Pos) -> BlockState {
        BlockState(w.block(pos))
    }

    /// The block at `pos`, `None` while that part of the world isn't loaded.
    pub fn loaded_at(w: &World, pos: Pos) -> Option<BlockState> {
        w.block_loaded(pos).map(BlockState)
    }

    /// `block` with its properties at their defaults.
    pub fn of(block: &Block) -> BlockState {
        BlockState(block.id())
    }

    /// The block `item` places, if it places one.
    pub fn placed_by(item: u32) -> Option<BlockState> {
        item_block(item).map(BlockState)
    }

    /// Puts it at `pos`; false where the world isn't loaded.
    pub fn place(self, w: &World, pos: Pos) -> bool {
        w.set_block(pos, self.0)
    }

    /// The SDK's number for it, for the SDK calls that take one.
    pub fn id(self) -> u32 {
        self.0
    }

    /// Whether it is a state of `block`.
    pub fn is(self, block: &Block) -> bool {
        block.is(self.0)
    }

    /// Whether its block has `tag`.
    pub fn has(self, tag: &Tag) -> bool {
        tag.on_block(self.0)
    }

    /// The value of its property `name`, if it has that property.
    pub fn prop(self, name: &str) -> Option<&'static str> {
        state_prop(self.0, name)
    }

    /// Itself with the `true`/`false` property `name` set to `on`; unchanged without it.
    pub fn with_flag(self, name: &str, on: bool) -> BlockState {
        self.with_prop(name, if on { "true" } else { "false" })
            .unwrap_or(self)
    }

    /// Itself with its property `name` set to `value`, if it has that property and value.
    pub fn with_prop(self, name: &str, value: &str) -> Option<BlockState> {
        state_with(self.0, name, value).map(BlockState)
    }

    /// Its block's name without its properties (`base:oak_log`), as loot tables know it; empty
    /// for a state no block has.
    pub fn block_name(self) -> &'static str {
        block_info(self.0).map_or("", |info| info.name)
    }

    /// How hard it is to break; below 0 for blocks that can't be, `None` for blocks that don't
    /// say.
    pub fn hardness(self) -> Option<f32> {
        HARDNESS.on_block(self.0)
    }

    /// Whether something built there simply takes its place: air, plants, liquids.
    pub fn can_build_into(self) -> bool {
        self.traits().can_build_into()
    }

    /// Whether a machine can break it: not air or liquid, and not unbreakable.
    pub fn can_break(self) -> bool {
        self.traits().can_break()
    }

    /// Whether it is a whole block that fills its cell: solid and not see-through.
    pub fn is_whole(self) -> bool {
        self.traits().is_whole()
    }

    fn traits(self) -> Traits {
        if self == BlockState::AIR {
            return Traits::AIR;
        }
        Traits {
            air: false,
            flags: state_flags(self.0),
            hardness: self.hardness(),
        }
    }
}

/// What the engine knows of a block that the questions above are answered from, kept apart so
/// the answers can be tested without the engine.
struct Traits {
    air: bool,
    /// `abi::block_flags`.
    flags: u32,
    hardness: Option<f32>,
}

impl Traits {
    const AIR: Traits = Traits {
        air: true,
        flags: 0,
        hardness: None,
    };

    fn can_build_into(&self) -> bool {
        self.air || self.flags & (REPLACEABLE | LIQUID) != 0
    }

    fn can_break(&self) -> bool {
        !self.air && self.flags & LIQUID == 0 && self.hardness.is_none_or(|h| h >= 0.0)
    }

    fn is_whole(&self) -> bool {
        self.flags & (SOLID | OPAQUE) == SOLID | OPAQUE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(flags: u32, hardness: f32) -> Traits {
        Traits {
            air: false,
            flags,
            hardness: Some(hardness),
        }
    }

    #[test]
    fn air_is_built_into_but_never_broken() {
        let air = Traits::AIR;
        assert!(air.can_build_into(), "built into");
        assert!(!air.can_break(), "nothing to break");
        assert!(!air.is_whole(), "not whole");
        assert_eq!(BlockState::AIR.id(), 0, "the engine's air");
    }

    #[test]
    fn stone_is_broken_and_fills_its_cell() {
        let stone = block(SOLID | OPAQUE, 1.5);
        assert!(stone.can_break(), "broken");
        assert!(!stone.can_build_into(), "in the way");
        assert!(stone.is_whole(), "whole");
    }

    #[test]
    fn plants_and_liquids_give_way() {
        let grass = block(REPLACEABLE, 0.0);
        assert!(grass.can_build_into(), "grass is built over");
        assert!(grass.can_break(), "and can be broken");
        let water = block(LIQUID, 100.0);
        assert!(water.can_build_into(), "water is built into");
        assert!(!water.can_break(), "but not broken");
    }

    #[test]
    fn unbreakable_and_see_through_blocks() {
        let bedrock = block(SOLID | OPAQUE, -1.0);
        assert!(!bedrock.can_break(), "bedrock stays");
        let glass = block(SOLID, 0.3);
        assert!(!glass.is_whole(), "glass is see-through");
        let unknown = Traits {
            hardness: None,
            ..block(SOLID | OPAQUE, 0.0)
        };
        assert!(unknown.can_break(), "a block without a hardness breaks");
    }
}
