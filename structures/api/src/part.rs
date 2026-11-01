//! What a plan is made of: templates and boxes of blocks, in the order the planner added them.

use std::sync::Arc;

use weft_sdk::Block;

use crate::fit::{self, Ground};
use crate::templates::Entry;
use crate::turn::Turn;

pub(crate) enum Part {
    Piece(Piece),
    /// Every block from `from` to `to`, both included.
    Fill {
        from: [i32; 3],
        to: [i32; 3],
        state: u32,
    },
}

/// A template placed by a plan.
pub(crate) struct Piece {
    pub template: Arc<Entry>,
    /// Its lowest corner.
    pub corner: [i32; 3],
    pub turn: Turn,
    /// Filled under its bottom layer down to the ground.
    pub foundation: Option<u32>,
    /// Ground rising above its bottom layer is cut away.
    pub clear_above: bool,
    /// The ground height under each column of the turned footprint, x fastest, then z; filled
    /// once a foundation or clearing needs it.
    pub ground: Vec<Option<i32>>,
}

impl Piece {
    /// The size along x, y and z as it stands, turned.
    pub fn size(&self) -> [i32; 3] {
        self.turn.size(self.template.size())
    }
}

impl Part {
    /// The box of the blocks it sets itself (not counting foundations and clearing), both
    /// corners included.
    pub fn bounds(&self) -> ([i32; 3], [i32; 3]) {
        match self {
            Part::Piece(piece) => box_of(piece.corner, piece.size()),
            Part::Fill { from, to, .. } => (*from, *to),
        }
    }
}

/// The box of `size` blocks from `corner`, both corners included.
pub(crate) fn box_of(corner: [i32; 3], size: [i32; 3]) -> ([i32; 3], [i32; 3]) {
    let to = [0, 1, 2].map(|i| corner[i] + size[i] - 1);
    (corner, to)
}

/// A template just added to a plan, to say how it meets the ground.
pub struct Placed<'p> {
    pub(crate) piece: &'p mut Piece,
    pub(crate) ground: &'p dyn Fn(i32, i32) -> Option<Ground>,
}

impl Placed<'_> {
    /// Fills under its bottom layer down to the ground with `block`, so it never floats; only
    /// under blocks of the bottom layer that aren't air.
    pub fn foundation(self, block: &Block) -> Self {
        self.piece.foundation = Some(block.id()).filter(|&id| id != u32::MAX);
        self.with_ground()
    }

    /// Cuts away the ground that rises above its bottom layer, so no hill runs through it.
    pub fn clear_above(self) -> Self {
        self.piece.clear_above = true;
        self.with_ground()
    }

    fn with_ground(self) -> Self {
        if self.piece.ground.is_empty() {
            let [x, _, z] = self.piece.corner;
            let [sx, _, sz] = self.piece.size();
            self.piece.ground = fit::heights([x, z], [sx, sz], self.ground);
        }
        self
    }
}
