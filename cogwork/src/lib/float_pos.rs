//! A point in the world, in blocks: where entities, particles and cameras are, anywhere inside a
//! block. A block's `Pos` turns into the point at its lowest corner; a point doesn't turn back
//! into a `Pos`, as it lies anywhere in its block (`Pos::containing` says which). The SDK takes
//! and gives `[f64; 3]`.

use weft_sdk::*;

use super::Pos;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct FloatPos {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl FloatPos {
    pub const fn new(x: f64, y: f64, z: f64) -> FloatPos {
        FloatPos { x, y, z }
    }

    /// `dx` blocks along x from here.
    pub const fn add_x(self, dx: f64) -> FloatPos {
        FloatPos {
            x: self.x + dx,
            ..self
        }
    }

    /// `dy` blocks up from here.
    pub const fn add_y(self, dy: f64) -> FloatPos {
        FloatPos {
            y: self.y + dy,
            ..self
        }
    }

    /// `dz` blocks along z from here.
    pub const fn add_z(self, dz: f64) -> FloatPos {
        FloatPos {
            z: self.z + dz,
            ..self
        }
    }

    /// `blocks` blocks from here toward `dir`; away from it for a negative number.
    pub fn toward_by(self, dir: Dir, blocks: f64) -> FloatPos {
        let [dx, dy, dz] = dir.offset().map(|v| v as f64 * blocks);
        self.add_x(dx).add_y(dy).add_z(dz)
    }

    pub fn distance_to(self, other: FloatPos) -> f64 {
        let (dx, dy, dz) = (other.x - self.x, other.y - self.y, other.z - self.z);
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// `blocks` blocks along the straight line to `goal`; `goal` itself when that is nearer.
    pub fn moved_toward(self, goal: FloatPos, blocks: f64) -> FloatPos {
        let length = self.distance_to(goal);
        if length <= blocks {
            return goal;
        }
        let share = blocks / length;
        FloatPos {
            x: self.x + (goal.x - self.x) * share,
            y: self.y + (goal.y - self.y) * share,
            z: self.z + (goal.z - self.z) * share,
        }
    }
}

impl From<Pos> for FloatPos {
    fn from(p: Pos) -> FloatPos {
        FloatPos::new(p.x as f64, p.y as f64, p.z as f64)
    }
}

impl From<[f64; 3]> for FloatPos {
    fn from([x, y, z]: [f64; 3]) -> FloatPos {
        FloatPos { x, y, z }
    }
}

impl From<FloatPos> for [f64; 3] {
    fn from(p: FloatPos) -> [f64; 3] {
        [p.x, p.y, p.z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_is_the_point_at_its_lowest_corner() {
        let p = FloatPos::from(Pos::new(-2, 3, 7));
        assert_eq!(p, FloatPos::new(-2.0, 3.0, 7.0), "whole numbers");
        assert_eq!(
            <[f64; 3]>::from(p),
            [-2.0, 3.0, 7.0],
            "into the SDK's array"
        );
    }

    #[test]
    fn a_point_moves_along_its_axes() {
        let p = FloatPos::new(1.0, 2.0, 3.0);
        assert_eq!(
            p.add_x(0.5).add_y(-1.0).add_z(2.0),
            FloatPos::new(1.5, 1.0, 5.0),
            "each by its own amount"
        );
        let down = p.toward_by(Dir::Down, 0.25);
        assert_eq!(down, FloatPos::new(1.0, 1.75, 3.0), "a quarter down");
    }

    #[test]
    fn a_point_moves_toward_a_goal_and_stops_there() {
        let (from, goal) = (FloatPos::new(0.0, 0.0, 0.0), FloatPos::new(3.0, 4.0, 0.0));
        assert_eq!(from.distance_to(goal), 5.0, "a 3-4-5 triangle");
        let part = from.moved_toward(goal, 2.5);
        assert_eq!(part, FloatPos::new(1.5, 2.0, 0.0), "half way");
        assert_eq!(from.moved_toward(goal, 9.0), goal, "never past it");
    }
}
