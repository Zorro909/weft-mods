//! Plots: who may claim which (`may_claim`), and `/plot`, with which players claim, share and
//! give up their plots and moderators hand them out. The blocks are `post.rs`.

mod commands;
pub mod post;

use weft_sdk::Registry;
use weftkit::{PlayerKey, Pos};

use crate::city::{Plot, PlotSize, cell_at};
use crate::settings;
use crate::tables::{PLAYERS, PLOTS};

pub fn register(r: &mut Registry) {
    post::register(r);
    commands::register(r);
}

/// The plot holding the block `pos` of dimension `dim`, with its number.
pub fn at(dim: u64, pos: Pos) -> Option<(u32, Plot)> {
    let id = cell_at(dim, pos)?.plot?;
    Some((id, PLOTS.get(&id)?))
}

/// The plots `key` owns, with their numbers.
pub fn owned_by(key: &PlayerKey) -> Vec<(u32, Plot)> {
    let mut owned = Vec::new();
    PLOTS.for_each(|id, plot| {
        if plot.owner.as_ref() == Some(key) {
            owned.push((id, plot));
        }
    });
    owned
}

/// Normal plots `key` may claim: what a moderator gave them, else the setting.
pub fn allowance_of(key: &PlayerKey) -> u32 {
    PLAYERS
        .get(key)
        .and_then(|entry| entry.plot_allowance)
        .unwrap_or(settings::get().plot_allowance)
}

/// Whether a player owning `claimed` (the plots they claimed themselves, not those given to
/// them) may claim a plot of `size`. Everyone has one small plot free; every other small or
/// normal plot counts against `allowance`; large plots are only given.
pub fn may_claim(size: PlotSize, claimed: &[PlotSize], allowance: u32) -> Result<(), &'static str> {
    if size == PlotSize::Large {
        return Err("plotty.plot.large_given");
    }
    let smalls = claimed
        .iter()
        .filter(|size| **size == PlotSize::Small)
        .count() as u32;
    if size == PlotSize::Small && smalls == 0 {
        return Ok(());
    }
    let counted = claimed.len() as u32 - smalls.min(1);
    if counted < allowance {
        Ok(())
    } else {
        Err("plotty.plot.allowance_used")
    }
}

#[cfg(test)]
mod tests {
    use PlotSize::{Large, Normal, Small};

    use super::*;

    #[test]
    fn one_small_plot_is_free_besides_the_allowance() {
        assert_eq!(
            may_claim(Small, &[], 0),
            Ok(()),
            "free even with no allowance"
        );
        assert_eq!(
            may_claim(Normal, &[Small], 1),
            Ok(()),
            "the normal one on top"
        );
        assert_eq!(
            may_claim(Small, &[Small, Normal], 1),
            Err("plotty.plot.allowance_used"),
            "a second small one counts"
        );
    }

    #[test]
    fn the_allowance_counts_normal_plots() {
        assert_eq!(
            may_claim(Normal, &[Normal], 1),
            Err("plotty.plot.allowance_used"),
            "one is the default"
        );
        assert_eq!(
            may_claim(Normal, &[Normal], 2),
            Ok(()),
            "a raised allowance"
        );
    }

    #[test]
    fn large_plots_are_only_given() {
        assert_eq!(
            may_claim(Large, &[], 9),
            Err("plotty.plot.large_given"),
            "never claimed"
        );
    }
}
