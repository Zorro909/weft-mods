//! A core drill's survey, with no world: which block it counts next, how far its turns have
//! drilled, and its tally of ore by kind and band. It counts the 16 by 16 square around the drill,
//! layer by layer from the one under it down to 128 below, in bands of 16 layers. Each turn drills
//! half a band, so sixteen turns reach the bottom.

use super::Core;
use crate::lib::Pos;

/// Blocks across the square it counts, layers in a band, and bands.
pub const SIDE: i32 = 16;
pub const BAND: i32 = 16;
pub const BANDS: usize = 8;
/// Kinds of ore it tells apart; more are left out of its report.
pub const KINDS: usize = 8;
/// Turns that drill all the way down: two a band.
pub const TURNS: u32 = 16;
/// Blocks in a layer of the square, and in the whole survey.
const LAYER: u32 = (SIDE * SIDE) as u32;
pub const TOTAL: u32 = LAYER * BAND as u32 * BANDS as u32;

/// The block the `index`th count looks at, below a drill at `at`: layer by layer from the one
/// under it, each row by row from the north-west corner of the square around it.
pub fn cell(at: Pos, index: u32) -> Pos {
    let (layer, within) = ((index / LAYER) as i32, (index % LAYER) as i32);
    let half = SIDE / 2;
    Pos::new(
        at.x - half + within % SIDE,
        at.y - 1 - layer,
        at.z - half + within / SIDE,
    )
}

/// The first and last block of the layer the `index`th count lies in.
pub fn layer_of(at: Pos, index: u32) -> (Pos, Pos) {
    let first = index - index % LAYER;
    (cell(at, first), cell(at, first + LAYER - 1))
}

/// The band the `index`th count lies in.
pub fn band(index: u32) -> usize {
    (index / (LAYER * BAND as u32)) as usize
}

/// Blocks the drill has reached after `turns`.
pub fn reached(turns: u32) -> u32 {
    TOTAL / TURNS * turns.min(TURNS)
}

/// Layers below the drill its bit has reached after `turns`.
pub fn depth(turns: u32) -> i32 {
    reached(turns) as i32 / LAYER as i32
}

impl Core {
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
        self.counts[slot * BANDS + band] += 1;
    }

    /// Whether it has counted everything down to the bottom.
    pub fn is_done(&self) -> bool {
        self.counted >= TOTAL
    }

    /// Every kind it found: the key of the name of the block that shows it, and its counts by
    /// band from the top.
    pub fn found(&self) -> Vec<(u64, [u32; BANDS])> {
        let kinds = self.kinds.iter().zip(&self.shown).enumerate();
        let found = kinds.filter(|(_, (kind, _))| **kind != 0);
        found
            .map(|(slot, (_, &shown))| {
                let counts = &self.counts[slot * BANDS..(slot + 1) * BANDS];
                (shown, counts.try_into().unwrap_or_default())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_counts_the_square_around_it_layer_by_layer() {
        let at = Pos::new(100, 64, -20);
        assert_eq!(
            cell(at, 0),
            Pos::new(92, 63, -28),
            "north-west, just under it"
        );
        assert_eq!(cell(at, 15), Pos::new(107, 63, -28), "along the first row");
        assert_eq!(
            cell(at, LAYER - 1),
            Pos::new(107, 63, -13),
            "the layer's last"
        );
        assert_eq!(
            cell(at, LAYER),
            Pos::new(92, 62, -28),
            "the next layer down"
        );
        assert_eq!(cell(at, TOTAL - 1).y, 64 - 128, "128 below at the bottom");
    }

    #[test]
    fn each_band_is_sixteen_layers() {
        assert_eq!(band(0), 0, "the top band");
        assert_eq!(band(LAYER * 16 - 1), 0, "to its sixteenth layer");
        assert_eq!(band(LAYER * 16), 1, "then the next");
        assert_eq!(band(TOTAL - 1), BANDS - 1, "the bottom band");
    }

    #[test]
    fn two_turns_drill_a_band() {
        assert_eq!(depth(0), 0, "not drilled yet");
        assert_eq!(depth(2), 16, "a band");
        assert_eq!(depth(TURNS), 128, "the bottom");
        assert_eq!(reached(TURNS + 5), TOTAL, "no further");
    }

    #[test]
    fn the_layer_of_a_count_spans_its_square() {
        let at = Pos::new(0, 0, 0);
        let (first, last) = layer_of(at, LAYER + 37);
        assert_eq!((first, last), (Pos::new(-8, -2, -8), Pos::new(7, -2, 7)));
    }

    #[test]
    fn ore_is_tallied_by_kind_and_band() {
        let mut core = Core::default();
        core.tally(11, 101, 0);
        core.tally(11, 102, 3);
        core.tally(12, 201, 3);
        let found = core.found();
        assert_eq!(found.len(), 2, "two kinds");
        assert_eq!(found[0].0, 101, "shown by the first block found");
        assert_eq!(
            found[0].1[..4],
            [1, 0, 0, 1],
            "one at the top, one in band 3"
        );
        assert_eq!(found[1].1[3], 1, "the second kind");
    }

    #[test]
    fn kinds_past_the_eighth_are_left_out() {
        let mut core = Core::default();
        for kind in 1..=KINDS as u64 + 2 {
            core.tally(kind, kind, 0);
        }
        assert_eq!(core.found().len(), KINDS, "eight kinds");
        core.tally(1, 1, 0);
        assert_eq!(core.found()[0].1[0], 2, "a known kind still counts");
    }

    #[test]
    fn a_survey_is_done_once_it_has_counted_the_bottom() {
        let mut core = Core::default();
        assert!(!core.is_done(), "a new drill");
        core.counted = TOTAL;
        assert!(core.is_done(), "down to 128 below");
    }
}
