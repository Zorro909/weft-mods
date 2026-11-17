//! Finding oil fields from afar. Fields lie on a grid of squares `SPACING` wide, at most one a
//! square, so a search asks the engine about each square near (`World::structure_near`), never
//! looking at a block. The engine answers only with the start nearest a point within a radius,
//! which may be a neighbour's: `start_in` narrows down to quarters of the square until the answer
//! is its own or none.

use weft_sdk::World;

use super::fields::{OIL_FIELD, SPACING};
use super::known::{self, Depth};
use crate::lib::Pos;

/// A field found: its start, where its seep is, and where its oil lies, if known.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Found {
    pub start: Pos,
    pub depth: Option<Depth>,
}

/// The squares of the grid that come within `reach` blocks of `at`, along x and z.
pub fn squares_near(at: Pos, reach: i32) -> Vec<(i32, i32)> {
    let span =
        |middle: i32| (middle - reach).div_euclid(SPACING)..=(middle + reach).div_euclid(SPACING);
    let mut squares = Vec::new();
    for z in span(at.z) {
        for x in span(at.x) {
            let square = Rect::square(x, z);
            if square.distance_squared(at) <= (reach as i64).pow(2) {
                squares.push((x, z));
            }
        }
    }
    squares
}

/// The field in the square `(x, z)` of the grid, in `world`, if one starts there.
pub fn field_in(world: &World, square: (i32, i32)) -> Option<Found> {
    let name = OIL_FIELD.name();
    let near = |at: Pos, radius: i32| world.structure_near(name, at, radius).map(Pos::from);
    let start = start_in(square, near)?;
    let depth = known::depth(world.seed(), world.dimension(), start);
    Some(Found { start, depth })
}

/// The start in the square `(x, z)`, asking `near` for the nearest start within a radius of a
/// point, as the engine's `structure_near` answers.
pub fn start_in(square: (i32, i32), near: impl Fn(Pos, i32) -> Option<Pos>) -> Option<Pos> {
    let whole = Rect::square(square.0, square.1);
    look(whole, whole, &near)
}

/// The start of `square` within `part` of it: none when the nearest start to the middle of
/// `part`, within a radius that covers it, is none or another square's and `part` is one column.
fn look(part: Rect, square: Rect, near: &impl Fn(Pos, i32) -> Option<Pos>) -> Option<Pos> {
    let found = near(part.middle(), part.reach())?;
    if square.has(found) {
        // A square holds one start at most: this is it.
        return Some(found);
    }
    part.quarters()
        .into_iter()
        .flatten()
        .find_map(|quarter| look(quarter, square, near))
}

/// Columns from (`x`, `z`), `width` along x and `depth` along z.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Rect {
    x: i32,
    z: i32,
    width: i32,
    depth: i32,
}

impl Rect {
    /// The square `(x, z)` of the grid.
    fn square(x: i32, z: i32) -> Rect {
        Rect {
            x: x * SPACING,
            z: z * SPACING,
            width: SPACING,
            depth: SPACING,
        }
    }

    fn has(&self, at: Pos) -> bool {
        (self.x..self.x + self.width).contains(&at.x)
            && (self.z..self.z + self.depth).contains(&at.z)
    }

    /// Its middle column, or the one past the middle.
    fn middle(&self) -> Pos {
        Pos::new(self.x + self.width / 2, 0, self.z + self.depth / 2)
    }

    /// A radius round its `middle` that takes in every column of it.
    fn reach(&self) -> i32 {
        let (half_x, half_z) = ((self.width / 2) as f64, (self.depth / 2) as f64);
        half_x.hypot(half_z).ceil() as i32
    }

    /// Its halves along each way it is more than a column across: two or four parts, none for one
    /// column.
    fn quarters(&self) -> [Option<Rect>; 4] {
        let halves = |from: i32, across: i32| match across {
            1 => [(from, 1), (0, 0)],
            _ => [(from, across / 2), (from + across / 2, across - across / 2)],
        };
        let (xs, zs) = (halves(self.x, self.width), halves(self.z, self.depth));
        let mut parts = [None; 4];
        if self.width == 1 && self.depth == 1 {
            return parts;
        }
        for (i, (&(x, width), &(z, depth))) in xs
            .iter()
            .flat_map(|x| zs.iter().map(move |z| (x, z)))
            .enumerate()
        {
            parts[i] = (width > 0 && depth > 0).then_some(Rect { x, z, width, depth });
        }
        parts
    }

    /// The square of the distance from `at` to its nearest column, along x and z.
    fn distance_squared(&self, at: Pos) -> i64 {
        let off =
            |from: i32, across: i32, v: i32| (from - v).max(v - (from + across - 1)).max(0) as i64;
        off(self.x, self.width, at.x).pow(2) + off(self.z, self.depth, at.z).pow(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Answers as the engine's `structure_near` does over `starts`: the nearest within the radius,
    /// counting each call.
    fn engine(
        starts: &[Pos],
    ) -> (
        impl Fn(Pos, i32) -> Option<Pos> + '_,
        std::rc::Rc<std::cell::Cell<u32>>,
    ) {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let counted = std::rc::Rc::clone(&calls);
        let near = move |at: Pos, radius: i32| {
            counted.set(counted.get() + 1);
            let distance =
                |start: &Pos| (((start.x - at.x) as f64).hypot((start.z - at.z) as f64)) as f32;
            starts
                .iter()
                .filter(|start| distance(start) <= radius as f32)
                .min_by(|a, b| distance(a).total_cmp(&distance(b)))
                .copied()
        };
        (near, calls)
    }

    #[test]
    fn a_lone_field_is_found_with_one_question() {
        let starts = [Pos::new(130, 70, 50)];
        let (near, calls) = engine(&starts);
        assert_eq!(start_in((1, 0), &near), Some(starts[0]), "its own square");
        assert_eq!(calls.get(), 1, "one question");
        assert_eq!(start_in((0, 0), &near), None, "the square beside has none");
    }

    #[test]
    fn a_field_hidden_by_a_nearer_neighbour_is_still_found() {
        // The neighbour's start sits just over the edge, nearer the square's middle than its own,
        // which lies in its far corner.
        let own = Pos::new(95, 70, 95);
        let neighbour = Pos::new(96, 70, 48);
        let starts = [own, neighbour];
        let (near, _) = engine(&starts);
        assert_eq!(
            near(Pos::new(48, 0, 48), 68),
            Some(neighbour),
            "hidden at first"
        );
        assert_eq!(start_in((0, 0), &near), Some(own), "found by its quarter");
        assert_eq!(start_in((1, 0), &near), Some(neighbour), "and the other's");
    }

    #[test]
    fn every_corner_of_a_square_is_searched() {
        for (x, z) in [(0, 0), (95, 0), (0, 95), (95, 95), (47, 48), (-96, -1)] {
            let start = Pos::new(x, 64, z);
            let square = (x.div_euclid(SPACING), z.div_euclid(SPACING));
            let others = [Pos::new(x + 1 - 2 * (x % 2), 64, z + 97)];
            let starts = [start, others[0]];
            let (near, _) = engine(&starts);
            assert_eq!(start_in(square, &near), Some(start), "at {x}, {z}");
        }
    }

    #[test]
    fn only_squares_within_reach_are_asked_about() {
        let at = Pos::new(10, 64, 10);
        let squares = squares_near(at, 256);
        assert!(squares.contains(&(0, 0)), "its own");
        assert!(
            squares.contains(&(-3, 0)) && squares.contains(&(2, 0)),
            "256 either way"
        );
        assert!(!squares.contains(&(-3, -3)), "a corner further than 256");
        assert!(squares.len() <= 49, "at most 7 by 7: {}", squares.len());
    }
}
