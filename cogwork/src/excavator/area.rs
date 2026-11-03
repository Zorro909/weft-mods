//! Where an excavator works: its frame, a box five blocks high from its own height, and the
//! order it mines the inside in, a layer at a time from the top, back and forth along rows.

use std::collections::HashSet;

use weft_sdk::*;

use super::GANTRY_FRAME;

/// The frame's height above the excavator's block.
const FRAME_RISE: i32 = 4;
/// Inside of the frame when no markers say otherwise: 9 by 9.
const DEFAULT_INSIDE: i32 = 9;
/// The most frame blocks followed looking for a standing frame: a ring round the largest area
/// survey markers mark out.
const MAX_RING: usize = 4 * 66;
const SIDES: [Dir; 4] = [Dir::West, Dir::East, Dir::North, Dir::South];

value! {
    /// An excavator's frame, by its lowest and highest corner. It mines the blocks inside.
    #[derive(Default)]
    pub struct Frame {
        pub lo: [i32; 3],
        pub hi: [i32; 3],
    }
}

impl Frame {
    /// The frame for an excavator at `pos` facing `front`: on the rectangle of the survey
    /// markers next to it, else on a frame already standing beside it (one put back by its old
    /// frame carries on there), else around a 9 by 9 area behind it.
    pub fn plan(w: &World, pos: [i32; 3], front: Dir) -> Frame {
        let (lo, hi) = crate::markers::take_area(w, pos)
            .or_else(|| standing_frame(w, pos))
            .unwrap_or_else(|| crate::markers::square_behind(pos, front, DEFAULT_INSIDE, 1));
        Frame {
            lo: [lo[0], pos[1], lo[2]],
            hi: [hi[0], pos[1] + FRAME_RISE, hi[2]],
        }
    }

    /// Blocks across the inside, along x and along z.
    pub fn inside(&self) -> [i32; 2] {
        let across = |axis: usize| (self.hi[axis] - self.lo[axis] - 1).max(0);
        [across(0), across(2)]
    }

    /// The frame's blocks: every edge of the box, from the bottom up.
    pub fn cells(&self) -> Vec<[i32; 3]> {
        let (lo, hi) = (self.lo, self.hi);
        let mut cells = Vec::new();
        for y in lo[1]..=hi[1] {
            for z in lo[2]..=hi[2] {
                for x in lo[0]..=hi[0] {
                    let on = [
                        x == lo[0] || x == hi[0],
                        y == lo[1] || y == hi[1],
                        z == lo[2] || z == hi[2],
                    ];
                    if on.iter().filter(|&&b| b).count() >= 2 {
                        cells.push([x, y, z]);
                    }
                }
            }
        }
        cells
    }

    /// The first block to mine: a corner of the inside, at the top of the frame.
    pub fn first(&self) -> [i32; 3] {
        [self.lo[0] + 1, self.hi[1], self.lo[2] + 1]
    }

    /// Which way along x the row of `c` is mined: 1 on the first row of a layer, then -1, 1 ...
    pub fn row_dir(&self, c: [i32; 3]) -> i32 {
        if (c[2] - (self.lo[2] + 1)) % 2 == 0 {
            1
        } else {
            -1
        }
    }

    /// The block mined after `c`: along the row, back the other way on the next row, then the
    /// next layer down.
    pub fn next(&self, c: [i32; 3]) -> [i32; 3] {
        let (x0, x1) = (self.lo[0] + 1, self.hi[0] - 1);
        let (z0, z1) = (self.lo[2] + 1, self.hi[2] - 1);
        let x = c[0] + self.row_dir(c);
        if (x0..=x1).contains(&x) {
            return [x, c[1], c[2]];
        }
        if c[2] < z1 {
            let z = c[2] + 1;
            return [if (z - z0) % 2 == 0 { x0 } else { x1 }, c[1], z];
        }
        [x0, c[1] - 1, z0]
    }
}

/// The bottom ring of a frame beside `pos` at its height, with `pos` outside it: its lowest and
/// highest corner.
fn standing_frame(w: &World, pos: [i32; 3]) -> Option<([i32; 3], [i32; 3])> {
    let (lo, hi) = ring_bounds(pos, |p| GANTRY_FRAME.is(w.get(p)))?;
    let inside = |axis: usize| (lo[axis]..=hi[axis]).contains(&pos[axis]);
    (!(inside(0) && inside(2))).then_some((lo, hi))
}

/// The corners round the frame blocks joined to a side of `pos`, at its height, if they make a
/// ring round an area at least one block across. One block may be missing from the ring, which
/// is still joined the other way round; with more, the ring may be in pieces, so it isn't
/// taken.
fn ring_bounds(pos: [i32; 3], is_frame: impl Fn([i32; 3]) -> bool) -> Option<([i32; 3], [i32; 3])> {
    let start = SIDES
        .map(|d| beside(pos, d))
        .into_iter()
        .find(|&p| is_frame(p))?;
    let mut seen = HashSet::from([start]);
    let mut todo = vec![start];
    let (mut lo, mut hi) = (start, start);
    while let Some(p) = todo.pop() {
        for n in SIDES.map(|d| beside(p, d)) {
            if seen.len() >= MAX_RING || seen.contains(&n) || !is_frame(n) {
                continue;
            }
            seen.insert(n);
            todo.push(n);
            lo = std::array::from_fn(|i| lo[i].min(n[i]));
            hi = std::array::from_fn(|i| hi[i].max(n[i]));
        }
    }
    let missing = Frame { lo, hi }
        .cells()
        .into_iter()
        .filter(|&c| !is_frame(c))
        .count();
    (hi[0] - lo[0] >= 2 && hi[2] - lo[2] >= 2 && missing <= 1).then_some((lo, hi))
}

fn beside(p: [i32; 3], d: Dir) -> [i32; 3] {
    let o = d.offset();
    [p[0] + o[0], p[1], p[2] + o[2]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_has_twelve_edges() {
        let frame = Frame {
            lo: [0, 0, 0],
            hi: [10, 4, 10],
        };
        let cells = frame.cells();
        // Two rings of 40 and four posts of 3 between them.
        assert_eq!(cells.len(), 40 * 2 + 4 * 3, "rings and posts");
    }

    #[test]
    fn the_inside_is_the_frame_less_its_edges() {
        let frame = Frame {
            lo: [0, 0, 0],
            hi: [10, 4, 6],
        };
        assert_eq!(frame.inside(), [9, 5], "across x and z");
        let unplanned = Frame::default();
        assert_eq!(unplanned.inside(), [0, 0], "nothing yet, not -1");
    }

    /// The bottom ring of a frame from `lo` to `hi`, less the blocks in `gaps`.
    fn ring(lo: [i32; 3], hi: [i32; 3], gaps: &[[i32; 3]]) -> HashSet<[i32; 3]> {
        Frame { lo, hi }
            .cells()
            .into_iter()
            .filter(|c| c[1] == lo[1] && !gaps.contains(c))
            .collect()
    }

    #[test]
    fn a_standing_frame_is_found_from_beside_it() {
        let (lo, hi) = ([0, 4, 0], [10, 8, 10]);
        let whole = ring(lo, hi, &[]);
        let found = ring_bounds([5, 4, 11], |p| whole.contains(&p));
        assert_eq!(found, Some(([0, 4, 0], [10, 4, 10])), "beside an edge");
        let found = ring_bounds([-1, 4, 0], |p| whole.contains(&p));
        assert_eq!(found, Some(([0, 4, 0], [10, 4, 10])), "beside a corner");
        let gap = ring(lo, hi, &[[5, 4, 10]]);
        let found = ring_bounds([4, 4, 11], |p| gap.contains(&p));
        assert_eq!(found, Some(([0, 4, 0], [10, 4, 10])), "with a gap in it");
        let pieces = ring(lo, hi, &[[5, 4, 10], [0, 4, 3]]);
        let found = ring_bounds([4, 4, 11], |p| pieces.contains(&p));
        assert_eq!(found, None, "in pieces, not a smaller frame");
        let post = HashSet::from([[0, 4, 0]]);
        assert_eq!(
            ring_bounds([-1, 4, 0], |p| post.contains(&p)),
            None,
            "a lone block"
        );
    }

    #[test]
    fn mining_order_snakes_through_a_layer() {
        let frame = Frame {
            lo: [0, 0, 0],
            hi: [3, 4, 3],
        };
        let mut c = frame.first();
        let mut seen = vec![c];
        for _ in 0..4 {
            c = frame.next(c);
            seen.push(c);
        }
        assert_eq!(
            seen,
            vec![[1, 4, 1], [2, 4, 1], [2, 4, 2], [1, 4, 2], [1, 3, 1]],
            "back and forth, then down"
        );
    }

    #[test]
    fn every_block_of_a_layer_is_mined_once_before_the_next() {
        // An even and an odd number of rows, 5 blocks long.
        for hi in [[6, 4, 5], [6, 4, 4]] {
            let frame = Frame { lo: [0, 0, 0], hi };
            let mut c = frame.first();
            let mut layer = HashSet::new();
            while c[1] == hi[1] {
                assert!(layer.insert(c), "{c:?} only once");
                c = frame.next(c);
            }
            assert_eq!(layer.len() as i32, 5 * (hi[2] - 1), "the whole inside");
            assert_eq!(c, [1, hi[1] - 1, 1], "then the same corner one down");
        }
    }
}
