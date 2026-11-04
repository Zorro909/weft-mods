//! A block as it stands in the world: which block, and the values of its properties. The SDK
//! passes these around as bare `u32` states; `BlockState::from` takes one.

use weft_sdk::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockState(u32);

impl BlockState {
    /// The engine's air.
    pub const AIR: BlockState = BlockState(0);

    /// `block` with its properties at their defaults.
    pub fn of(block: &Block) -> BlockState {
        BlockState(block.id())
    }

    /// The block named `name` (`base:stone`), if a mod registered it.
    pub fn named(name: &str) -> Option<BlockState> {
        Block::lookup(name).map(BlockState)
    }

    /// The SDK's number for it, for the SDK calls that take one.
    pub fn id(self) -> u32 {
        self.0
    }
}

impl From<u32> for BlockState {
    fn from(state: u32) -> BlockState {
        BlockState(state)
    }
}
