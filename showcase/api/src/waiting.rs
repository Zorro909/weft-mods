//! Sets put off to the next tick, for those built on ground `Set::level` makes: a fill happens
//! at the end of the tick it was asked in, and would take whatever was built before then.

use std::sync::{Mutex, MutexGuard};

use weft_sdk::World;

use crate::{Pos, Set};

/// Sets to build the tick after they were asked for, each with what building it needs.
pub struct Waiting<T> {
    sets: Mutex<Vec<Asked<T>>>,
}

struct Asked<T> {
    player: u64,
    origin: Pos,
    /// The tick it was asked in.
    tick: u64,
    with: T,
}

impl<T> Waiting<T> {
    pub const fn new() -> Waiting<T> {
        Waiting {
            sets: Mutex::new(Vec::new()),
        }
    }

    /// Puts off building `set` to a later tick, when `ready` gives it back with `with`.
    pub fn push(&self, set: &Set, with: T) {
        self.lock().push(Asked {
            player: set.player(),
            origin: set.origin(),
            tick: set.world().game_time(),
            with,
        });
    }

    /// The sets in `world` that were asked for before this tick, taken out to be built: from
    /// `r.world_tick`.
    pub fn ready(&self, world: &World) -> Vec<(Set, T)> {
        let here = |player| world.entity(player).is_some();
        self.take_ready(world.game_time(), here)
            .into_iter()
            .map(|asked| (Set::new(*world, asked.player, asked.origin), asked.with))
            .collect()
    }

    /// Those asked for before tick `now` by players `here` says are in this world.
    fn take_ready(&self, now: u64, here: impl Fn(u64) -> bool) -> Vec<Asked<T>> {
        self.lock()
            .extract_if(.., |asked| asked.tick < now && here(asked.player))
            .collect()
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Asked<T>>> {
        // Still usable after a panic elsewhere left it poisoned.
        self.sets.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl<T> Default for Waiting<T> {
    fn default() -> Waiting<T> {
        Waiting::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asked(tick: u64, player: u64) -> Asked<()> {
        Asked {
            player,
            origin: Pos::default(),
            tick,
            with: (),
        }
    }

    #[test]
    fn a_set_waits_until_the_tick_after_it_was_asked_for() {
        let waiting = Waiting::new();
        waiting.lock().push(asked(10, 1));
        assert!(
            waiting.take_ready(10, |_| true).is_empty(),
            "not in its own tick"
        );
        assert_eq!(waiting.take_ready(11, |_| true).len(), 1, "in the next");
        assert!(
            waiting.take_ready(12, |_| true).is_empty(),
            "built only once"
        );
    }

    #[test]
    fn a_set_is_built_where_its_player_is() {
        let waiting = Waiting::new();
        waiting.lock().push(asked(10, 1));
        assert!(
            waiting.take_ready(11, |player| player != 1).is_empty(),
            "elsewhere"
        );
        assert_eq!(
            waiting.take_ready(11, |player| player == 1).len(),
            1,
            "here"
        );
    }
}
