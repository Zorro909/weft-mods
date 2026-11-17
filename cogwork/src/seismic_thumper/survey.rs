//! A thumper's survey, with no world: the area it counts, which block it counts next, and its
//! tally of ore by kind and band. It counts the columns of its area layer by layer, from the one
//! under it down to `BANDS` bands of 16 layers or the world's lowest layer, whichever comes
//! first.

use super::Survey;
use crate::lib::{Area, Pos};

/// Layers in a band, and bands it counts at most: 160 layers.
pub const BAND: i32 = 16;
pub const BANDS: usize = 10;
/// Kinds of ore it tells apart; more are left out of its screen.
pub const KINDS: usize = 8;
/// Blocks across its area without survey markers, around it, and the most with them.
pub const DEFAULT_SIDE: i32 = 32;
pub const LARGEST_SIDE: i32 = 64;

/// The columns a thumper at `pos` surveys: the rectangle of the survey markers beside it, cropped
/// to `LARGEST_SIDE` across at the end nearest it, or the square of `DEFAULT_SIDE` around it.
pub fn columns(marked: Option<Area>, pos: Pos) -> Area {
    let Some(marked) = marked else {
        let half = DEFAULT_SIDE / 2;
        let lo = Pos::new(pos.x - half, pos.y, pos.z - half);
        return Area::sized(lo, [DEFAULT_SIDE, 1, DEFAULT_SIDE]);
    };
    let side = |lo: i32, hi: i32, near: i32| {
        if hi - lo < LARGEST_SIDE {
            (lo, hi)
        } else if (near - lo).abs() <= (hi - near).abs() {
            (lo, lo + LARGEST_SIDE - 1)
        } else {
            (hi - LARGEST_SIDE + 1, hi)
        }
    };
    let (lo_x, hi_x) = side(marked.lo.x, marked.hi.x, pos.x);
    let (lo_z, hi_z) = side(marked.lo.z, marked.hi.z, pos.z);
    Area::between(Pos::new(lo_x, pos.y, lo_z), Pos::new(hi_x, pos.y, hi_z))
}

/// Layers a thumper at height `y` counts, down to `lowest_y` at most.
pub fn layers(y: i32, lowest_y: i32) -> u32 {
    let deepest = BAND * BANDS as i32;
    (y - lowest_y).clamp(0, deepest) as u32
}

impl Survey {
    /// Blocks in one layer of its area.
    fn layer_size(&self) -> u32 {
        let [x, _, z] = self.area().size();
        (x * z) as u32
    }

    /// Blocks it counts in all.
    pub fn total(&self) -> u32 {
        self.layer_size() * self.layers
    }

    /// The block its `index`th count looks at: layer by layer from the one under it, each row by
    /// row from its north-west corner.
    pub fn cell(&self, index: u32) -> Pos {
        let area = self.area();
        let size = self.layer_size().max(1);
        let (layer, within) = ((index / size) as i32, (index % size) as i32);
        let across = area.size()[0];
        Pos::new(
            area.lo.x + within % across,
            area.lo.y - 1 - layer,
            area.lo.z + within / across,
        )
    }

    /// The first and last block of the layer its `index`th count lies in.
    pub fn layer_of(&self, index: u32) -> (Pos, Pos) {
        let size = self.layer_size().max(1);
        let first = index - index % size;
        (self.cell(first), self.cell(first + size - 1))
    }

    /// The band its `index`th count lies in.
    pub fn band(&self, index: u32) -> usize {
        let layer = index / self.layer_size().max(1);
        (layer / BAND as u32) as usize
    }

    /// Bands its layers reach into.
    pub fn bands(&self) -> usize {
        (self.layers as usize).div_ceil(BAND as usize).min(BANDS)
    }

    /// Counts an ore block of the kind keyed `kind`, shown by the block whose name is keyed
    /// `shown`, in `band`. A kind past the first `KINDS` found is left out.
    pub fn tally(&mut self, kind: u64, shown: u64, band: usize) {
        let slot = match self.kinds.iter().position(|&known| known == kind) {
            Some(slot) => slot,
            None => match self.kinds.iter().position(|&known| known == 0) {
                Some(free) => {
                    (self.kinds[free], self.shown[free]) = (kind, shown);
                    free
                }
                None => return,
            },
        };
        self.counts[slot * BANDS + band.min(BANDS - 1)] += 1;
    }

    /// Its count of the `slot`th kind it found in `band`.
    pub fn count(&self, slot: usize, band: usize) -> u32 {
        self.counts[slot * BANDS + band]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn survey(lo: Pos, hi: Pos, layers: u32) -> Survey {
        let mut survey = Survey::default();
        survey.set_area(Area::between(lo, hi), layers);
        survey
    }

    #[test]
    fn without_markers_it_surveys_the_square_around_it() {
        let pos = Pos::new(100, 64, -20);
        let area = columns(None, pos);
        assert_eq!(area.size(), [32, 1, 32], "32 by 32");
        assert!(area.spans(pos), "with itself in it");
    }

    #[test]
    fn markers_wider_than_it_takes_are_cropped_at_its_end() {
        let pos = Pos::new(-1, 70, 0);
        let marked = Area::between(Pos::new(0, 70, 0), Pos::new(99, 70, 20));
        let area = columns(Some(marked), pos);
        assert_eq!(area.lo, Pos::new(0, 70, 0), "from its end");
        assert_eq!(
            area.size(),
            [64, 1, 21],
            "64 across, as marked the other way"
        );
        let far = columns(Some(marked), Pos::new(100, 70, 0));
        assert_eq!(far.hi.x, 99, "or the other end, nearer it");
    }

    #[test]
    fn it_counts_layer_by_layer_row_by_row() {
        let area = survey(Pos::new(10, 64, 20), Pos::new(13, 64, 21), 5);
        assert_eq!(area.total(), 4 * 2 * 5, "8 a layer, 5 layers");
        assert_eq!(
            area.cell(0),
            Pos::new(10, 63, 20),
            "north-west, just under it"
        );
        assert_eq!(area.cell(3), Pos::new(13, 63, 20), "along the first row");
        assert_eq!(area.cell(4), Pos::new(10, 63, 21), "the next row");
        assert_eq!(area.cell(8), Pos::new(10, 62, 20), "the next layer");
        let (first, last) = area.layer_of(9);
        assert_eq!((first, last), (Pos::new(10, 62, 20), Pos::new(13, 62, 21)));
    }

    #[test]
    fn each_band_is_sixteen_layers() {
        let area = survey(Pos::new(0, 64, 0), Pos::new(1, 64, 0), 40);
        assert_eq!(area.band(0), 0, "the top band");
        assert_eq!(area.band(2 * 16 - 1), 0, "to its sixteenth layer");
        assert_eq!(area.band(2 * 16), 1, "then the next");
        assert_eq!(area.bands(), 3, "40 layers reach into three");
    }

    #[test]
    fn it_counts_down_to_the_lowest_layer_or_its_last_band() {
        assert_eq!(layers(64, -32), 96, "to the floor");
        assert_eq!(layers(200, -64), 160, "ten bands at most");
        assert_eq!(layers(-40, -32), 0, "nothing under the floor");
    }

    #[test]
    fn ore_is_tallied_by_kind_and_band() {
        let mut area = Survey::default();
        area.tally(11, 101, 0);
        area.tally(11, 102, 3);
        area.tally(12, 201, 3);
        assert_eq!(area.shown[0], 101, "shown by the first block found");
        assert_eq!([area.count(0, 0), area.count(0, 3)], [1, 1], "both bands");
        assert_eq!(area.count(1, 3), 1, "the second kind");
        for kind in 13..=20 {
            area.tally(kind, kind, 0);
        }
        assert_eq!(area.kinds.iter().filter(|&&kind| kind != 0).count(), KINDS);
        area.tally(11, 101, 0);
        assert_eq!(area.count(0, 0), 2, "a known kind still counts");
    }
}
