//! Where an excavator works: its frame, a box five blocks high from its own height, and the
//! order it mines the inside in, a layer at a time from the top, back and forth along rows.

use weft_sdk::*;

/// The frame's height above the excavator's block.
const FRAME_RISE: i32 = 4;
/// Inside of the frame when no markers say otherwise: 9 by 9.
const DEFAULT_INSIDE: i32 = 9;

/// The frame's lowest and highest corner for an excavator at `pos` facing `front`: on the
/// rectangle of the survey markers next to it, else around a 9 by 9 area behind it.
pub fn plan(w: &World, pos: [i32; 3], front: Dir) -> ([i32; 3], [i32; 3]) {
    let (lo, hi) = crate::markers::take_area(w, pos)
        .unwrap_or_else(|| crate::markers::square_behind(pos, front, DEFAULT_INSIDE, 1));
    ([lo[0], pos[1], lo[2]], [hi[0], pos[1] + FRAME_RISE, hi[2]])
}

/// The frame's blocks: every edge of the box, from the bottom up.
pub fn frame_cells(lo: [i32; 3], hi: [i32; 3]) -> Vec<[i32; 3]> {
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
pub fn first(lo: [i32; 3], hi: [i32; 3]) -> [i32; 3] {
    [lo[0] + 1, hi[1], lo[2] + 1]
}

/// Which way along x the row of `c` is mined: 1 on the first row of a layer, then -1, 1 ...
pub fn row_dir(lo: [i32; 3], c: [i32; 3]) -> i32 {
    if (c[2] - (lo[2] + 1)) % 2 == 0 { 1 } else { -1 }
}

/// The block mined after `c`: along the row, back the other way on the next row, then the
/// next layer down.
pub fn next(lo: [i32; 3], hi: [i32; 3], c: [i32; 3]) -> [i32; 3] {
    let (x0, x1, z0, z1) = (lo[0] + 1, hi[0] - 1, lo[2] + 1, hi[2] - 1);
    let x = c[0] + row_dir(lo, c);
    if (x0..=x1).contains(&x) {
        return [x, c[1], c[2]];
    }
    if c[2] < z1 {
        let z = c[2] + 1;
        return [if (z - z0) % 2 == 0 { x0 } else { x1 }, c[1], z];
    }
    [x0, c[1] - 1, z0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_has_twelve_edges() {
        let cells = frame_cells([0, 0, 0], [10, 4, 10]);
        // Two rings of 40 and four posts of 3 between them.
        assert_eq!(cells.len(), 40 * 2 + 4 * 3, "rings and posts");
    }

    #[test]
    fn mining_order_snakes_through_a_layer() {
        let (lo, hi) = ([0, 0, 0], [3, 4, 3]);
        let mut c = first(lo, hi);
        let mut seen = vec![c];
        for _ in 0..4 {
            c = next(lo, hi, c);
            seen.push(c);
        }
        assert_eq!(
            seen,
            vec![[1, 4, 1], [2, 4, 1], [2, 4, 2], [1, 4, 2], [1, 3, 1]],
            "back and forth, then down"
        );
    }
}
