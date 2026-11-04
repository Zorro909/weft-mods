//! A block position with named axes. The SDK takes and gives `[i32; 3]`; `Pos` turns into one and
//! back with `into` and `from`.

use std::ops::Add;

use weft_sdk::*;

value! {
    /// A block's position, or an offset from one. Laid out as the `[i32; 3]` the SDK uses.
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
    fn from(p: Pos) -> [i32; 3] {
        [p.x, p.y, p.z]
    }
}

/// The block's lowest corner, as a point.
impl From<Pos> for [f64; 3] {
    fn from(p: Pos) -> [f64; 3] {
        [p.x, p.y, p.z].map(f64::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_turns_into_the_sdks_array_and_back() {
        let p = Pos::new(4, -5, 6);
        let array: [i32; 3] = p.into();
        assert_eq!(array, [4, -5, 6], "x, y, z in order");
        assert_eq!(Pos::from(array), p, "and back");
        assert_eq!(<[f64; 3]>::from(p), [4.0, -5.0, 6.0], "as a point");
    }

    #[test]
    fn offsets_add_axis_by_axis() {
        assert_eq!(
            Pos::new(10, 64, -3) + Pos::new(2, 0, -1),
            Pos::new(12, 64, -4)
        );
    }
}
