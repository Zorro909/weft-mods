//! The drawn ground cut into structures `/structure load` can take: pieces of at most 64 by
//! 64, each a layer of ground with a layer above for crops, hedges and air.

use crate::draft::{AIR, Draft};
use crate::raster::{Cell, Ground};
use crate::structure_file::Structure;

/// The longest side of a piece.
const SIDE: usize = 64;

/// One piece and where its north-west corner goes.
pub struct Piece {
    pub corner: [i32; 2],
    pub structure: Structure,
}

pub fn cut(ground: &Ground) -> Vec<Piece> {
    let mut pieces = Vec::new();
    for z in (0..ground.size[1]).step_by(SIDE) {
        for x in (0..ground.size[0]).step_by(SIDE) {
            let size = [SIDE.min(ground.size[0] - x), SIDE.min(ground.size[1] - z)];
            pieces.push(Piece {
                corner: [ground.origin[0] + x as i32, ground.origin[1] + z as i32],
                structure: piece(ground, [x, z], size),
            });
        }
    }
    pieces
}

fn piece(ground: &Ground, from: [usize; 2], size: [usize; 2]) -> Structure {
    let mut d = Draft::new([size[0], 2, size[1]]);
    for z in 0..size[1] {
        for x in 0..size[0] {
            let [floor, above] = blocks(ground.get([from[0] + x, from[1] + z]));
            d.set([x, 0, z], &floor);
            d.set([x, 1, z], &above);
        }
    }
    d.done()
}

/// The block a cell puts in the ground and the one above it.
fn blocks(cell: Cell) -> [String; 2] {
    let floor = match cell {
        Cell::Ground | Cell::Hedge => "@ground",
        Cell::Furrow => "base:dirt",
        Cell::Crop(_) => "base:farmland",
        Cell::Rut | Cell::Worn => "@path/worn",
        Cell::Path => "@path/surface",
        Cell::Kerb => "@street/kerb",
        Cell::Paving | Cell::Square => "@street/paving",
        Cell::Patch | Cell::SquarePatch => "@street/patch",
        Cell::Gutter => "@street/gutter[type=bottom]",
    };
    let above = match cell {
        Cell::Hedge => "base:oak_leaves".to_string(),
        Cell::Crop(age) => format!("base:wheat[age={age}]"),
        _ => AIR.to_string(),
    };
    [floor.to_string(), above]
}
