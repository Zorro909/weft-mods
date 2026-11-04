//! A block position with named axes, for code that works positions out. The SDK takes and gives
//! `[i32; 3]`; `Pos` turns into one and back with `into` and `from`.

use weft_sdk::*;

value! {
    /// A block's position. Laid out as the `[i32; 3]` the SDK uses, so a saved `[i32; 3]` loads
    /// as one.
    #[derive(Default, Eq, Hash)]
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
        let [dx, dy, dz] = dir.offset();
        self.add_x(dx * blocks)
            .add_y(dy * blocks)
            .add_z(dz * blocks)
    }

    /// The block next to this one across its face `face` (in `Dir` order). Tubes, engines,
    /// shafts and machines all hand things to their neighbors through it.
    pub fn neighbor(self, face: u32) -> Pos {
        self.toward(Dir::from_face(face))
    }

    /// The middle of this block, in the world.
    pub fn center(self) -> [f64; 3] {
        block_center(self.into())
    }

    /// The block the point `at` in the world lies in.
    pub fn containing(at: [f64; 3]) -> Pos {
        at.map(|v| v.floor() as i32).into()
    }
}

impl From<[i32; 3]> for Pos {
    fn from([x, y, z]: [i32; 3]) -> Pos {
        Pos { x, y, z }
    }
}

impl From<Pos> for [i32; 3] {
    fn from(p: Pos) -> [i32; 3] {
        [p.x, p.y, p.z]
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
        assert_eq!(
            p.neighbor(Dir::Up.face()),
            Pos::new(1, 3, 3),
            "across the top"
        );
    }

    #[test]
    fn a_point_in_the_world_lies_in_one_block() {
        let block = Pos::new(-2, 3, 0);
        assert_eq!(Pos::containing(block.center()), block, "its middle");
        assert_eq!(Pos::containing([-1.5, 3.0, 0.99]), block, "its edges");
    }

    #[test]
    fn it_turns_into_the_sdks_array_and_back() {
        let p = Pos::new(4, -5, 6);
        let array: [i32; 3] = p.into();
        assert_eq!(array, [4, -5, 6], "x, y, z in order");
        assert_eq!(Pos::from(array), p, "and back");
        let size = std::mem::size_of::<Pos>();
        assert_eq!(size, std::mem::size_of::<[i32; 3]>(), "saved alike");
    }
}
