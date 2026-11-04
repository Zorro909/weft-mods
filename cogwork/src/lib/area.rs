//! A rectangle on the ground: what survey markers mark out, an excavator's frame stands on and
//! a leveler makes flat.

use weft_sdk::*;

use super::Pos;

value! {
    /// A rectangle on the ground, by its lowest and highest corner at the same height.
    #[derive(Default)]
    pub struct Area {
        pub lo: Pos,
        pub hi: Pos,
    }
}

impl Area {
    /// The rectangle with the opposite corners `a` and `b`, at `a`'s height.
    pub fn between(a: Pos, b: Pos) -> Area {
        Area {
            lo: Pos::new(a.x.min(b.x), a.y, a.z.min(b.z)),
            hi: Pos::new(a.x.max(b.x), a.y, a.z.max(b.z)),
        }
    }

    /// Grows the area to take in `p`'s column.
    pub fn grow_to(&mut self, p: Pos) {
        let (lo, hi) = (self.lo, self.hi);
        self.lo = Pos::new(lo.x.min(p.x), lo.y, lo.z.min(p.z));
        self.hi = Pos::new(hi.x.max(p.x), hi.y, hi.z.max(p.z));
    }

    /// Whether `p` lies on, over or under the area, its edges included.
    pub fn spans(&self, p: Pos) -> bool {
        (self.lo.x..=self.hi.x).contains(&p.x) && (self.lo.z..=self.hi.z).contains(&p.z)
    }

    /// Blocks along x and along z, its edges included.
    pub fn size(&self) -> [i32; 2] {
        [self.hi.x - self.lo.x + 1, self.hi.z - self.lo.z + 1]
    }

    /// Its four corners, at its height.
    pub fn corners(&self) -> [Pos; 4] {
        let (lo, hi) = (self.lo, self.hi);
        [
            lo,
            Pos { x: hi.x, ..lo },
            Pos { z: hi.z, ..lo },
            hi.with_y(lo.y),
        ]
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
        let a = Area::between(Pos::new(5, 2, -3), Pos::new(-1, 9, 4));
        assert_eq!(
            a,
            area([-1, 2, -3], [5, 2, 4]),
            "at the first corner's height"
        );
        assert_eq!(a.size(), [7, 8], "edges included");
        let corners = a.corners().map(|c| [c.x, c.z]);
        assert_eq!(
            corners,
            [[-1, -3], [5, -3], [-1, 4], [5, 4]],
            "four corners"
        );
        assert!(
            corners.iter().all(|&[x, z]| a.spans(Pos::new(x, 99, z))),
            "spanned"
        );
    }

    #[test]
    fn an_area_grows_to_take_in_a_column() {
        let mut a = area([0, 0, 0], [2, 0, 2]);
        a.grow_to(Pos::new(-3, 7, 5));
        assert_eq!(a, area([-3, 0, 0], [2, 0, 5]), "out both ways, not up");
        assert!(a.spans(Pos::new(-3, 99, 5)), "a corner, at any height");
        assert!(!a.spans(Pos::new(3, 0, 0)), "not past its edge");
    }
}
