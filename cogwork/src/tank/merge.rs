//! Which tank blocks make one tank, and how its liquid lies in it. Tank blocks next to each
//! other are one tank when together they fill a box no more than `WIDE` across either way and
//! `HIGH` tall, holding no more than one liquid; otherwise each is a tank of its own. A tank fills
//! from its bottom layer up, a layer's blocks equally.

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
