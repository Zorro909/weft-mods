//! A block as it stands in the world: which block, and the values of its properties (a tube's
//! arms, an engine's facing). The SDK passes these around as bare `u32` states; here they are a
//! `BlockState` that answers what cogwork asks of a block.

use weft_sdk::*;

use crate::pos::Pos;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct BlockState(u32);

impl BlockState {
    pub const AIR: BlockState = BlockState(0);

    /// The block at `pos`.
    pub fn at(w: &World, pos: Pos) -> BlockState {
        BlockState(w.get(pos.into()))
    }

    /// The block at `pos`, `None` while that part of the world isn't loaded.
    pub fn loaded_at(w: &World, pos: Pos) -> Option<BlockState> {
        w.get_loaded(pos.into()).map(BlockState)
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
        w.set(pos.into(), self.0)
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

    /// Its block's name with its properties, as loot tables and logs know it
    /// (`base:oak_log[axis=y]`).
    pub fn name(self) -> &'static str {
        block_name(self.0)
    }

    /// How hard it is to break; below 0 for blocks that can't be, `None` for blocks that don't
    /// say.
    pub fn hardness(self) -> Option<f32> {
        HARDNESS.on_block(self.0)
    }

    /// Whether something built there simply takes its place: air, plants, liquids.
    pub fn can_build_into(self) -> bool {
        use abi::block_flags::{LIQUID, REPLACEABLE};
        self == BlockState::AIR || state_flags(self.0) & (REPLACEABLE | LIQUID) != 0
    }

    /// Whether a machine can break it: not air or liquid, and not unbreakable.
    pub fn can_break(self) -> bool {
        self != BlockState::AIR
            && state_flags(self.0) & abi::block_flags::LIQUID == 0
            && self.hardness().is_none_or(|h| h >= 0.0)
    }

    /// Whether it is a whole block that fills its cell: solid and not see-through.
    pub fn is_whole(self) -> bool {
        let whole = abi::block_flags::SOLID | abi::block_flags::OPAQUE;
        state_flags(self.0) & whole == whole
    }
}
