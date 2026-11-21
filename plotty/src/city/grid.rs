//! The grid cities are laid out on: 16 by 16 block cells, the same for the whole world, so a
//! cell is found from a position alone. A footprint is a square of cells something is built on.

use serde::{Deserialize, Serialize};
use weftkit::{Area, Pos};

/// Blocks along each side of a cell.
pub const CELL: i32 = 16;

/// A cell of the grid: the blocks from `x * 16` to `x * 16 + 15`, and the same along z.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CellXZ {
    pub x: i32,
    pub z: i32,
}

impl CellXZ {
    pub const fn new(x: i32, z: i32) -> CellXZ {
        CellXZ { x, z }
    }

    /// The cell holding the block `pos`.
    pub fn of(pos: Pos) -> CellXZ {
        CellXZ::new(pos.x.div_euclid(CELL), pos.z.div_euclid(CELL))
    }

    /// Its block with the lowest x and z, at height `y`.
    pub fn corner(self, y: i32) -> Pos {
        Pos::new(self.x * CELL, y, self.z * CELL)
    }

    /// The four cells sharing a side with it.
    pub fn neighbours(self) -> [CellXZ; 4] {
        let CellXZ { x, z } = self;
        [
            CellXZ::new(x, z - 1),
            CellXZ::new(x + 1, z),
            CellXZ::new(x, z + 1),
            CellXZ::new(x - 1, z),
        ]
    }
}

/// A side of a footprint, as the compass has it: north is toward -z.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    North,
    East,
    South,
    West,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::South, Side::North, Side::West, Side::East];
}

/// A square of `side` cells from `corner` toward +x and +z.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Footprint {
    pub corner: CellXZ,
    pub side: i32,
}

impl Footprint {
    /// The square of `side` cells whose middle lies nearest the block `pos`, so that the player
    /// standing there stands inside it.
    pub fn around(pos: Pos, side: i32) -> Footprint {
        let toward_corner = (side - 1) * CELL / 2;
        let corner = CellXZ::new(
            (pos.x - toward_corner).div_euclid(CELL),
            (pos.z - toward_corner).div_euclid(CELL),
        );
        Footprint { corner, side }
    }

    pub fn cells(self) -> impl Iterator<Item = CellXZ> {
        let Footprint { corner, side } = self;
        (0..side)
            .flat_map(move |dz| (0..side).map(move |dx| CellXZ::new(corner.x + dx, corner.z + dz)))
    }

    /// Whether it and `other` share a cell.
    pub fn overlaps(self, other: Footprint) -> bool {
        let apart = |a: i32, b: i32, a_side: i32, b_side: i32| a + a_side <= b || b + b_side <= a;
        !apart(self.corner.x, other.corner.x, self.side, other.side)
            && !apart(self.corner.z, other.corner.z, self.side, other.side)
    }

    /// The cells outside it that share a side with it.
    pub fn ring(self) -> Vec<CellXZ> {
        let inside: Vec<CellXZ> = self.cells().collect();
        let mut ring: Vec<CellXZ> = inside
            .iter()
            .flat_map(|cell| cell.neighbours())
            .filter(|cell| !inside.contains(cell))
            .collect();
        ring.sort_by_key(|cell| (cell.z, cell.x));
        ring.dedup();
        ring
    }

    /// The cells just outside it along `facing`.
    pub fn beyond(self, facing: Side) -> Vec<CellXZ> {
        let Footprint { corner, side } = self;
        (0..side)
            .map(|step| match facing {
                Side::North => CellXZ::new(corner.x + step, corner.z - 1),
                Side::South => CellXZ::new(corner.x + step, corner.z + side),
                Side::West => CellXZ::new(corner.x - 1, corner.z + step),
                Side::East => CellXZ::new(corner.x + side, corner.z + step),
            })
            .collect()
    }

    /// Its blocks from height `low` to `high`, both included.
    pub fn area(self, low: i32, high: i32) -> Area {
        let size = self.side * CELL;
        Area::sized(self.corner.corner(low), [size, high - low + 1, size])
    }

    /// Blocks along each side.
    pub fn blocks(self) -> i32 {
        self.side * CELL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cell_holds_sixteen_blocks_each_way_negatives_too() {
        assert_eq!(
            CellXZ::of(Pos::new(0, 5, 15)),
            CellXZ::new(0, 0),
            "first cell"
        );
        assert_eq!(
            CellXZ::of(Pos::new(16, 5, -1)),
            CellXZ::new(1, -1),
            "next and before"
        );
        assert_eq!(
            CellXZ::of(Pos::new(-16, 5, -17)),
            CellXZ::new(-1, -2),
            "below zero"
        );
    }

    #[test]
    fn a_footprint_is_built_around_its_player() {
        for (x, side) in [(5, 1), (20, 2), (30, 2), (40, 4), (-3, 2)] {
            let footprint = Footprint::around(Pos::new(x, 0, x), side);
            let stands_in = CellXZ::of(Pos::new(x, 0, x));
            assert!(
                footprint.cells().any(|cell| cell == stands_in),
                "x {x}, side {side}: inside"
            );
        }
    }

    #[test]
    fn a_footprint_has_its_cells_and_a_ring() {
        let footprint = Footprint {
            corner: CellXZ::new(0, 0),
            side: 2,
        };
        assert_eq!(footprint.cells().count(), 4, "two by two");
        assert_eq!(footprint.ring().len(), 8, "two on each side");
        assert_eq!(footprint.blocks(), 32, "32 blocks a side");
    }

    #[test]
    fn beyond_a_side_lies_a_row_as_long_as_it() {
        let footprint = Footprint {
            corner: CellXZ::new(0, 0),
            side: 2,
        };
        assert_eq!(
            footprint.beyond(Side::South),
            vec![CellXZ::new(0, 2), CellXZ::new(1, 2)],
            "south"
        );
        assert_eq!(
            footprint.beyond(Side::West),
            vec![CellXZ::new(-1, 0), CellXZ::new(-1, 1)],
            "west"
        );
    }

    #[test]
    fn a_footprints_area_spans_its_heights() {
        let area = Footprint {
            corner: CellXZ::new(-1, 2),
            side: 1,
        }
        .area(60, 70);
        assert!(area.contains(Pos::new(-16, 60, 32)), "low corner");
        assert!(area.contains(Pos::new(-1, 70, 47)), "high corner");
        assert!(!area.contains(Pos::new(0, 65, 40)), "next cell");
    }

    #[test]
    fn footprints_overlap_when_they_share_a_cell() {
        let plaza = Footprint {
            corner: CellXZ::new(0, 0),
            side: 2,
        };
        let at = |x, z| Footprint {
            corner: CellXZ::new(x, z),
            side: 1,
        };
        assert!(plaza.overlaps(at(1, 1)), "its far corner");
        assert!(!plaza.overlaps(at(2, 0)), "beside it");
        assert!(!plaza.overlaps(at(-1, 1)), "beside it on the other side");
    }
}
