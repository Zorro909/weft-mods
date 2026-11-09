//! A finished plan, its parts sorted by the cube columns they reach, so a cube looks only at
//! the parts that can touch it.

use std::collections::HashMap;

use weftkit::{Area, Pos};

use crate::part::Part;
use crate::stamp::{self, Set, Surface};

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

    /// The box round every part; `None` for an empty plan.
    pub fn bounds(&self) -> Option<Area> {
        let mut parts = self.parts.iter().map(Part::bounds);
        let mut all = parts.next()?;
        for area in parts {
            all.grow_to(area.lo);
            all.grow_to(area.hi);
        }
        Some(all)
    }

    /// Whether any of its parts comes within `room` blocks of one of `other`'s, seen from above.
    pub fn near(&self, other: &Planned, room: i32) -> bool {
        (self.parts.iter()).any(|part| other.reaches_into(&widened(part.bounds(), room)))
    }

    /// Whether any part reaches into the columns of `area`, seen from above.
    fn reaches_into(&self, area: &Area) -> bool {
        let columns = area.heights(i32::MIN, i32::MAX);
        let cubes = |lo: i32, hi: i32| lo.div_euclid(16)..=hi.div_euclid(16);
        let (lo, hi) = (area.lo, area.hi);
        let cube_columns = cubes(lo.z, hi.z).flat_map(|z| cubes(lo.x, hi.x).map(move |x| [x, z]));
        let nearby = cube_columns
            .filter_map(|column| self.columns.get(&column))
            .flatten();
        nearby
            .map(|&i| self.parts[i].bounds())
            .any(|bounds| columns.overlaps(&bounds))
    }

    /// Calls `set` for every block the plan puts into the cube at `origin`, later parts after
    /// earlier ones.
    pub fn stamp(&self, origin: Pos, surface: Surface, set: Set) {
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

/// `area` with `room` more blocks on every side.
pub(crate) fn widened(area: Area, room: i32) -> Area {
    let (out, back) = (Pos::new(room, room, room), Pos::new(-room, -room, -room));
    Area::between(area.lo + back, area.hi + out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fill(from: [i32; 3], to: [i32; 3], state: u32) -> Part {
        Part::Fill {
            area: Area::between(from.into(), to.into()),
            state,
            only: None,
        }
    }

    fn states(planned: &Planned, origin: [i32; 3]) -> Vec<u32> {
        let origin = Pos::from(origin);
        let mut out = Vec::new();
        planned.stamp(origin, &|_, _| 0, &mut |_, state, _| out.push(state));
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
