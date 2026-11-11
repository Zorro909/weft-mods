//! How much liquid goes from one holder to another: what evens out how full the two are. A
//! holder twice the size takes twice as much to be as full, so a big tank between a pump and a
//! tube fills slowly and empties slowly.

/// How full a holder is: millibuckets in it, of how many it holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Level {
    pub amount: u32,
    pub capacity: u32,
}

/// Millibuckets that go from `from` to `to` in one turn: as many as leave both equally full,
/// rounded down so that two holders a millibucket apart don't pass it to and fro, and at most
/// `max`. None when `to` is as full as `from` or fuller.
pub fn share(from: Level, to: Level, max: u32) -> u32 {
    let (a, b) = (from.amount as u64, to.amount as u64);
    let (ca, cb) = (from.capacity as u64, to.capacity as u64);
    // From ends up holding a share `ca / (ca + cb)` of both: what it has beyond that goes.
    // Never more than `to` has room for: it ends up no fuller than `from`, so at most full.
    let beyond = (a * cb).saturating_sub(b * ca) / (ca + cb).max(1);
    beyond.min(max as u64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(amount: u32, capacity: u32) -> Level {
        Level { amount, capacity }
    }

    #[test]
    fn two_equal_holders_end_up_equally_full() {
        assert_eq!(
            share(level(800, 1000), level(200, 1000), 1000),
            300,
            "half the difference"
        );
        assert_eq!(
            share(level(200, 1000), level(800, 1000), 1000),
            0,
            "never uphill"
        );
    }

    #[test]
    fn a_bigger_holder_takes_more_to_be_as_full() {
        let moved = share(level(1000, 1000), level(0, 3000), 1000);
        assert_eq!(moved, 750, "both a quarter full after");
    }

    #[test]
    fn a_last_millibucket_stays_put() {
        assert_eq!(
            share(level(1, 1000), level(0, 1000), 1000),
            0,
            "no to and fro"
        );
        assert_eq!(
            share(level(2, 1000), level(0, 1000), 1000),
            1,
            "two even out"
        );
    }

    #[test]
    fn a_turn_moves_no_more_than_its_most() {
        assert_eq!(
            share(level(1000, 1000), level(0, 1000), 100),
            100,
            "the most a turn"
        );
    }

    #[test]
    fn a_full_holder_overfills_nothing() {
        let into_nearly_full = share(level(8000, 8000), level(990, 1000), 1000);
        assert!(into_nearly_full <= 10, "only what room is left");
    }
}
