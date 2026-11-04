//! A box of blocks: what a set fills or levels.

use crate::Pos;

/// The blocks between two corners, both included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    pub lo: Pos,
    pub hi: Pos,
}

impl Area {
    /// The box with the opposite corners `a` and `b`.
    pub fn between(a: Pos, b: Pos) -> Area {
        Area {
            lo: Pos::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            hi: Pos::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
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

    #[test]
    fn an_area_is_the_same_from_either_pair_of_corners() {
        let a = Area::between(Pos::new(5, 2, -3), Pos::new(-1, 0, 4));
        let b = Area::between(Pos::new(-1, 2, 4), Pos::new(5, 0, -3));
        assert_eq!(a, b);
        assert_eq!((a.lo, a.hi), (Pos::new(-1, 0, -3), Pos::new(5, 2, 4)));
    }

    #[test]
    fn every_block_is_counted_once_edges_included() {
        let a = Area::between(Pos::new(0, 0, 0), Pos::new(1, 2, 3));
        assert_eq!(a.columns().count(), 2 * 4, "columns");
        let blocks: Vec<Pos> = a.blocks().collect();
        assert_eq!(blocks.len(), 2 * 3 * 4, "blocks");
        assert!(blocks.contains(&Pos::new(1, 2, 3)), "the far corner");
    }
}
