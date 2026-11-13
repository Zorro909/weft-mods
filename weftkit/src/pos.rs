//! A block position that can be saved and sent, for code that works positions out. The SDK
//! takes and gives `BlockPos`, which is no `Value`; `Pos` goes to SDK calls as it is and comes
//! back from them with `from`.

use std::ops::Add;

use weft_sdk::{BlockPos, Dir, Holder, Vec3, value};

value! {
    /// A block's position, or an offset from one. Laid out as `[i32; 3]`, so a saved or sent
    /// `[i32; 3]` reads as one.
    #[derive(Default, Eq, Hash)]
    #[cfg_attr(feature = "store", derive(serde::Serialize, serde::Deserialize))]
    pub struct Pos {
        pub x: i32,
        pub y: i32,
        pub z: i32,
    }
}

impl Pos {
    pub const fn new(x: i32, y: i32, z: i32) -> Pos {
        Pos { x, y, z }
    }

    /// `dx` blocks along x from here.
    pub const fn add_x(self, dx: i32) -> Pos {
        Pos {
            x: self.x + dx,
            ..self
        }
    }

    /// `dy` blocks up from here.
    pub const fn add_y(self, dy: i32) -> Pos {
        Pos {
            y: self.y + dy,
            ..self
        }
    }

    /// `dz` blocks along z from here.
    pub const fn add_z(self, dz: i32) -> Pos {
        Pos {
            z: self.z + dz,
            ..self
        }
    }

    /// The lowest and the highest corner of the box with `a` and `b` at opposite corners, both
    /// taken axis by axis.
    pub fn corners(a: Pos, b: Pos) -> (Pos, Pos) {
        let lowest = Pos::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
        let highest = Pos::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));
        (lowest, highest)
    }

    /// The same column at the height `y`.
    pub const fn with_y(self, y: i32) -> Pos {
        Pos { y, ..self }
    }

    /// The block next to this one toward `dir`.
    pub fn toward(self, dir: Dir) -> Pos {
        self.toward_by(dir, 1)
    }

    /// `blocks` blocks from here toward `dir`; away from it for a negative number.
    pub fn toward_by(self, dir: Dir, blocks: i32) -> Pos {
        let step = dir.offset();
        self.add_x(step.x * blocks)
            .add_y(step.y * blocks)
            .add_z(step.z * blocks)
    }

    /// The middle of this block.
    pub fn center(self) -> Vec3 {
        BlockPos::from(self).center()
    }

    /// The middle of this block's floor, where something stands on it.
    pub fn floor_middle(self) -> Vec3 {
        self.center().offset(0.0, -0.5, 0.0)
    }
}

impl Add for Pos {
    type Output = Pos;

    fn add(self, by: Pos) -> Pos {
        Pos::new(self.x + by.x, self.y + by.y, self.z + by.z)
    }
}

impl From<[i32; 3]> for Pos {
    fn from([x, y, z]: [i32; 3]) -> Pos {
        Pos { x, y, z }
    }
}

impl From<Pos> for [i32; 3] {
    fn from(Pos { x, y, z }: Pos) -> [i32; 3] {
        [x, y, z]
    }
}

impl From<BlockPos> for Pos {
    fn from(BlockPos { x, y, z }: BlockPos) -> Pos {
        Pos { x, y, z }
    }
}

impl From<Pos> for BlockPos {
    fn from(Pos { x, y, z }: Pos) -> BlockPos {
        BlockPos { x, y, z }
    }
}

/// The block's lowest corner in every direction, as a point in the world.
impl From<Pos> for Vec3 {
    fn from(Pos { x, y, z }: Pos) -> Vec3 {
        Vec3::new(x as f64, y as f64, z as f64)
    }
}

/// The block's lowest corner, as a point.
impl From<Pos> for [f64; 3] {
    fn from(pos: Pos) -> [f64; 3] {
        Vec3::from(pos).to_array()
    }
}

/// The block at `pos`, as the holder of its block entity's inventory and components.
impl From<Pos> for Holder {
    fn from(pos: Pos) -> Holder {
        Holder::block(pos.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_axis_moves_on_its_own() {
        let p = Pos::new(1, 2, 3);
        assert_eq!(p.add_x(-2), Pos::new(-1, 2, 3), "along x");
        assert_eq!(p.add_y(5), Pos::new(1, 7, 3), "up");
        assert_eq!(p.add_z(1), Pos::new(1, 2, 4), "along z");
        assert_eq!(p.with_y(-64), Pos::new(1, -64, 3), "same column");
        assert_eq!(p.toward(Dir::Down), Pos::new(1, 1, 3), "one block down");
        assert_eq!(p.toward_by(Dir::East, -4), Pos::new(-3, 2, 3), "four west");
    }

    #[test]
    fn a_point_in_the_world_lies_in_one_block() {
        let block = Pos::new(-2, 3, 0);
        assert_eq!(block.center(), Vec3::new(-1.5, 3.5, 0.5), "its middle");
        assert_eq!(Pos::from(block.center().block()), block, "lies in it");
        assert_eq!(Pos::from(block.floor_middle().block()), block, "its floor");
        assert_eq!(Vec3::from(block), Vec3::new(-2.0, 3.0, 0.0), "its corner");
        let edge = Vec3::new(-1.5, 3.0, 0.99);
        assert_eq!(Pos::from(edge.block()), block, "near its edges");
    }

    #[test]
    fn offsets_add_axis_by_axis() {
        let moved = Pos::new(10, 64, -3) + Pos::new(2, 0, -1);
        assert_eq!(moved, Pos::new(12, 64, -4));
    }

    #[test]
    fn corners_come_lowest_first_axis_by_axis() {
        let (lowest, highest) = Pos::corners(Pos::new(5, 64, -2), Pos::new(1, 70, -9));
        assert_eq!(lowest, Pos::new(1, 64, -9), "each axis's lowest");
        assert_eq!(highest, Pos::new(5, 70, -2), "each axis's highest");
        let one = Pos::new(3, 3, 3);
        assert_eq!(Pos::corners(one, one), (one, one), "a box of one block");
    }

    #[test]
    fn it_turns_into_arrays_and_back() {
        let p = Pos::new(4, -5, 6);
        let array: [i32; 3] = p.into();
        assert_eq!(array, [4, -5, 6], "x, y, z in order");
        assert_eq!(Pos::from(array), p, "and back");
        assert_eq!(<[f64; 3]>::from(p), [4.0, -5.0, 6.0], "as a point");
    }

    #[test]
    fn it_turns_into_the_sdks_position_and_back() {
        let p = Pos::new(4, -5, 6);
        let sdk = BlockPos::from(p);
        assert_eq!(sdk, BlockPos::new(4, -5, 6), "x, y, z in order");
        assert_eq!(Pos::from(sdk), p, "and back");
        let size = std::mem::size_of::<Pos>();
        assert_eq!(size, std::mem::size_of::<[i32; 3]>(), "saved alike");
    }
}
