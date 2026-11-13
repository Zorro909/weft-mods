//! Where an excavator works: its frame, a box five blocks high from its own height, and the
//! order it mines the inside in, a layer at a time from the top, back and forth along rows.

use std::collections::HashSet;

use weft_sdk::{Dir, World, value};

use super::GANTRY_FRAME;
use crate::lib::{Area, Pos, TypedWorld};
use crate::markers::{LARGEST_REACH, square_behind, take_area};

/// The frame's height above the excavator's block.
const FRAME_RISE: i32 = 4;
/// Inside of the frame when no markers say otherwise: 9 by 9.
const DEFAULT_INSIDE: i32 = 9;
/// The most frame blocks followed looking for a standing frame: a ring round the largest area
/// survey markers mark out in any world.
const MAX_RING: usize = 4 * (LARGEST_REACH as usize + 2);
const SIDES: [Dir; 4] = [Dir::West, Dir::East, Dir::North, Dir::South];

value! {
    /// An excavator's frame, by its lowest and highest corner. It mines the blocks inside.
    #[derive(Default)]
    pub struct Frame {
        pub lo: Pos,
        pub hi: Pos,
    }
}

impl Frame {
    /// The frame for an excavator at `pos` facing `front`: on the rectangle of the survey
    /// markers next to it, else on a frame already standing beside it (one put back by its old
    /// frame carries on there), else around a 9 by 9 area behind it.
    pub fn plan(world: &World, pos: Pos, front: Dir) -> Frame {
        let ground = take_area(world, pos)
            .or_else(|| standing_frame(world, pos))
            .unwrap_or_else(|| square_behind(pos, front, DEFAULT_INSIDE, 1));
        Frame {
            lo: ground.lo.with_y(pos.y),
            hi: ground.hi.with_y(pos.y + FRAME_RISE),
        }
    }

    /// Blocks across the inside, along x and along z.
    pub fn inside(&self) -> [i32; 2] {
        let across = |lo: i32, hi: i32| (hi - lo - 1).max(0);
        [across(self.lo.x, self.hi.x), across(self.lo.z, self.hi.z)]
    }

    /// The frame's blocks: every edge of the box, from the bottom up.
    pub fn cells(&self) -> Vec<Pos> {
        let (lo, hi) = (self.lo, self.hi);
        let mut cells = Vec::new();
        for y in lo.y..=hi.y {
            for z in lo.z..=hi.z {
                for x in lo.x..=hi.x {
                    let on = [
                        x == lo.x || x == hi.x,
                        y == lo.y || y == hi.y,
                        z == lo.z || z == hi.z,
                    ];
                    if on.iter().filter(|&&on_edge| on_edge).count() >= 2 {
                        cells.push(Pos::new(x, y, z));
                    }
                }
            }
        }
        cells
    }

    /// The first block to mine: a corner of the inside, at the top of the frame.
    pub fn first(&self) -> Pos {
        self.first_of_layer(self.hi.y)
    }

    /// Which way along x the row of `cell` is mined: 1 on the first row of a layer, then -1, 1 ...
    pub fn row_dir(&self, cell: Pos) -> i32 {
        if (cell.z - (self.lo.z + 1)) % 2 == 0 {
            1
        } else {
            -1
        }
    }

    /// The block mined after `cell`: along the row, back the other way on the next row, then the
    /// next layer down.
    pub fn next(&self, cell: Pos) -> Pos {
        let (first_x, last_x) = (self.lo.x + 1, self.hi.x - 1);
        let along = cell.add_x(self.row_dir(cell));
        if (first_x..=last_x).contains(&along.x) {
            return along;
        }
        if cell.z < self.hi.z - 1 {
            let next_row = cell.add_z(1);
            let x = if self.row_dir(next_row) == 1 {
                first_x
            } else {
                last_x
            };
            return Pos { x, ..next_row };
        }
        self.first_of_layer(cell.y - 1)
    }

    fn first_of_layer(&self, y: i32) -> Pos {
        self.lo.add_x(1).add_z(1).with_y(y)
    }
}

/// A flat frame: only its ring round `area`.
impl From<Area> for Frame {
    fn from(area: Area) -> Frame {
        Frame {
            lo: area.lo,
            hi: area.hi,
        }
    }
}

/// For tests: a frame from its corners written as arrays.
#[cfg(test)]
impl From<([i32; 3], [i32; 3])> for Frame {
    fn from((lo, hi): ([i32; 3], [i32; 3])) -> Frame {
        Frame {
            lo: lo.into(),
            hi: hi.into(),
        }
    }
}

/// The bottom ring of a frame beside `pos` at its height, with `pos` outside it.
fn standing_frame(world: &World, pos: Pos) -> Option<Area> {
    let ring = ring_bounds(pos, |at| world.block_typed(at).is(&GANTRY_FRAME))?;
    (!ring.spans(pos)).then_some(ring)
}

/// The area the frame blocks joined to a side of `pos` ring, at its height, if they make a ring
/// round an inside at least one block across. One block may be missing from the ring,
/// which is still joined the other way round; with more, the ring may be in pieces, so it isn't
/// taken.
fn ring_bounds(pos: Pos, is_frame: impl Fn(Pos) -> bool) -> Option<Area> {
    let start = SIDES
        .map(|dir| pos.toward(dir))
        .into_iter()
        .find(|&at| is_frame(at))?;
    let mut seen = HashSet::from([start]);
    let mut todo = vec![start];
    let mut ring = Area::between(start, start);
    while let Some(at) = todo.pop() {
        for beside in SIDES.map(|dir| at.toward(dir)) {
            if seen.len() >= MAX_RING || seen.contains(&beside) || !is_frame(beside) {
                continue;
            }
            seen.insert(beside);
            todo.push(beside);
            ring.grow_to(beside);
        }
    }
    let flat = Frame::from(ring);
    let missing = flat
        .cells()
        .into_iter()
        .filter(|&cell| !is_frame(cell))
        .count();
    let [x, z] = flat.inside();
    (x >= 1 && z >= 1 && missing <= 1).then_some(ring)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_has_twelve_edges() {
        let cells = Frame::from(([0, 0, 0], [10, 4, 10])).cells();
        // Two rings of 40 and four posts of 3 between them.
        assert_eq!(cells.len(), 40 * 2 + 4 * 3, "rings and posts");
    }

    #[test]
    fn the_inside_is_the_frame_less_its_edges() {
        let frame = Frame::from(([0, 0, 0], [10, 4, 6]));
        assert_eq!(frame.inside(), [9, 5], "across x and z");
        let unplanned = Frame::default();
        assert_eq!(unplanned.inside(), [0, 0], "nothing yet, not -1");
    }

    /// The bottom ring of a frame from `lo` to `hi`, less the blocks in `gaps`.
    fn ring(lo: [i32; 3], hi: [i32; 3], gaps: &[[i32; 3]]) -> HashSet<Pos> {
        let gaps: Vec<Pos> = gaps.iter().map(|&gap| gap.into()).collect();
        Frame::from((lo, hi))
            .cells()
            .into_iter()
            .filter(|cell| cell.y == lo[1] && !gaps.contains(cell))
            .collect()
    }

    #[test]
    fn a_standing_frame_is_found_from_beside_it() {
        let (lo, hi) = ([0, 4, 0], [10, 8, 10]);
        let bottom = Some(Area::between(Pos::new(0, 4, 0), Pos::new(10, 4, 10)));
        let found = |from: [i32; 3], blocks: &HashSet<Pos>| {
            ring_bounds(from.into(), |at| blocks.contains(&at))
        };
        let whole = ring(lo, hi, &[]);
        assert_eq!(found([5, 4, 11], &whole), bottom, "beside an edge");
        assert_eq!(found([-1, 4, 0], &whole), bottom, "beside a corner");
        let gap = ring(lo, hi, &[[5, 4, 10]]);
        assert_eq!(found([4, 4, 11], &gap), bottom, "with a gap in it");
        let pieces = ring(lo, hi, &[[5, 4, 10], [0, 4, 3]]);
        assert_eq!(
            found([4, 4, 11], &pieces),
            None,
            "in pieces, not a smaller frame"
        );
        let post = HashSet::from([Pos::new(0, 4, 0)]);
        assert_eq!(found([-1, 4, 0], &post), None, "a lone block");
    }

    #[test]
    fn mining_order_snakes_through_a_layer() {
        let frame = Frame::from(([0, 0, 0], [3, 4, 3]));
        let mut cell = frame.first();
        let mut seen = vec![cell];
        for _ in 0..4 {
            cell = frame.next(cell);
            seen.push(cell);
        }
        let expected = [[1, 4, 1], [2, 4, 1], [2, 4, 2], [1, 4, 2], [1, 3, 1]];
        assert_eq!(seen, expected.map(Pos::from), "back and forth, then down");
    }

    #[test]
    fn every_block_of_a_layer_is_mined_once_before_the_next() {
        // An even and an odd number of rows, 5 blocks long.
        for hi in [[6, 4, 5], [6, 4, 4]] {
            let frame = Frame::from(([0, 0, 0], hi));
            let mut cell = frame.first();
            let mut layer = HashSet::new();
            while cell.y == hi[1] {
                assert!(layer.insert(cell), "{cell:?} only once");
                cell = frame.next(cell);
            }
            assert_eq!(layer.len() as i32, 5 * (hi[2] - 1), "the whole inside");
            assert_eq!(
                cell,
                Pos::new(1, hi[1] - 1, 1),
                "then the same corner one down"
            );
        }
    }
}
