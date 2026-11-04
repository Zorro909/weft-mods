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
        let [dx, dy, dz] = dir.offset();
        self.add_x(dx).add_y(dy).add_z(dz)
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
