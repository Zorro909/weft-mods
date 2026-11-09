//! `World`'s block calls with typed block states: each is the SDK's method of the same name with
//! `_typed` after it, taking and giving `BlockState` where the SDK has a bare `u32` and the same
//! arguments otherwise. Once the SDK's own methods take typed states, dropping every `_typed`
//! (`sed -E 's/\.([a-z_]+)_typed\(/.\1(/g'`) and this trait's `use` moves a mod over.

use weft_sdk::*;

use crate::BlockState;

pub trait TypedWorld {
    /// `World::block`.
    fn block_typed(&self, pos: impl Into<BlockPos>) -> BlockState;
    /// `World::block_loaded`.
    fn block_loaded_typed(&self, pos: impl Into<BlockPos>) -> Option<BlockState>;
    /// `World::set_block`.
    fn set_block_typed(&self, pos: impl Into<BlockPos>, state: BlockState) -> bool;
    /// `World::parts`.
    fn parts_typed(&self, pos: impl Into<BlockPos>) -> Vec<BlockState>;
    /// `World::part_add`.
    fn part_add_typed(&self, pos: impl Into<BlockPos>, state: BlockState) -> bool;
    /// `World::part_remove`.
    fn part_remove_typed(&self, pos: impl Into<BlockPos>, state: BlockState) -> bool;
    /// `World::part_replace`.
    fn part_replace_typed(
        &self,
        pos: impl Into<BlockPos>,
        old: BlockState,
        new: BlockState,
    ) -> bool;
    /// `World::fill`.
    fn fill_typed(&self, shape: Shape, state: BlockState) -> Fill;
}

impl TypedWorld for World {
    fn block_typed(&self, pos: impl Into<BlockPos>) -> BlockState {
        self.block(pos).into()
    }

    fn block_loaded_typed(&self, pos: impl Into<BlockPos>) -> Option<BlockState> {
        self.block_loaded(pos).map(BlockState::from)
    }

    fn set_block_typed(&self, pos: impl Into<BlockPos>, state: BlockState) -> bool {
        self.set_block(pos, state.id())
    }

    fn parts_typed(&self, pos: impl Into<BlockPos>) -> Vec<BlockState> {
        self.parts(pos).into_iter().map(BlockState::from).collect()
    }

    fn part_add_typed(&self, pos: impl Into<BlockPos>, state: BlockState) -> bool {
        self.part_add(pos, state.id())
    }

    fn part_remove_typed(&self, pos: impl Into<BlockPos>, state: BlockState) -> bool {
        self.part_remove(pos, state.id())
    }

    fn part_replace_typed(
        &self,
        pos: impl Into<BlockPos>,
        old: BlockState,
        new: BlockState,
    ) -> bool {
        self.part_replace(pos, old.id(), new.id())
    }

    fn fill_typed(&self, shape: Shape, state: BlockState) -> Fill {
        self.fill(shape, state.id())
    }
}
