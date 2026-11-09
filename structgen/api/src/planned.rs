//! A finished plan, its parts sorted by the cube columns they reach, so a cube looks only at
//! the parts that can touch it.

use std::collections::HashMap;

use weftkit::{Area, Pos};

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
            let Area { lo, hi } = part.bounds();
            for z in lo.z.div_euclid(16)..=hi.z.div_euclid(16) {
                for x in lo.x.div_euclid(16)..=hi.x.div_euclid(16) {
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
    pub fn anchor(&self) -> Option<Pos> {
        self.parts.first().map(|part| part.bounds().lo)
    }

    /// The box round every part, seen from above and below; `None` for an empty plan.
    pub fn bounds(&self) -> Option<([i32; 3], [i32; 3])> {
        let boxes = self.parts.iter().map(Part::bounds);
        boxes.reduce(|(a, b), (c, d)| {
            (
                [0, 1, 2].map(|i| a[i].min(c[i])),
                [0, 1, 2].map(|i| b[i].max(d[i])),
            )
        })
    }

    /// Whether any of its parts comes within `room` blocks of one of `other`'s, seen from above.
    pub fn near(&self, other: &Planned, room: i32) -> bool {
        self.parts.iter().any(|part| {
            let (from, to) = part.bounds();
            other.reaches_into(from.map(|v| v - room), to.map(|v| v + room))
        })
    }

    /// Whether any part reaches into the columns from `from` to `to`, seen from above.
    fn reaches_into(&self, from: [i32; 3], to: [i32; 3]) -> bool {
        let cubes = |i: usize| from[i].div_euclid(16)..=to[i].div_euclid(16);
        let columns = cubes(2).flat_map(|z| cubes(0).map(move |x| [x, z]));
        let nearby = columns
            .filter_map(|column| self.columns.get(&column))
            .flatten();
        nearby
            .map(|&i| self.parts[i].bounds())
            .any(|(a, b)| [0, 2].into_iter().all(|i| a[i] <= to[i] && from[i] <= b[i]))
    }

    /// Calls `set` for every block the plan puts into the cube at `origin`, later parts after
    /// earlier ones.
    pub fn stamp(&self, origin: Pos, surface: Surface, set: &mut dyn FnMut([usize; 3], u32)) {
        for &i in self.parts_in(origin) {
            stamp::stamp(&self.parts[i], origin, surface, set);
        }
    }

    /// Whether anything of it may be in the cube at `origin`.
    pub fn reaches(&self, origin: Pos) -> bool {
        self.parts_in(origin).next().is_some()
    }

    fn parts_in(&self, origin: Pos) -> impl Iterator<Item = &usize> {
        let column = [origin.x.div_euclid(16), origin.z.div_euclid(16)];
        self.columns.get(&column).into_iter().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fill(from: [i32; 3], to: [i32; 3], state: u32) -> Part {
        Part::Fill {
            area: Area::between(from.into(), to.into()),
            state,
        }
    }

    fn states(planned: &Planned, origin: [i32; 3]) -> Vec<u32> {
        let origin = Pos::from(origin);
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
        assert_eq!(
            planned.anchor(),
            Some(Pos::new(5, 1, 7)),
            "its lowest corner"
        );
        assert_eq!(Planned::new(Vec::new()).anchor(), None, "nothing planned");
    }
}
