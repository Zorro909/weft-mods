//! What a plan is made of: templates and boxes of blocks, in the order the planner added them.

use std::sync::Arc;

use weft_sdk::Block;
use weftkit::{Area, Pos};

use crate::fit::{self, Columns};
use crate::materials::Materials;
use crate::templates::Entry;
use crate::turn::Turn;

pub(crate) enum Part {
    Piece(Piece),
    /// Every block of `area`.
    Fill {
        area: Area,
        state: u32,
    },
}

/// What fills under a structure's bottom layer down to the ground, so it never floats. Only
/// under blocks of the bottom layer that aren't air.
#[derive(Clone, Copy)]
pub enum Foundation {
    /// The block the ground has on top in each column: grass, sand, snow.
    Ground,
    Block(&'static Block),
}

/// A template placed by a plan.
pub(crate) struct Piece {
    pub template: Arc<Entry>,
    /// Its lowest corner.
    pub corner: Pos,
    pub turn: Turn,
    pub materials: Materials,
    pub foundation: Option<Foundation>,
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
    /// The box of the blocks it sets itself, not counting foundations and clearing.
    pub fn bounds(&self) -> Area {
        match self {
            Part::Piece(piece) => Area::sized(piece.corner, piece.size()),
            Part::Fill { area, .. } => *area,
        }
    }
}

/// A template just added to a plan, to say how it meets the ground.
pub struct Placed<'p> {
    pub(crate) piece: &'p mut Piece,
    pub(crate) ground: Columns<'p>,
}

impl Placed<'_> {
    /// Fills its slots with `materials` instead of their defaults.
    pub fn materials(self, materials: &Materials) -> Self {
        self.piece.materials = materials.clone();
        self
    }

    pub fn foundation(self, foundation: Foundation) -> Self {
        self.piece.foundation = Some(foundation);
        self.with_ground()
    }

    /// Cuts away the ground that rises above its bottom layer, so no hill runs through it.
    pub fn clear_above(self) -> Self {
        self.piece.clear_above = true;
        self.with_ground()
    }

    fn with_ground(self) -> Self {
        if self.piece.ground.is_empty() {
            let Pos { x, z, .. } = self.piece.corner;
            let [sx, _, sz] = self.piece.size();
            self.piece.ground = fit::heights([x, z], [sx, sz], self.ground);
        }
        self
    }
}
