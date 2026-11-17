//! A thumper's survey, with no world: the area it counts, which block it counts next, and its
//! tally of ore by kind and band. It counts the columns of its area layer by layer, from the one
//! under it down to `BANDS` bands of 16 layers; where the world ends sooner, a layer that never
//! loads ends the count (`work`). Depths are worked out in `i64`, so no height overflows.

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

/// Layers a thumper at height `y` counts: `BANDS` bands, or as many as there are heights below it.
pub fn layers(y: i32) -> u32 {
    let deepest = i64::from(BAND) * BANDS as i64;
    let below = i64::from(y) - i64::from(i32::MIN);
    below.min(deepest) as u32
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

    /// The height of the layer just under it, where it counts first.
    pub fn top(&self) -> i64 {
        i64::from(self.lo.y) - 1
    }

    /// The height of the last layer it counts.
    pub fn bottom(&self) -> i64 {
        self.top() - i64::from(self.layers) + 1
    }

    /// The height of the first layer of `band`.
    pub fn band_top(&self, band: usize) -> i64 {
        self.top() - band as i64 * i64::from(BAND)
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
    fn it_counts_ten_bands_whatever_the_quarries_lowest_layer() {
        assert_eq!(layers(64), 160, "below 0 too");
        assert_eq!(layers(-2_000), 160, "deep down");
        assert_eq!(layers(i32::MAX), 160, "at the top of the world");
    }

    #[test]
    fn it_counts_no_deeper_than_the_lowest_height_there_is() {
        assert_eq!(layers(i32::MIN), 0, "nothing under the lowest");
        assert_eq!(layers(i32::MIN + 5), 5, "to the lowest");
        let lo = Pos::new(0, i32::MIN + 5, 0);
        let area = survey(lo, lo, layers(lo.y));
        assert_eq!(area.cell(area.total() - 1).y, i32::MIN, "its last block");
        assert_eq!(area.bottom(), i64::from(i32::MIN), "as its screen says");
        assert_eq!(area.band_top(0), i64::from(i32::MIN) + 4, "its one band");
    }

    #[test]
    fn its_screen_tells_its_layers_by_height() {
        let lo = Pos::new(0, 64, 0);
        let area = survey(lo, lo, layers(lo.y));
        assert_eq!((area.top(), area.bottom()), (63, -96), "160 layers");
        assert_eq!(area.band_top(1), 47, "the second band");
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
