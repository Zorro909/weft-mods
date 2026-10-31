//! An in-memory world for tests.

use std::collections::BTreeMap;

use crate::{Cell, Dir, Pos, View, Wire};

#[derive(Clone, Default, Debug)]
pub struct MapGrid {
    pub cells: BTreeMap<Pos, Cell>,
}

impl MapGrid {
    pub fn put(&mut self, p: Pos, c: Cell) -> &mut Self {
        self.cells.insert(p, c);
        self
    }
    pub fn solid(&mut self, p: Pos) -> &mut Self {
        self.put(p, Cell::block(true))
    }
    pub fn dust(&mut self, p: Pos) -> &mut Self {
        self.put(p, Cell::Wire(Wire::dust(0)))
    }
    pub fn unloaded(&mut self, p: Pos) -> &mut Self {
        self.put(p, Cell::Unloaded)
    }
    pub fn air(&mut self, p: Pos) -> &mut Self {
        self.cells.remove(&p);
        self
    }

    /// The same build turned a quarter clockwise around the Y axis through the origin.
    pub fn rotated(&self) -> MapGrid {
        MapGrid { cells: self.cells.iter().map(|(p, c)| (rotate_pos(*p), rotate_cell(c))).collect() }
    }
}

/// `(x, z) → (−z, x)`: north (−Z) becomes east (+X), matching `Dir::rotate_y`.
pub fn rotate_pos(p: Pos) -> Pos {
    [-p[2], p[1], p[0]]
}

pub fn rotate_sides<T: Copy>(a: [T; 6]) -> [T; 6] {
    let mut out = a;
    for d in Dir::ALL {
        out[d.rotate_y().index()] = a[d.index()];
    }
    out
}

pub fn rotate_mask(m: u8) -> u8 {
    Dir::ALL.iter().filter(|d| m & (1 << d.index()) != 0).fold(0, |acc, d| acc | 1 << d.rotate_y().index())
}

fn rotate_cell(c: &Cell) -> Cell {
    match c {
        Cell::Device(d) => {
            let mut d = d.clone();
            d.output = rotate_sides(d.output);
            d.connects = rotate_mask(d.connects);
            d.kind = d.kind.map(|k| k.rotate_y());
            Cell::Device(d)
        }
        c => c.clone(),
    }
}

impl View for MapGrid {
    fn cell(&self, p: Pos) -> Cell {
        self.cells.get(&p).cloned().unwrap_or(Cell::Air)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Device, Kind, Power};

    #[test]
    fn four_rotations_are_identity() {
        let mut g = MapGrid::default();
        let mut output = [Power::default(); 6];
        output[Dir::North.index()] = Power { strength: 7, strong: true };
        g.put([1, 2, 3], Cell::Device(Device { output, connects: 0b010001, receives: true, solid: false, kind: Some(Kind::Torch { lit: true, attached: Dir::West }) }));
        g.dust([-4, 0, 9]);
        let back = g.rotated().rotated().rotated().rotated();
        assert_eq!(back.cells, g.cells);
        let once = g.rotated();
        let Cell::Device(d) = once.cell(rotate_pos([1, 2, 3])) else { panic!() };
        assert_eq!(d.output[Dir::East.index()].strength, 7);
        assert_eq!(d.kind, Some(Kind::Torch { lit: true, attached: Dir::North }));
    }
}
