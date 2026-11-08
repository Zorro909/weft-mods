//! Whether a structure fits the ground at a spot, and at what height. Worked out from the
//! terrain's columns alone, so every cube and every client gets the same answer.

use weft_sdk::terrain::Column;

use crate::biomes::Biomes;

/// The ground of column (`x`, `z`): `None` where nothing may stand (no terrain, a cave mouth).
pub(crate) type Columns<'a> = &'a dyn Fn(i32, i32) -> Option<Column>;

/// The conditions a spot must meet, shared by static structures and plans.
#[derive(Clone, Copy)]
pub struct Fit {
    biomes: Option<&'static Biomes>,
    max_slope: i32,
    sink: i32,
}

impl Default for Fit {
    fn default() -> Fit {
        Fit::new()
    }
}

impl Fit {
    /// Any biome, ground varying by at most 6 blocks, the bottom layer level with it.
    pub const fn new() -> Fit {
        Fit {
            biomes: None,
            max_slope: 6,
            sink: 0,
        }
    }
    /// Only where the biome under its middle is one of `biomes`.
    pub fn biome(mut self, biomes: &'static Biomes) -> Fit {
        self.biomes = Some(biomes);
        self
    }
    /// How much the ground under it may rise and fall, in blocks.
    pub fn max_slope(mut self, blocks: i32) -> Fit {
        self.max_slope = blocks;
        self
    }
    /// How much further down its bottom layer goes.
    pub fn buried(mut self, blocks: i32) -> Fit {
        self.sink = blocks;
        self
    }

    /// The height of the bottom layer of a footprint of `size` (x, z) with its lowest corner at
    /// `corner`, if the ground there meets the conditions. The bottom layer takes the place of
    /// the ground's top block, so a structure's own floor is level with the ground around it.
    pub(crate) fn floor(&self, corner: [i32; 2], size: [i32; 2], ground: Columns) -> Option<i32> {
        let fits_biome = |biome| self.biomes.is_none_or(|biomes| biomes.contains(biome));
        floor(self.max_slope, self.sink, corner, size, ground, &fits_biome)
    }
}

/// `Fit::floor` with the biome test passed in, so it runs without a game.
fn floor(
    max_slope: i32,
    sink: i32,
    [x, z]: [i32; 2],
    [sx, sz]: [i32; 2],
    ground: Columns,
    fits_biome: &dyn Fn(u32) -> bool,
) -> Option<i32> {
    if !fits_biome(ground(x + sx / 2, z + sz / 2)?.biome) {
        return None;
    }
    let heights: Option<Vec<i32>> = heights([x, z], [sx, sz], ground).into_iter().collect();
    let heights = heights?;
    let (lowest, highest) = (heights.iter().min()?, heights.iter().max()?);
    if highest - lowest > max_slope {
        return None;
    }
    Some(median(&heights) - sink)
}

/// The ground height of each column of a footprint, x fastest, then z.
pub(crate) fn heights([x, z]: [i32; 2], [sx, sz]: [i32; 2], ground: Columns) -> Vec<Option<i32>> {
    (0..sz)
        .flat_map(|dz| (0..sx).map(move |dx| (dx, dz)))
        .map(|(dx, dz)| ground(x + dx, z + dz).map(|g| g.height))
        .collect()
}

/// Half the ground is at or below it: the structure meets the ground in its middle, with as
/// much foundation below as cut above.
fn median(heights: &[i32]) -> i32 {
    let mut sorted = heights.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(height: i32) -> impl Fn(i32, i32) -> Option<Column> {
        move |_, _| Some(Column { height, biome: 1 })
    }

    fn anywhere(_: u32) -> bool {
        true
    }

    #[test]
    fn flat_ground_is_stood_on() {
        let y = floor(4, 0, [3, 3], [5, 5], &flat(70), &anywhere);
        assert_eq!(
            y,
            Some(70),
            "the floor takes the place of the top block, level with it"
        );
        let buried = floor(4, 2, [3, 3], [5, 5], &flat(70), &anywhere);
        assert_eq!(buried, Some(68), "two layers further down");
    }

    #[test]
    fn the_floor_is_the_median_of_the_ground() {
        let slope = |x: i32, _| {
            Some(Column {
                height: 60 + x.rem_euclid(5),
                biome: 1,
            })
        };
        let y = floor(4, 0, [0, 0], [5, 1], &slope, &anywhere);
        assert_eq!(y, Some(62), "62 is the middle of 60..=64");
    }

    #[test]
    fn steep_ground_does_not_fit() {
        let cliff = |x: i32, _| {
            Some(Column {
                height: 60 + 5 * x.rem_euclid(2),
                biome: 1,
            })
        };
        let y = floor(4, 0, [0, 0], [5, 5], &cliff, &anywhere);
        assert_eq!(y, None, "5 blocks of slope, 4 allowed");
    }

    #[test]
    fn a_hole_in_the_ground_does_not_fit() {
        let cave = |x: i32, z: i32| {
            (x != 3 || z != 3).then_some(Column {
                height: 60,
                biome: 1,
            })
        };
        assert_eq!(
            floor(4, 0, [0, 0], [5, 5], &cave, &anywhere),
            None,
            "the cave mouth is under it"
        );
        assert!(
            floor(4, 0, [4, 0], [5, 5], &cave, &anywhere).is_some(),
            "next to it is fine"
        );
    }

    #[test]
    fn another_biome_does_not_fit() {
        let y = floor(4, 0, [0, 0], [5, 5], &flat(70), &|_| false);
        assert_eq!(y, None, "the biome is asked");
    }
}
