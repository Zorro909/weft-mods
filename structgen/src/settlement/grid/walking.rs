//! Whether the grid's ground can be walked: along a way, or all across an area.

use super::*;

impl Grid {
    /// The height at the end of a walk of `blocks` blocks from `from` along `way`, if every
    /// column on the way has ground and no building, and none is more than a block above or
    /// below the last.
    pub fn walkable(&self, from: Point, way: Point, blocks: i32) -> Option<i32> {
        let mut last = self.ground(from)?;
        for k in 1..=blocks {
            let next = self.ground(add(from, scale(way, k as f32)))?;
            if (next - last).abs() > 1 {
                return None;
            }
            last = next;
        }
        Some(last)
    }

    /// The height of the ground under `at`, if it can be walked on: no water, no building.
    pub fn ground(&self, at: Point) -> Option<i32> {
        let at = column(at);
        self.height(at).filter(|_| self.get(at) != Cell::Building)
    }

    /// Whether every column inside `area`, or within `margin` of its edge, has ground no more
    /// than a block above or below its neighbours there and `rise` above the lowest: a heart a
    /// player can walk across, not a hillside of steps.
    pub fn even(&self, area: &[Point], margin: f32, rise: i32) -> bool {
        let heights = self.heights(area, margin);
        let Some(ground) = heights.iter().map(|&(h, _)| h).collect::<Option<Vec<_>>>() else {
            return false;
        };
        let (low, high) = (ground.iter().min(), ground.iter().max());
        no_ledge(&heights) && high.zip(low).is_none_or(|(h, l)| h - l <= rise)
    }

    /// Whether no column inside `area`, or within `margin` of its edge, has ground more than a
    /// block above or below its neighbours there; water and land left out don't count.
    pub fn level(&self, area: &[Point], margin: f32) -> bool {
        no_ledge(&self.heights(area, margin))
    }

    /// The ground's height in each column inside `area`, or within `margin` of its edge, and in
    /// its neighbours east and south among them.
    fn heights(&self, area: &[Point], margin: f32) -> Vec<(Option<i32>, [Option<i32>; 2])> {
        let mut edge = area.to_vec();
        edge.push(area[0]);
        let within =
            |at: [i32; 2]| inside(area, middle(at)) || distance(&edge, middle(at)) <= margin;
        let (from, to) = bounds(area, margin + 1.0);
        let columns = (from[1]..=to[1]).flat_map(|z| (from[0]..=to[0]).map(move |x| [x, z]));
        let height = |at: [i32; 2]| self.height(at).filter(|_| within(at));
        let next = |[x, z]: [i32; 2]| [[x + 1, z], [x, z + 1]].map(height);
        columns
            .filter(|&at| within(at))
            .map(|at| (self.height(at), next(at)))
            .collect()
    }
}

/// Whether no column in `heights` is more than a block above or below a neighbour.
fn no_ledge(heights: &[(Option<i32>, [Option<i32>; 2])]) -> bool {
    heights.iter().all(|&(here, next)| {
        let step = |there: Option<i32>| here.zip(there).is_some_and(|(a, b)| (a - b).abs() > 1);
        !next.into_iter().any(step)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_walk_climbs_a_block_at_a_time_and_stops_at_water_or_a_ledge() {
        let east = [1.0, 0.0];
        let gentle = row(&[Some(64), Some(65), Some(66), Some(65)]);
        assert_eq!(gentle.walkable([0.5, 0.5], east, 3), Some(65));
        let ledge = row(&[Some(64), Some(64), Some(66), Some(66)]);
        assert_eq!(
            ledge.walkable([0.5, 0.5], east, 3),
            None,
            "two blocks up at once"
        );
        let stream = row(&[Some(64), None, Some(64), Some(64)]);
        assert_eq!(
            stream.walkable([0.5, 0.5], east, 3),
            None,
            "water between dry ends"
        );
        let mut house = row(&[Some(64); 4]);
        house.put([2, 0], Cell::Building);
        assert_eq!(
            house.walkable([0.5, 0.5], east, 3),
            None,
            "no way through a house"
        );
    }

    #[test]
    fn a_heart_on_a_ledge_or_water_is_uneven() {
        let strip = [[0.0, 0.0], [4.0, 0.0], [4.0, 1.0], [0.0, 1.0]];
        let gentle = row(&[Some(64), Some(65), Some(65), Some(66), Some(66)]);
        assert!(gentle.even(&strip, 0.0, 2), "a block at a time");
        assert!(!gentle.even(&strip, 0.0, 1), "but two up in all");
        let ledge = row(&[Some(64), Some(64), Some(66), Some(66), Some(66)]);
        assert!(!ledge.even(&strip, 0.0, 2), "two blocks up at once");
        let pond = row(&[Some(64), None, Some(64), Some(64), Some(64)]);
        assert!(!pond.even(&strip, 0.0, 2), "water in it");
    }

    #[test]
    fn a_way_past_water_is_level_but_not_one_up_a_ledge() {
        let strip = [[0.0, 0.0], [3.0, 0.0], [3.0, 1.0], [0.0, 1.0]];
        let shore = row(&[Some(64), None, Some(65)]);
        assert!(shore.level(&strip, 0.0), "the water is left out");
        assert!(!shore.even(&strip, 0.0, 9), "but no heart stands in it");
        let ledge = row(&[Some(64), Some(66), Some(66)]);
        assert!(!ledge.level(&strip, 0.0), "two blocks up at once");
    }
}
