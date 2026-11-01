//! Where a structure stands: one chance per grid cell, a spot in the cell from the seed, and a
//! floor fitted to the ground under it. Worked out from the seed and the terrain's columns
//! alone, so every chunk, and every client, finds the same sites without reading blocks.

use std::ops::RangeInclusive;

use weft_sdk::noise;

/// How a structure finds its place, fixed when it is registered.
#[derive(Clone, Debug)]
pub struct Rules {
    /// Gives each structure a grid of its own.
    pub salt: u64,
    /// The cell size in blocks: at most one site per cell.
    pub spacing: i32,
    /// How likely a cell is to have a site, out of 65536.
    pub chance: u32,
    /// How much the ground under the footprint may rise and fall, in blocks.
    pub max_slope: i32,
    /// How far the bottom layer goes into the ground.
    pub sink: i32,
}

/// What generation knows about a column without reading any block.
#[derive(Clone, Copy, Debug)]
pub struct Ground {
    /// The top solid block.
    pub height: i32,
    pub biome: u32,
}

/// Where one structure stands.
#[derive(Debug, PartialEq)]
pub struct Site {
    /// Where the structure's lowest corner goes.
    pub corner: [i32; 3],
    /// The ground height of each column under it, x fastest, then z.
    pub ground: Vec<i32>,
}

/// The site in grid cell `cell` of a structure covering `footprint` (x, z), if it has one.
/// `ground` is `None` where nothing may stand (no terrain, a cave mouth); `fits_biome` is asked
/// about the column under the middle.
pub fn site_in(
    rules: &Rules,
    footprint: [i32; 2],
    seed: u64,
    cell: [i32; 2],
    ground: &dyn Fn(i32, i32) -> Option<Ground>,
    fits_biome: &dyn Fn(u32) -> bool,
) -> Option<Site> {
    let h = noise::hash(seed ^ rules.salt, cell[0] as i64, 0, cell[1] as i64);
    if (h >> 48) as u32 >= rules.chance {
        return None;
    }
    let [sx, sz] = footprint;
    let room = (rules.spacing - sx.max(sz)).max(1) as u64;
    let x = cell[0] * rules.spacing + (h % room) as i32;
    let z = cell[1] * rules.spacing + ((h >> 24) % room) as i32;
    if !fits_biome(ground(x + sx / 2, z + sz / 2)?.biome) {
        return None;
    }
    let heights = footprint_heights(x, z, footprint, ground)?;
    let (lowest, highest) = (heights.iter().min()?, heights.iter().max()?);
    if highest - lowest > rules.max_slope {
        return None;
    }
    let floor = median(&heights);
    Some(Site {
        corner: [x, floor + 1 - rules.sink, z],
        ground: heights,
    })
}

/// The cells whose structure may reach into blocks `from..=to` of one axis, for a structure
/// `size` blocks long: it stays inside its cell unless it is longer than the cell.
pub fn cells(from: i32, to: i32, size: i32, spacing: i32) -> RangeInclusive<i32> {
    (from - size).div_euclid(spacing)..=to.div_euclid(spacing)
}

fn footprint_heights(
    x: i32,
    z: i32,
    [sx, sz]: [i32; 2],
    ground: &dyn Fn(i32, i32) -> Option<Ground>,
) -> Option<Vec<i32>> {
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

    fn rules() -> Rules {
        Rules {
            salt: 7,
            spacing: 32,
            chance: 65536,
            max_slope: 4,
            sink: 0,
        }
    }

    fn flat(height: i32) -> impl Fn(i32, i32) -> Option<Ground> {
        move |_, _| Some(Ground { height, biome: 1 })
    }

    fn anywhere(_: u32) -> bool {
        true
    }

    #[test]
    fn a_site_stands_on_flat_ground() {
        let site = site_in(&rules(), [5, 5], 1, [2, -3], &flat(70), &anywhere).expect("a site");
        assert_eq!(site.corner[1], 71, "the bottom layer on top of the ground");
        assert!(
            (64..64 + 32 - 5).contains(&site.corner[0]),
            "inside its cell along x"
        );
        assert!(
            (-96..-96 + 32 - 5).contains(&site.corner[2]),
            "inside its cell along z"
        );
    }

    #[test]
    fn the_same_cell_gives_the_same_site() {
        let first = site_in(&rules(), [5, 5], 1, [4, 4], &flat(70), &anywhere);
        assert_eq!(
            first,
            site_in(&rules(), [5, 5], 1, [4, 4], &flat(70), &anywhere),
            "pure"
        );
    }

    #[test]
    fn the_floor_is_the_median_of_the_ground() {
        let slope = |x: i32, _| {
            Some(Ground {
                height: 60 + x.rem_euclid(5),
                biome: 1,
            })
        };
        let site = site_in(&rules(), [5, 1], 1, [0, 0], &slope, &anywhere).expect("a site");
        assert_eq!(site.corner[1], 63, "62 is the middle of 60..=64");
    }

    #[test]
    fn steep_ground_has_no_site() {
        let cliff = |x: i32, _| {
            Some(Ground {
                height: 60 + 5 * x.rem_euclid(2),
                biome: 1,
            })
        };
        assert_eq!(
            site_in(&rules(), [5, 5], 1, [0, 0], &cliff, &anywhere),
            None,
            "5 blocks of slope, 4 allowed"
        );
    }

    #[test]
    fn a_hole_in_the_ground_has_no_site() {
        let cave = |x: i32, z: i32| {
            (x.rem_euclid(32) != 3 || z.rem_euclid(32) != 3).then_some(Ground {
                height: 60,
                biome: 1,
            })
        };
        let blocked = (0..200)
            .filter(|&i| site_in(&rules(), [32, 32], 1, [i, 0], &cave, &anywhere).is_some())
            .count();
        assert_eq!(blocked, 0, "every footprint covers the cave mouth");
    }

    #[test]
    fn another_biome_has_no_site() {
        assert_eq!(
            site_in(&rules(), [5, 5], 1, [0, 0], &flat(70), &|_| false),
            None,
            "the biome is asked"
        );
    }

    #[test]
    fn chance_thins_the_cells_out() {
        let rules = Rules {
            chance: 65536 / 4,
            ..rules()
        };
        let sites = (0..1000)
            .filter(|&i| site_in(&rules, [5, 5], 1, [i, 0], &flat(70), &anywhere).is_some())
            .count();
        assert!((180..320).contains(&sites), "about a quarter, got {sites}");
    }

    #[test]
    fn cells_cover_every_reach() {
        assert_eq!(cells(0, 15, 5, 32), -1..=0, "the cell before may reach in");
        assert_eq!(cells(64, 79, 40, 32), 0..=2, "longer than a cell");
    }
}
