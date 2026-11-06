//! Cubes: the 16-block units the engine loads the world in, and keeps loaded. A cube has its
//! own position, `CubePos`, apart from a block's, so the two can't be mixed up.

use std::ops::Sub;

use weft_sdk::*;

use crate::lib::Pos;

/// Blocks along each side of a cube.
pub const CUBE: i32 = 16;

value! {
    /// A cube's position: that of its blocks divided by 16, rounded down.
    #[derive(Eq, Hash)]
    pub struct CubePos {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }
}

impl CubePos {
    pub const fn new(x: i32, y: i32, z: i32) -> CubePos {
        CubePos { x, y, z }
    }

    /// The cube the block `pos` is in.
    pub fn of(pos: Pos) -> CubePos {
        let cube = |block: i32| block.div_euclid(CUBE);
        CubePos::new(cube(pos.x), cube(pos.y), cube(pos.z))
    }

    /// Its lowest block in every direction.
    pub fn first_block(self) -> Pos {
        Pos::new(self.x * CUBE, self.y * CUBE, self.z * CUBE)
    }

    /// Its highest block in every direction.
    pub fn last_block(self) -> Pos {
        let last = CUBE - 1;
        self.first_block().add_x(last).add_y(last).add_z(last)
    }

    /// `dx` cubes along x from here.
    pub const fn add_x(self, dx: i32) -> CubePos {
        CubePos {
            x: self.x + dx,
            ..self
        }
    }

    /// `dz` cubes along z from here.
    pub const fn add_z(self, dz: i32) -> CubePos {
        CubePos {
            z: self.z + dz,
            ..self
        }
    }

    /// The same column of cubes at the height `y`.
    pub const fn with_y(self, y: i32) -> CubePos {
        CubePos { y, ..self }
    }

    /// Whether it lies at or below `other` along each of x, y and z.
    fn at_or_below(self, other: CubePos) -> bool {
        self.x <= other.x && self.y <= other.y && self.z <= other.z
    }
}

/// How many cubes apart along each of x, y and z.
impl Sub for CubePos {
    type Output = CubePos;

    fn sub(self, other: CubePos) -> CubePos {
        CubePos::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

value! {
    /// The cubes from `lo` to `hi`, both included. `lo` lies at or below `hi` along each of x, y
    /// and z: every way to make one sees to it.
    #[derive(Eq)]
    pub struct Cubes {
        lo: CubePos,
        hi: CubePos,
    }
}

impl Cubes {
    /// The cubes with the opposite corners `a` and `b`.
    pub fn between(a: CubePos, b: CubePos) -> Cubes {
        Cubes {
            lo: CubePos::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z)),
            hi: CubePos::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z)),
        }
    }

    /// The cubes up to `range` away from the one the block `pos` is in, along x and z, at its
    /// height.
    pub fn around(pos: Pos, range: i32) -> Cubes {
        let center = CubePos::of(pos);
        Cubes::between(
            center.add_x(-range).add_z(-range),
            center.add_x(range).add_z(range),
        )
    }

    /// The cubes the blocks from `a` to `b` are in, both included.
    pub fn spanning(a: Pos, b: Pos) -> Cubes {
        Cubes::between(CubePos::of(a), CubePos::of(b))
    }

    /// The lowest cube in every direction.
    pub fn lo(&self) -> CubePos {
        self.lo
    }

    /// The highest cube in every direction.
    pub fn hi(&self) -> CubePos {
        self.hi
    }

    pub fn count(&self) -> i32 {
        let span = self.hi - self.lo;
        (span.x + 1) * (span.y + 1) * (span.z + 1)
    }

    /// Whether every cube of `other` is one of these.
    pub fn contains(&self, other: &Cubes) -> bool {
        self.lo.at_or_below(other.lo) && other.hi.at_or_below(self.hi)
    }

    /// The first and the last block of the box, as `World::keep_loaded` takes it.
    pub fn blocks(&self) -> (Pos, Pos) {
        (self.lo.first_block(), self.hi.last_block())
    }

    /// Each cube.
    pub fn each(&self) -> impl Iterator<Item = CubePos> + use<> {
        let (lo, hi) = (self.lo, self.hi);
        (lo.y..=hi.y).flat_map(move |y| {
            (lo.z..=hi.z).flat_map(move |z| (lo.x..=hi.x).map(move |x| CubePos::new(x, y, z)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_below_zero_are_in_the_cube_below() {
        let cube = CubePos::of(Pos::new(-1, -16, 15));
        assert_eq!(cube, CubePos::new(-1, -1, 0), "-1 and -16 in -1, 15 in 0");
        assert_eq!(cube.first_block(), Pos::new(-16, -16, 0), "starts at");
        assert_eq!(cube.last_block(), Pos::new(-1, -1, 15), "ends at");
    }

    #[test]
    fn corners_given_either_way_round_make_the_same_box() {
        let (a, b) = (CubePos::new(3, -1, 0), CubePos::new(-2, 4, 0));
        let cubes = Cubes::between(a, b);
        assert_eq!(cubes, Cubes::between(b, a), "swapped");
        assert_eq!(cubes.lo(), CubePos::new(-2, -1, 0), "lowest of each");
        assert_eq!(cubes.hi(), CubePos::new(3, 4, 0), "highest of each");
        assert_eq!(cubes.count(), 36, "6 by 6, one deep");
    }

    #[test]
    fn a_range_spreads_along_x_and_z_only() {
        let cubes = Cubes::around(Pos::new(40, 70, 8), 3);
        assert_eq!(cubes.count(), 49, "7 by 7, one high");
        assert_eq!(
            cubes.lo(),
            CubePos::new(-1, 4, -3),
            "three each way of cube 2, 4, 0"
        );
        assert_eq!(cubes.hi(), CubePos::new(5, 4, 3), "and the same up");
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
        assert_eq!(cubes.lo(), CubePos::new(-2, 1, -1), "-17 is two cubes down");
        assert_eq!(
            cubes.hi(),
            CubePos::new(1, 2, 0),
            "33 is in the third cube up"
        );
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
        let all: Vec<CubePos> = cubes.each().collect();
        assert_eq!(all.len() as i32, cubes.count(), "as many as it counts");
        assert!(all.contains(&CubePos::new(-1, 0, 1)), "a corner cube");
    }
}
