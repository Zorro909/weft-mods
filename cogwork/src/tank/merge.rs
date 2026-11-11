//! Which tank blocks make one tank, and how its liquid lies in it. A tank fills a box no more
//! than `WIDE` across either way and `HIGH` tall. A block placed beside tanks grows into them
//! one at a time, as long as together they still fill such a box (`grow`): a block that doesn't
//! fit stays a tank of its own and leaves the others as they are. A tank that loses a block parts
//! into tanks that fill boxes again (`part`). A tank fills from its bottom layer up, a layer's
//! blocks equally.

use std::collections::HashSet;

use weft_sdk::Dir;

use crate::lib::{Area, Pos};

/// The most blocks a tank is across, each way, and tall.
pub const WIDE: i32 = 3;
pub const HIGH: i32 = 6;
/// The most blocks a tank has.
pub const MOST: usize = (WIDE * WIDE * HIGH) as usize;

/// Whether `cells`, each once, fill a box a tank can be.
pub fn forms_box(cells: &[Pos]) -> bool {
    let Some(&first) = cells.first() else {
        return false;
    };
    let mut bounds = Area::between(first, first);
    for &cell in cells {
        bounds.grow_to(cell);
    }
    let [x, y, z] = bounds.size();
    let fits = x <= WIDE && z <= WIDE && y <= HIGH;
    fits && (x * y * z) as usize == cells.len()
}

/// The tank `start` grows into: it and, one after another, each tank beside it that it still
/// `fits` together with, until no more does. `tank_at` is the tank a block is part of, if it is
/// a tank block.
pub fn grow(
    start: Vec<Pos>,
    tank_at: impl Fn(Pos) -> Option<Vec<Pos>>,
    fits: impl Fn(&[Pos]) -> bool,
) -> Vec<Pos> {
    let mut tank = start;
    'grow: loop {
        for beside in around(&tank) {
            let Some(other) = tank_at(beside) else {
                continue;
            };
            let both: Vec<Pos> = tank.iter().chain(&other).copied().collect();
            if fits(&both) {
                tank = both;
                continue 'grow;
            }
        }
        return tank;
    }
}

/// `cells` parted into tanks that each `fits`: from the lowest block up, each not yet in a tank
/// starts one and grows over the blocks not yet in one.
pub fn part(cells: &[Pos], fits: impl Fn(&[Pos]) -> bool) -> Vec<Vec<Pos>> {
    let mut cells = cells.to_vec();
    cells.sort_by_key(|cell| (cell.y, cell.x, cell.z));
    let mut taken = HashSet::new();
    let mut tanks = Vec::new();
    for &cell in &cells {
        if taken.contains(&cell) {
            continue;
        }
        let free = |pos: Pos| (cells.contains(&pos) && !taken.contains(&pos)).then(|| vec![pos]);
        let tank = grow(vec![cell], free, &fits);
        taken.extend(tank.iter().copied());
        tanks.push(tank);
    }
    tanks
}

/// The blocks beside `cells` that are not among them, in the same order every time.
fn around(cells: &[Pos]) -> Vec<Pos> {
    let mut beside: Vec<Pos> = cells
        .iter()
        .flat_map(|&cell| Dir::ALL.map(|dir| cell.toward(dir)))
        .filter(|pos| !cells.contains(pos))
        .collect();
    beside.sort_by_key(|pos| (pos.y, pos.x, pos.z));
    beside.dedup();
    beside
}

/// How much each block of a tank holds when `total` lies in it: `layers` gives the number of
/// blocks of each layer, bottom first, and the answer a block's share in the same order, each
/// layer's blocks in turn.
pub fn spread(layers: &[usize], total: u32, per_block: u32) -> Vec<u32> {
    let mut left = total;
    let mut shares = Vec::new();
    for &blocks in layers {
        let layer = left.min(per_block * blocks as u32);
        left -= layer;
        let (each, over) = (layer / blocks as u32, layer % blocks as u32);
        shares.extend((0..blocks as u32).map(|i| each + u32::from(i < over)));
    }
    shares
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(x: i32, y: i32, z: i32) -> Pos {
        Pos { x, y, z }
    }

    #[test]
    fn a_whole_box_is_one_tank() {
        let column: Vec<Pos> = (0..HIGH).map(|y| at(0, y, 0)).collect();
        assert!(forms_box(&column), "as tall as a tank gets");
        let square: Vec<Pos> = (0..2)
            .flat_map(|x| (0..2).map(move |z| at(x, 0, z)))
            .collect();
        assert!(forms_box(&square), "two by two");
        assert!(forms_box(&[at(5, 5, 5)]), "a block alone");
    }

    #[test]
    fn a_box_with_a_gap_or_too_big_is_none() {
        let corner = [at(0, 0, 0), at(1, 0, 0), at(0, 0, 1)];
        assert!(!forms_box(&corner), "an L, not a box");
        let tall: Vec<Pos> = (0..=HIGH).map(|y| at(0, y, 0)).collect();
        assert!(!forms_box(&tall), "taller than a tank");
        let wide: Vec<Pos> = (0..=WIDE).map(|x| at(x, 0, 0)).collect();
        assert!(!forms_box(&wide), "wider than a tank");
    }

    /// The blocks from `from` to `to`, both in.
    fn block(from: Pos, to: Pos) -> Vec<Pos> {
        let mut cells = Vec::new();
        for x in from.x..=to.x {
            for y in from.y..=to.y {
                for z in from.z..=to.z {
                    cells.push(at(x, y, z));
                }
            }
        }
        cells
    }

    fn sorted(mut cells: Vec<Pos>) -> Vec<Pos> {
        cells.sort_by_key(|cell| (cell.y, cell.x, cell.z));
        cells
    }

    #[test]
    fn a_block_that_does_not_fit_leaves_the_tank_beside_it_whole() {
        let tank = block(at(0, 0, 0), at(2, 0, 2));
        let tank_at = |pos: Pos| tank.contains(&pos).then(|| tank.clone());
        let on_top = grow(vec![at(1, 1, 1)], tank_at, forms_box);
        assert_eq!(on_top, [at(1, 1, 1)], "a tank of its own on top");
        let beside = grow(vec![at(3, 0, 0)], tank_at, forms_box);
        assert_eq!(beside, [at(3, 0, 0)], "and beside");
    }

    #[test]
    fn tanks_built_a_row_at_a_time_grow_into_one() {
        let row = block(at(0, 0, 0), at(1, 0, 0));
        let other = at(0, 0, 1);
        let tank_at = |pos: Pos| {
            if row.contains(&pos) {
                Some(row.clone())
            } else {
                (pos == other).then(|| vec![other])
            }
        };
        // The fourth block fits only with the one beside it, and then the two with the row.
        let grown = grow(vec![at(1, 0, 1)], tank_at, forms_box);
        assert_eq!(
            sorted(grown),
            sorted(block(at(0, 0, 0), at(1, 0, 1))),
            "two by two"
        );
    }

    #[test]
    fn a_tank_with_a_block_gone_parts_into_boxes() {
        let ring: Vec<Pos> = block(at(0, 0, 0), at(2, 0, 2))
            .into_iter()
            .filter(|&cell| cell != at(1, 0, 1))
            .collect();
        let tanks = part(&ring, forms_box);
        assert!(tanks.iter().all(|tank| forms_box(tank)), "each a box");
        let all: Vec<Pos> = tanks.concat();
        assert_eq!(
            sorted(all),
            sorted(ring.clone()),
            "every block in one of them"
        );
        let column = [at(0, 0, 0), at(0, 1, 0), at(0, 3, 0)];
        assert_eq!(
            part(&column, forms_box).len(),
            2,
            "apart where the block was"
        );
    }

    #[test]
    fn a_tank_fills_from_the_bottom_up_each_layer_evenly() {
        let shares = spread(&[2, 2], 3000, 1000);
        assert_eq!(
            shares,
            [1000, 1000, 500, 500],
            "the bottom full, the top half"
        );
        let odd = spread(&[3], 1000, 1000);
        assert_eq!(odd, [334, 333, 333], "nothing lost to rounding");
        assert_eq!(
            spread(&[1], 5000, 1000),
            [1000],
            "never more than a block holds"
        );
    }
}
