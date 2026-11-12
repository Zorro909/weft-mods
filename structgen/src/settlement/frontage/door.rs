//! Where a building's door is: which way it turns to face a road, where its corner goes for
//! its front to land on a column, and the column in front of its door.

use crate::settlement::grid::{Grid, middle};
use crate::settlement::shape::Point;
use crate::turn::Turn;

/// The turn that points a building's front, north as saved, back along `out` toward the road.
pub fn facing(out: Point) -> Turn {
    match (out[1].abs() >= out[0].abs(), out[0] > 0.0, out[1] > 0.0) {
        (true, _, true) => Turn::None,
        (true, _, false) => Turn::Back,
        (false, true, _) => Turn::Left,
        (false, false, _) => Turn::Right,
    }
}

/// The north-west corner of a building of `size` whose front, turned by `turn`, has its middle
/// at column `front`.
pub fn corner_for(front: [i32; 2], [sx, _, sz]: [i32; 3], turn: Turn) -> [i32; 2] {
    let [x, z] = front;
    match turn {
        Turn::None => [x - sx / 2, z],
        Turn::Back => [x - sx / 2, z - sz + 1],
        Turn::Right => [x - sx + 1, z - sz / 2],
        Turn::Left => [x, z - sz / 2],
    }
}

/// The height of the `door` column in front of a building whose front stands `length` blocks
/// out from `at` along `out`, if a walk from `at` leads onto it. A way out at a slant ends
/// beside the door, not always in front of it.
pub fn doorstep(grid: &Grid, at: Point, out: Point, length: i32, door: [i32; 2]) -> Option<i32> {
    let end = grid.walkable(at, out, (length - 1).max(0))?;
    grid.ground(middle(door)).filter(|y| (y - end).abs() <= 1)
}

/// The column in front of the middle of a front at `face`, turned by `turn`.
pub fn outside([x, z]: [i32; 2], turn: Turn) -> [i32; 2] {
    match turn {
        Turn::None => [x, z - 1],
        Turn::Back => [x, z + 1],
        Turn::Right => [x + 1, z],
        Turn::Left => [x - 1, z],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settlement::grid::row;

    #[test]
    fn a_building_turns_its_front_to_the_road() {
        assert_eq!(
            facing([0.0, 1.0]),
            Turn::None,
            "south of the road: front north"
        );
        assert_eq!(facing([0.0, -1.0]), Turn::Back, "north of it: front south");
        assert_eq!(facing([1.0, 0.2]), Turn::Left, "east of it: front west");
        assert_eq!(facing([-1.0, 0.0]), Turn::Right, "west of it: front east");
    }

    #[test]
    fn the_front_lands_on_the_column_asked_for() {
        let size = [7, 9, 5];
        assert_eq!(
            corner_for([10, 20], size, Turn::None),
            [7, 20],
            "front row is z 20"
        );
        assert_eq!(
            corner_for([10, 20], size, Turn::Back),
            [7, 16],
            "back row is z 16"
        );
        assert_eq!(
            corner_for([10, 20], size, Turn::Left),
            [10, 18],
            "front column is x 10"
        );
        assert_eq!(
            corner_for([10, 20], size, Turn::Right),
            [4, 18],
            "front column is x 10"
        );
    }

    #[test]
    fn a_slanted_way_must_reach_the_column_in_front_of_the_door() {
        let mut grid = row(&[Some(64); 6]);
        // Out at 45 degrees from the corner, the way ends at (2, 2) and the front at (3, 3).
        for at in [[1, 1], [2, 2], [3, 2]] {
            grid.raise(at, 64);
        }
        let out = [0.71, 0.71];
        let (at, door) = ([0.5, 0.5], outside([3, 3], facing(out)));
        assert_eq!(door, [3, 2], "beside the way's end at (2, 2)");
        assert_eq!(doorstep(&grid, at, out, 4, door), Some(64));
        grid.raise(door, 66);
        assert_eq!(
            doorstep(&grid, at, out, 4, door),
            None,
            "a step up to the door"
        );
    }

    #[test]
    fn the_door_opens_onto_the_column_just_outside_the_front() {
        let size = [7, 9, 5];
        for turn in [Turn::None, Turn::Back, Turn::Left, Turn::Right] {
            let [x, z] = corner_for([10, 20], size, turn);
            let [sx, _, sz] = size;
            let within = |[a, b]: [i32; 2]| (x..x + sx).contains(&a) && (z..z + sz).contains(&b);
            let door = outside([10, 20], turn);
            assert!(!within(door), "{turn:?}: outside the building");
            assert_eq!(
                (door[0] - 10).abs() + (door[1] - 20).abs(),
                1,
                "{turn:?}: next to it"
            );
        }
    }
}
