//! A finished plan, its parts sorted by the cube columns they reach, so a cube looks only at
//! the parts that can touch it.

use std::collections::HashMap;

use crate::part::Part;
use crate::stamp::{self, Surface};

pub(crate) struct Planned {
    parts: Vec<Part>,
    /// Indices into `parts` by cube column (x, z), in the order they were added.
    columns: HashMap<[i32; 2], Vec<usize>>,
}

impl Planned {
    pub fn new(parts: Vec<Part>) -> Planned {
        let mut columns: HashMap<[i32; 2], Vec<usize>> = HashMap::new();
        for (i, part) in parts.iter().enumerate() {
            let (from, to) = part.bounds();
            for z in from[2].div_euclid(16)..=to[2].div_euclid(16) {
                for x in from[0].div_euclid(16)..=to[0].div_euclid(16) {
                    columns.entry([x, z]).or_default().push(i);
                }
            }
        }
        Planned { parts, columns }
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    /// The lowest corner of what was added first: where `locate` points.
    pub fn anchor(&self) -> Option<[i32; 3]> {
        self.parts.first().map(|part| part.bounds().0)
    }

    /// Calls `set` for every block the plan puts into the cube at `origin`, later parts after
    /// earlier ones.
    pub fn stamp(&self, origin: [i32; 3], surface: Surface, set: &mut dyn FnMut([usize; 3], u32)) {
        for &i in self.parts_in(origin) {
            stamp::stamp(&self.parts[i], origin, surface, set);
        }
    }

    /// Whether anything of it may be in the cube at `origin`.
    pub fn reaches(&self, origin: [i32; 3]) -> bool {
        self.parts_in(origin).next().is_some()
    }

    fn parts_in(&self, origin: [i32; 3]) -> impl Iterator<Item = &usize> {
        let column = [origin[0].div_euclid(16), origin[2].div_euclid(16)];
        self.columns.get(&column).into_iter().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fill(from: [i32; 3], to: [i32; 3], state: u32) -> Part {
        Part::Fill { from, to, state }
    }

    fn states(planned: &Planned, origin: [i32; 3]) -> Vec<u32> {
        let mut out = Vec::new();
        planned.stamp(origin, &|_, _| 0, &mut |_, state| out.push(state));
        out
    }

    #[test]
    fn a_part_is_found_in_every_column_it_reaches() {
        let planned = Planned::new(vec![fill([10, 0, 0], [20, 0, 0], 1)]);
        assert_eq!(states(&planned, [0, 0, 0]).len(), 6, "10 to 15");
        assert_eq!(states(&planned, [16, 0, 0]).len(), 5, "16 to 20");
        assert!(states(&planned, [32, 0, 0]).is_empty(), "not further");
    }

    #[test]
    fn later_parts_come_later() {
        let planned = Planned::new(vec![
            fill([0, 0, 0], [0, 0, 0], 1),
            fill([0, 0, 0], [0, 0, 0], 2),
        ]);
        assert_eq!(states(&planned, [0, 0, 0]), [1, 2], "so the later one wins");
    }

    #[test]
    fn the_anchor_is_the_first_part() {
        let planned = Planned::new(vec![
            fill([5, 1, 7], [9, 1, 9], 1),
            fill([0, 0, 0], [1, 1, 1], 2),
        ]);
        assert_eq!(planned.anchor(), Some([5, 1, 7]), "its lowest corner");
        assert_eq!(Planned::new(Vec::new()).anchor(), None, "nothing planned");
    }
}
