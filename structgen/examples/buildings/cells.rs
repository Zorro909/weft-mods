//! A structure's cells as a walker meets them: which are open, which give a footing.

use crate::draft::AIR;
use crate::structure_file::Structure;

/// A cell, in blocks from the structure's lowest corner: x east, y up, z south.
pub type Cell = [i32; 3];

/// Blocks a player walks through.
const OPEN: &[&str] = &[
    AIR,
    "@light",
    "base:torch",
    "base:ladder",
    "base:wheat",
    "base:water",
    "base:tall_grass",
    "base:dandelion",
    "base:rose",
    "base:sapling",
    "base:sugar_cane",
    "core:covered",
];
pub const LADDER: &str = "base:ladder";

/// A structure's cells by where they are, for asking about one and its neighbours.
pub struct Cells<'a> {
    pub size: Cell,
    names: Vec<&'a str>,
}

impl<'a> Cells<'a> {
    pub fn of(structure: &'a Structure) -> Cells<'a> {
        let size = structure.size();
        let mut cells = Cells {
            size,
            names: vec![AIR; size.iter().product::<i32>() as usize],
        };
        for (at, name) in structure.blocks() {
            let index = cells.index(at);
            cells.names[index] = name;
        }
        cells
    }

    fn index(&self, [x, y, z]: Cell) -> usize {
        ((y * self.size[2] + z) * self.size[0] + x) as usize
    }

    pub fn holds(&self, at: Cell) -> bool {
        (0..3).all(|axis| (0..self.size[axis]).contains(&at[axis]))
    }

    /// The block at `at` without its properties; air around and above the box, and solid
    /// ground under it.
    pub fn block(&self, at: Cell) -> &'a str {
        if at[1] < 0 {
            return "base:stone";
        }
        if !self.holds(at) {
            return AIR;
        }
        let name = self.names[self.index(at)];
        name.split('[').next().unwrap_or(name)
    }

    pub fn open(&self, at: Cell) -> bool {
        OPEN.contains(&self.block(at))
    }

    /// Whether the block at `at` is a low step, a stair or a bottom slab, which a walker
    /// stands on inside its cell.
    fn low_step(&self, at: Cell) -> bool {
        if !self.holds(at) {
            return false;
        }
        let name = self.names[self.index(at)];
        let stepped = ["stair", "slab", "step", "gutter"]
            .iter()
            .any(|part| name.contains(part));
        stepped && !name.contains("=top") && !name.contains("=double")
    }

    /// Whether a walker can have its feet in `at`: two cells of room, on something to stand
    /// on or a ladder to hold.
    pub fn standable(&self, [x, y, z]: Cell) -> bool {
        if self.low_step([x, y, z]) {
            return self.open([x, y + 1, z]) && self.open([x, y + 2, z]);
        }
        let held = [[x, y, z], [x, y - 1, z]]
            .iter()
            .any(|&at| self.block(at) == LADDER);
        let room = self.open([x, y, z]) && self.open([x, y + 1, z]);
        room && (held || !self.open([x, y - 1, z]))
    }
}
