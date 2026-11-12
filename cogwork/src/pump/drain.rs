//! What a pump finds below its hose: how many sources of the liquid there it can reach, and
//! which it takes next, the one farthest from the hose along the liquid, so the pool shrinks
//! from its edges toward the hose rather than being holed under it.

use std::collections::{HashSet, VecDeque};

use weft_sdk::Dir;

use crate::lib::Pos;
use crate::liquid::LiquidKey;

/// Blocks along the liquid from the hose a pump takes sources at, at most.
pub const REACH: u32 = 32;
/// Blocks of liquid looked at for one bucket at most: a sea is no lake.
const SEARCH_MOST: usize = 2048;

/// The sources of a liquid a pump reaches.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Pool {
    /// How many, as far as the search went.
    pub sources: usize,
    /// The one farthest from the hose, the next to take.
    pub farthest: Option<Pos>,
}

/// The sources of `liquid` reached from `intake` along it, within `REACH`; `liquid_at` says what
/// liquid a block is, and whether a source of it.
pub fn survey(
    intake: Pos,
    liquid: LiquidKey,
    liquid_at: impl Fn(Pos) -> Option<(LiquidKey, bool)>,
) -> Pool {
    let is_ours = |pos: Pos| liquid_at(pos).filter(|&(found, _)| found == liquid);
    let mut pool = Pool::default();
    if is_ours(intake).is_none() {
        return pool;
    }
    let mut queue = VecDeque::from([(intake, 0)]);
    let mut seen = HashSet::from([intake]);
    while let Some((pos, steps)) = queue.pop_front() {
        if is_ours(pos).is_some_and(|(_, source)| source) {
            pool.sources += 1;
            pool.farthest = Some(pos);
        }
        if steps == REACH || seen.len() >= SEARCH_MOST {
            continue;
        }
        for dir in Dir::ALL {
            let next = pos.toward(dir);
            if is_ours(next).is_some() && seen.insert(next) {
                queue.push_back((next, steps + 1));
            }
        }
    }
    pool
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    const OIL: LiquidKey = 7;
    const WATER: LiquidKey = 8;

    fn at(x: i32, y: i32, z: i32) -> Pos {
        Pos { x, y, z }
    }

    /// A row of oil along x from 0, sources where `sources` says.
    fn row(length: i32, sources: impl Fn(i32) -> bool) -> HashMap<Pos, (LiquidKey, bool)> {
        (0..length)
            .map(|x| (at(x, 0, 0), (OIL, sources(x))))
            .collect()
    }

    #[test]
    fn the_farthest_source_goes_first() {
        let pool = row(5, |_| true);
        let found = survey(at(0, 0, 0), OIL, |pos| pool.get(&pos).copied());
        assert_eq!(found.farthest, Some(at(4, 0, 0)), "the far end of the row");
        assert_eq!(found.sources, 5, "every source counted");
    }

    #[test]
    fn flowing_liquid_leads_on_to_sources() {
        let pool = row(5, |x| x == 2);
        let found = survey(at(0, 0, 0), OIL, |pos| pool.get(&pos).copied());
        assert_eq!(
            found.farthest,
            Some(at(2, 0, 0)),
            "the one source, through flowing oil"
        );
    }

    #[test]
    fn another_liquid_or_none_at_the_hose_gives_nothing() {
        let pool = row(5, |_| true);
        let found = survey(at(0, 0, 0), WATER, |pos| pool.get(&pos).copied());
        assert_eq!(found, Pool::default(), "oil is no water");
        let dry = survey(at(9, 9, 9), OIL, |pos| pool.get(&pos).copied());
        assert_eq!(dry, Pool::default(), "nothing at the hose");
    }

    #[test]
    fn sources_beyond_the_reach_stay() {
        let pool = row(REACH as i32 + 10, |_| true);
        let found = survey(at(0, 0, 0), OIL, |pos| pool.get(&pos).copied());
        assert_eq!(
            found.farthest,
            Some(at(REACH as i32, 0, 0)),
            "as far as it reaches"
        );
    }
}
