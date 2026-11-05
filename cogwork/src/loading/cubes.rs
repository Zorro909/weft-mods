//! Boxes of cubes: the 16-block units the engine loads the world in, and keeps loaded.

use crate::lib::Pos;

/// Blocks along each side of a cube.
pub const CUBE: i32 = 16;

/// The cubes from `lo` to `hi`, both included, in cube coordinates (a block's divided by 16).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cubes {
    pub lo: Pos,
    pub hi: Pos,
}

impl Cubes {
    /// The cubes up to `range` away from the one the block `pos` is in, along x and z, at its
    /// height.
    pub fn around(pos: Pos, range: i32) -> Cubes {
        let center = cube_of(pos);
        Cubes {
            lo: center.add_x(-range).add_z(-range),
            hi: center.add_x(range).add_z(range),
        }
    }

    /// The cubes the blocks from `a` to `b` are in, both included.
    pub fn spanning(a: Pos, b: Pos) -> Cubes {
        let (a, b) = (cube_of(a), cube_of(b));
        Cubes {
            lo: Pos::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            hi: Pos::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
    }

    pub fn count(&self) -> i64 {
        let across = |lo: i32, hi: i32| (hi - lo + 1).max(0) as i64;
        let (lo, hi) = (self.lo, self.hi);
        across(lo.x, hi.x) * across(lo.y, hi.y) * across(lo.z, hi.z)
    }

    /// Whether every cube of `other` is one of these.
    pub fn contains(&self, other: &Cubes) -> bool {
        let (lo, hi) = (self.lo, self.hi);
        let inside = |p: Pos| {
            (lo.x..=hi.x).contains(&p.x)
                && (lo.y..=hi.y).contains(&p.y)
                && (lo.z..=hi.z).contains(&p.z)
        };
        inside(other.lo) && inside(other.hi)
    }

    /// The first and the last block of the box, as `World::keep_loaded` takes it.
    pub fn blocks(&self) -> (Pos, Pos) {
        let last = CUBE - 1;
        let first_block = |c: Pos| Pos::new(c.x * CUBE, c.y * CUBE, c.z * CUBE);
        let end = first_block(self.hi);
        (
            first_block(self.lo),
            Pos::new(end.x + last, end.y + last, end.z + last),
        )
    }

    /// Each cube's first block.
    pub fn each(&self) -> impl Iterator<Item = Pos> + use<> {
        let (lo, hi) = (self.lo, self.hi);
        (lo.y..=hi.y).flat_map(move |y| {
            (lo.z..=hi.z).flat_map(move |z| {
                (lo.x..=hi.x).map(move |x| Pos::new(x * CUBE, y * CUBE, z * CUBE))
            })
        })
    }
}

fn cube_of(pos: Pos) -> Pos {
    Pos::new(
        pos.x.div_euclid(CUBE),
        pos.y.div_euclid(CUBE),
        pos.z.div_euclid(CUBE),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_below_zero_are_in_the_cube_below() {
        let cubes = Cubes::around(Pos::new(-1, -16, 15), 0);
        assert_eq!(cubes.lo, Pos::new(-1, -1, 0), "-1 and -16 in -1, 15 in 0");
        assert_eq!(cubes.count(), 1, "one cube");
    }

    #[test]
    fn a_range_spreads_along_x_and_z_only() {
        let cubes = Cubes::around(Pos::new(40, 70, 8), 3);
        assert_eq!(cubes.count(), 49, "7 by 7, one high");
        assert_eq!(
            cubes.lo,
            Pos::new(-1, 4, -3),
            "three each way of cube 2, 4, 0"
        );
        assert_eq!(cubes.hi, Pos::new(5, 4, 3), "and the same up");
    }

    #[test]
    fn the_blocks_run_from_the_first_cube_to_the_end_of_the_last() {
        let cubes = Cubes::around(Pos::new(0, 0, 0), 1);
        let (from, to) = cubes.blocks();
        assert_eq!(
            from,
            Pos::new(-16, 0, -16),
            "the first block of the first cube"
        );
        assert_eq!(to, Pos::new(31, 15, 31), "the last block of the last cube");
    }

    #[test]
    fn a_span_takes_every_cube_its_blocks_are_in() {
        let cubes = Cubes::spanning(Pos::new(20, 30, -1), Pos::new(-17, 33, 15));
        assert_eq!(cubes.lo, Pos::new(-2, 1, -1), "-17 is two cubes down");
        assert_eq!(cubes.hi, Pos::new(1, 2, 0), "33 is in the third cube up");
    }

    #[test]
    fn a_cube_is_contained_only_when_inside() {
        let home = Cubes::around(Pos::new(0, 0, 0), 1);
        assert!(
            home.contains(&Cubes::around(Pos::new(20, 5, -10), 0)),
            "beside it"
        );
        assert!(
            !home.contains(&Cubes::around(Pos::new(0, -1, 0), 0)),
            "below it"
        );
        assert!(
            !home.contains(&Cubes::around(Pos::new(32, 0, 0), 0)),
            "two away"
        );
    }

    #[test]
    fn each_cube_is_listed_once() {
        let cubes = Cubes::around(Pos::new(0, 0, 0), 1);
        let all: Vec<Pos> = cubes.each().collect();
        assert_eq!(all.len() as i64, cubes.count(), "as many as it counts");
        assert!(all.contains(&Pos::new(-16, 0, 16)), "a corner cube");
    }
}
