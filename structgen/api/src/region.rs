//! The coarse grid structures are planned on: one plan per square region, which every cube in
//! it works out alike.

use std::ops::RangeInclusive;

use weftkit::Pos;

/// A square of the world one plan may fill, in blocks, both corners included. Plans may reach
/// any height.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub min: [i32; 2],
    pub max: [i32; 2],
}

impl Region {
    /// Region `index` of a grid of `size`-block squares.
    pub(crate) fn at([ix, iz]: [i32; 2], size: i32) -> Region {
        let min = [ix * size, iz * size];
        Region {
            min,
            max: [min[0] + size - 1, min[1] + size - 1],
        }
    }

    pub fn size(&self) -> i32 {
        self.max[0] - self.min[0] + 1
    }

    pub fn contains(&self, x: i32, z: i32) -> bool {
        (self.min[0]..=self.max[0]).contains(&x) && (self.min[1]..=self.max[1]).contains(&z)
    }

    /// Whether the box from `from` to `to` (included) lies inside, seen from above.
    pub fn holds(&self, from: Pos, to: Pos) -> bool {
        self.contains(from.x, from.z) && self.contains(to.x, to.z)
    }

    /// The squared distance from (`x`, `z`) to the nearest column of the region.
    fn distance_squared(&self, x: i32, z: i32) -> i64 {
        let dx = (self.min[0] - x).max(x - self.max[0]).max(0) as i64;
        let dz = (self.min[1] - z).max(z - self.max[1]).max(0) as i64;
        dx * dx + dz * dz
    }
}

/// The grid indices of the regions that blocks `from..=to` along one axis touch.
pub(crate) fn indices(from: i32, to: i32, size: i32) -> RangeInclusive<i32> {
    from.div_euclid(size)..=to.div_euclid(size)
}

/// The regions within `radius` blocks of (`x`, `z`), nearest first: each as its index and the
/// squared distance to its nearest column.
pub(crate) fn around(x: i32, z: i32, radius: i32, size: i32) -> Vec<([i32; 2], i64)> {
    let mut found: Vec<([i32; 2], i64)> = indices(z - radius, z + radius, size)
        .flat_map(|iz| indices(x - radius, x + radius, size).map(move |ix| [ix, iz]))
        .map(|index| (index, Region::at(index, size).distance_squared(x, z)))
        .filter(|&(_, distance)| distance <= radius as i64 * radius as i64)
        .collect();
    found.sort_by_key(|&(_, distance)| distance);
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_tile_the_world() {
        let region = Region::at([-1, 2], 100);
        assert_eq!(region.min, [-100, 200], "lowest corner");
        assert_eq!(region.max, [-1, 299], "highest corner, included");
        assert!(
            region.contains(-1, 200) && !region.contains(0, 200),
            "ends at its edge"
        );
    }

    #[test]
    fn a_cube_may_touch_two_regions() {
        assert_eq!(indices(0, 15, 64), 0..=0, "inside one");
        assert_eq!(indices(16, 31, 24), 0..=1, "across an edge");
        assert_eq!(indices(-16, -1, 64), -1..=-1, "below zero");
    }

    #[test]
    fn a_box_holds_only_when_all_of_it_is_inside() {
        let region = Region::at([0, 0], 32);
        let (low, high) = (Pos::new(0, -50, 0), Pos::new(31, 300, 31));
        assert!(region.holds(low, high), "any height");
        assert!(
            !region.holds(Pos::new(20, 0, 20), Pos::new(32, 0, 25)),
            "one block over the edge"
        );
    }

    #[test]
    fn nearest_regions_come_first() {
        let found = around(10, 10, 40, 32);
        assert_eq!(found[0], ([0, 0], 0), "the one it stands in");
        assert!(
            found.windows(2).all(|w| w[0].1 <= w[1].1),
            "sorted by distance"
        );
        assert!(
            !found.iter().any(|&(index, _)| index == [2, 2]),
            "too far away"
        );
    }
}
