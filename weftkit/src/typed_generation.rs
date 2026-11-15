//! `Generation`'s block reads with typed block states, as `TypedWorld` has `World`'s: each is the
//! SDK's method of the same name with `_typed` after it, giving `BlockState` where the SDK gives
//! a bare `u32` and taking the same arguments otherwise. A plan or a decoration reaches it
//! through `p.view()` or `g.view()`, and a look outside generation through
//! `Dimension::generation`.

use weft_sdk::{BlockPos, Dir, Generation};

use crate::BlockState;

pub trait TypedGeneration {
    /// `Generation::read`.
    fn read_typed(&self, pos: impl Into<BlockPos>) -> BlockState;
    /// `Generation::find`.
    fn find_typed(
        &self,
        from: impl Into<BlockPos>,
        dir: Dir,
        distance: u32,
        test: impl FnMut(BlockState) -> bool,
    ) -> Option<BlockPos>;
    /// `Generation::all`.
    fn all_typed(
        &self,
        from: impl Into<BlockPos>,
        to: impl Into<BlockPos>,
        test: impl FnMut(BlockState) -> bool,
    ) -> bool;
    /// `Generation::surface_block`.
    fn surface_block_typed(&self, x: i32, z: i32) -> BlockState;
}

impl TypedGeneration for Generation<'_> {
    fn read_typed(&self, pos: impl Into<BlockPos>) -> BlockState {
        self.read(pos).into()
    }

    fn find_typed(
        &self,
        from: impl Into<BlockPos>,
        dir: Dir,
        distance: u32,
        mut test: impl FnMut(BlockState) -> bool,
    ) -> Option<BlockPos> {
        self.find(from, dir, distance, |state| test(state.into()))
    }

    fn all_typed(
        &self,
        from: impl Into<BlockPos>,
        to: impl Into<BlockPos>,
        mut test: impl FnMut(BlockState) -> bool,
    ) -> bool {
        self.all(from, to, |state| test(state.into()))
    }

    fn surface_block_typed(&self, x: i32, z: i32) -> BlockState {
        self.surface_block(x, z).into()
    }
}
