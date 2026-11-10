//! What cogwork works out about points in the world (`Vec3`, in blocks) that `Vec3` itself
//! doesn't: going toward a direction or a goal, and how far apart two points are.

use weft_sdk::*;

/// `blocks` blocks from `at` toward `dir`; away from it for a negative number.
pub fn toward(at: Vec3, dir: Dir, blocks: f64) -> Vec3 {
    let step = dir.offset();
    at.offset(
        step.x as f64 * blocks,
        step.y as f64 * blocks,
        step.z as f64 * blocks,
    )
}

pub fn distance(a: Vec3, b: Vec3) -> f64 {
    (b - a).length_squared().sqrt()
}

/// `blocks` blocks along the straight line from `from` to `goal`; `goal` itself when that is
/// nearer.
pub fn moved_toward(from: Vec3, goal: Vec3, blocks: f64) -> Vec3 {
    let length = distance(from, goal);
    if length <= blocks {
        return goal;
    }
    from + (goal - from) * (blocks / length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_point_moves_toward_a_direction() {
        let start = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(
            toward(start, Dir::Down, 0.25),
            Vec3::new(1.0, 1.75, 3.0),
            "a quarter down"
        );
        assert_eq!(
            toward(start, Dir::East, -2.0),
            Vec3::new(-1.0, 2.0, 3.0),
            "two west"
        );
    }

    #[test]
    fn a_point_moves_toward_a_goal_and_stops_there() {
        let (from, goal) = (Vec3::ZERO, Vec3::new(3.0, 4.0, 0.0));
        assert_eq!(distance(from, goal), 5.0, "a 3-4-5 triangle");
        let part = moved_toward(from, goal, 2.5);
        assert_eq!(part, Vec3::new(1.5, 2.0, 0.0), "half way");
        assert_eq!(moved_toward(from, goal, 9.0), goal, "never past it");
    }
}
