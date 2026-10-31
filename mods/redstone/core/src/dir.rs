/// A block position.
pub type Pos = [i32; 3];

/// A direction in Weft's face order (`-X +X -Y +Y -Z +Z`), so `index()` is the face index.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Dir {
    West,
    East,
    Down,
    Up,
    North,
    South,
}

impl Dir {
    pub const ALL: [Dir; 6] = [Dir::West, Dir::East, Dir::Down, Dir::Up, Dir::North, Dir::South];
    pub const HORIZONTAL: [Dir; 4] = [Dir::West, Dir::East, Dir::North, Dir::South];

    pub fn index(self) -> usize {
        self as usize
    }
    pub fn from_index(i: usize) -> Dir {
        Dir::ALL[i % 6]
    }
    pub fn opposite(self) -> Dir {
        Dir::from_index(self.index() ^ 1)
    }
    pub fn offset(self) -> [i32; 3] {
        [[-1, 0, 0], [1, 0, 0], [0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1]][self.index()]
    }
    /// A quarter turn clockwise seen from above (north → east → south → west).
    pub fn rotate_y(self) -> Dir {
        match self {
            Dir::North => Dir::East,
            Dir::East => Dir::South,
            Dir::South => Dir::West,
            Dir::West => Dir::North,
            d => d,
        }
    }
}

pub fn step(p: Pos, d: Dir) -> Pos {
    let o = d.offset();
    [p[0] + o[0], p[1] + o[1], p[2] + o[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_pairs() {
        for d in Dir::ALL {
            assert_eq!(d.opposite().opposite(), d);
            assert_eq!(d.opposite().index(), d.index() ^ 1);
        }
    }

    #[test]
    fn face_order_matches_weft() {
        assert_eq!(Dir::ALL.map(|d| d.offset()), [[-1, 0, 0], [1, 0, 0], [0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1]]);
    }

    #[test]
    fn rotate_four_times_is_identity() {
        for d in Dir::ALL {
            assert_eq!(d.rotate_y().rotate_y().rotate_y().rotate_y(), d);
        }
        assert_eq!(Dir::North.rotate_y(), Dir::East);
    }
}
