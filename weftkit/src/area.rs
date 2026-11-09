//! A box of blocks, both corners included: what a machine works, a set fills, a structure
//! covers. One block high, it is a rectangle on the ground.

use weft_sdk::*;

use crate::Pos;

value! {
    /// The blocks from its lowest corner `lo` to its highest `hi`, both included.
    #[derive(Default)]
    pub struct Area {
        pub lo: Pos,
        pub hi: Pos,
    }
}

impl Area {
    /// The box with the opposite corners `a` and `b`.
    pub fn between(a: Pos, b: Pos) -> Area {
        let (lo, hi) = Pos::corners(a, b);
        Area { lo, hi }
    }

    /// The box of `size` blocks along x, y and z from its lowest corner `lo`.
    pub fn sized(lo: Pos, [sx, sy, sz]: [i32; 3]) -> Area {
        Area {
            lo,
            hi: lo + Pos::new(sx - 1, sy - 1, sz - 1),
        }
    }

    /// Blocks along x, y and z, its edges included.
    pub fn size(&self) -> [i32; 3] {
        let (lo, hi) = (self.lo, self.hi);
        [hi.x - lo.x + 1, hi.y - lo.y + 1, hi.z - lo.z + 1]
    }

    pub fn contains(&self, p: Pos) -> bool {
        self.spans(p) && (self.lo.y..=self.hi.y).contains(&p.y)
    }

    /// Whether `p` lies in, over or under the box, its edges included.
    pub fn spans(&self, p: Pos) -> bool {
        (self.lo.x..=self.hi.x).contains(&p.x) && (self.lo.z..=self.hi.z).contains(&p.z)
    }

    /// Whether the two boxes share a block.
    pub fn overlaps(&self, other: &Area) -> bool {
        let (a, b) = (self, other);
        let apart = |a_lo: i32, a_hi: i32, b_lo: i32, b_hi: i32| a_hi < b_lo || b_hi < a_lo;
        !(apart(a.lo.x, a.hi.x, b.lo.x, b.hi.x)
            || apart(a.lo.y, a.hi.y, b.lo.y, b.hi.y)
            || apart(a.lo.z, a.hi.z, b.lo.z, b.hi.z))
    }

    /// Grows the box to take in `p`.
    pub fn grow_to(&mut self, p: Pos) {
        let (lo, _) = Pos::corners(self.lo, p);
        let (_, hi) = Pos::corners(self.hi, p);
        *self = Area { lo, hi };
    }

    /// The same columns, from height `lo` to `hi`.
    pub fn heights(&self, lo: i32, hi: i32) -> Area {
        Area::between(self.lo.with_y(lo), self.hi.with_y(hi))
    }

    /// The four corners of its lowest layer.
    pub fn bottom_corners(&self) -> [Pos; 4] {
        let (lo, hi) = (self.lo, self.hi);
        [
            lo,
            Pos { x: hi.x, ..lo },
            Pos { z: hi.z, ..lo },
            hi.with_y(lo.y),
        ]
    }

    /// The x and z of each column it spans.
    pub fn columns(&self) -> impl Iterator<Item = (i32, i32)> {
        let (lo, hi) = (self.lo, self.hi);
        (lo.x..=hi.x).flat_map(move |x| (lo.z..=hi.z).map(move |z| (x, z)))
    }

    /// Every block in it.
    pub fn blocks(&self) -> impl Iterator<Item = Pos> {
        let (lo, hi) = (self.lo.y, self.hi.y);
        self.columns()
            .flat_map(move |(x, z)| (lo..=hi).map(move |y| Pos::new(x, y, z)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(lo: [i32; 3], hi: [i32; 3]) -> Area {
        Area::between(lo.into(), hi.into())
    }

    #[test]
    fn an_area_is_the_same_from_either_pair_of_corners() {
        let a = Area::between(Pos::new(5, 2, -3), Pos::new(-1, 0, 4));
        assert_eq!(a, area([-1, 2, 4], [5, 0, -3]), "either pair");
        assert_eq!((a.lo, a.hi), (Pos::new(-1, 0, -3), Pos::new(5, 2, 4)));
        assert_eq!(a.size(), [7, 3, 8], "edges included");
        assert_eq!(Area::sized(a.lo, a.size()), a, "and back from its size");
    }

    #[test]
    fn a_flat_area_has_four_corners_and_spans_its_columns() {
        let a = area([-1, 5, -3], [5, 5, 4]);
        let corners = a.bottom_corners().map(|c| [c.x, c.y, c.z]);
        let expected = [[-1, 5, -3], [5, 5, -3], [-1, 5, 4], [5, 5, 4]];
        assert_eq!(corners, expected, "at its height");
        assert!(
            a.spans(Pos::new(5, 99, 4)),
            "a corner column, at any height"
        );
        assert!(!a.contains(Pos::new(5, 99, 4)), "but not above it");
        assert!(a.contains(Pos::new(0, 5, 0)), "inside");
        assert!(!a.spans(Pos::new(6, 5, 0)), "not past its edge");
    }

    #[test]
    fn an_area_grows_to_take_in_a_block() {
        let mut a = area([0, 0, 0], [2, 0, 2]);
        a.grow_to(Pos::new(-3, 0, 5));
        assert_eq!(a, area([-3, 0, 0], [2, 0, 5]), "out both ways");
        a.grow_to(Pos::new(1, 4, 1));
        assert_eq!(a, area([-3, 0, 0], [2, 4, 5]), "and up");
    }

    #[test]
    fn areas_overlap_when_they_share_a_block() {
        let house = area([0, 0, 0], [4, 3, 4]);
        assert!(
            house.overlaps(&area([4, 3, 4], [6, 6, 6])),
            "a corner block"
        );
        assert!(!house.overlaps(&area([5, 0, 0], [6, 3, 4])), "side by side");
        assert!(!house.overlaps(&area([0, 4, 0], [4, 5, 4])), "one on top");
    }

    #[test]
    fn heights_keep_the_columns_and_set_the_layers() {
        let a = area([-2, 0, 1], [3, 5, 4]);
        let below = a.heights(-9, -2);
        assert_eq!(below, area([-2, -9, 1], [3, -2, 4]));
        assert!(below.columns().eq(a.columns()), "the same columns");
    }

    #[test]
    fn every_block_is_counted_once_edges_included() {
        let a = area([0, 0, 0], [1, 2, 3]);
        assert_eq!(a.columns().count(), 2 * 4, "columns");
        let blocks: Vec<Pos> = a.blocks().collect();
        assert_eq!(blocks.len(), 2 * 3 * 4, "blocks");
        assert!(blocks.contains(&Pos::new(1, 2, 3)), "the far corner");
    }
}
