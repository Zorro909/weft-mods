//! Cogwork's tickets: which cubes each keeps loaded, for whom and until when. A ticket runs out
//! unless renewed, and is renewed only while cogwork's tickets together stay within the world's
//! limit.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::*;

use super::Cubes;

/// Ticks a ticket lasts unless renewed, and how few of them may be left before it is: the
/// ticket of a block that is gone without being broken runs out on its own.
const LIFETIME: u32 = 100;
const RENEW_WITHIN: u64 = 40;

/// The engine's limit on the cubes all tickets keep together.
static MAX_FORCED: Config<i64> = Config::named("core:max_forced_cubes");

/// Who keeps cubes loaded, and so what runs in them; the lens colors them by it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, FromRepr)]
#[repr(u32)]
pub enum Keeper {
    /// A working excavator or leveler: machines only.
    Machine,
    /// A cuboid loader keeping machines running.
    Loader,
    /// A cuboid loader keeping everything running.
    LoaderAll,
}

impl Keeper {
    fn loading(self) -> Loading {
        match self {
            Keeper::LoaderAll => Loading::All,
            Keeper::Machine | Keeper::Loader => Loading::Machines,
        }
    }
}

/// What came of keeping a box of cubes; a loader's screen says `cogwork.loader.<name>`. Kept in
/// `Loader::kept` as its number.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Kept {
    #[default]
    Loading,
    /// Not kept: with cogwork's others it would pass the world's limit.
    OverLimit,
    /// Not kept: a mod refused it (someone else's claim).
    Refused,
}

/// What a ticket keeps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Ticket {
    dimension: u32,
    cubes: Cubes,
    keeper: Keeper,
}

/// A ticket and the game tick it runs out at.
type Tickets = HashMap<u64, (Ticket, u64)>;

/// What to do about a ticket wanted now.
#[derive(PartialEq, Eq, Debug)]
enum Plan {
    /// Kept as it is for a while yet.
    Nothing,
    /// New, changed or running out.
    Renew,
    OverLimit,
}

static TICKETS: LazyLock<Mutex<Tickets>> = LazyLock::new(Default::default);

/// Keeps `cubes` loaded for `keeper` under ticket `key`, renewed as it runs out.
pub fn keep(w: &World, key: u64, cubes: Cubes, keeper: Keeper) -> Kept {
    let now = w.game_time();
    let wanted = Ticket {
        dimension: w.dimension(),
        cubes,
        keeper,
    };
    // Planned apart from the match, so the lock is let go before `release` takes it.
    let planned = plan(&mut lock(), key, wanted, now, limit());
    match planned {
        Plan::Nothing => return Kept::Loading,
        Plan::OverLimit => {
            release(w, key);
            return Kept::OverLimit;
        }
        Plan::Renew => {}
    }
    let (from, to) = cubes.blocks();
    if !w.keep_loaded(key, from, to, keeper.loading(), LIFETIME) {
        lock().remove(&key);
        return Kept::Refused;
    }
    lock().insert(key, (wanted, now + LIFETIME as u64));
    Kept::Loading
}

/// Whether to renew ticket `key` for `wanted` at game tick `now`, with `limit` cubes kept in
/// all at most.
fn plan(tickets: &mut Tickets, key: u64, wanted: Ticket, now: u64, limit: i32) -> Plan {
    forget_stale(tickets, now);
    if let Some(&(held, until)) = tickets.get(&key)
        && held == wanted
        && until > now + RENEW_WITHIN
    {
        return Plan::Nothing;
    }
    let others: i32 = tickets
        .iter()
        .filter(|&(&k, _)| k != key)
        .map(|(_, (t, _))| t.cubes.count())
        .sum();
    if others + wanted.cubes.count() > limit {
        return Plan::OverLimit;
    }
    Plan::Renew
}

/// Drops tickets run out, and any from another world played before, whose ticks are ahead.
fn forget_stale(tickets: &mut Tickets, now: u64) {
    tickets.retain(|_, &mut (_, until)| until > now && until <= now + LIFETIME as u64);
}

/// Lets ticket `key` go, if cogwork holds it; whether it did.
pub fn release(w: &World, key: u64) -> bool {
    let held = lock().remove(&key).is_some();
    if held {
        w.release_loaded(key);
    }
    held
}

/// Lets ticket `key` go even when cogwork doesn't hold it: one restored with the world is not
/// noted again until it is renewed.
pub fn release_any(w: &World, key: u64) {
    lock().remove(&key);
    w.release_loaded(key);
}

/// Cubes cogwork's tickets keep in all, counted like the engine counts them: each ticket's own.
pub fn used(now: u64) -> i32 {
    let mut tickets = lock();
    forget_stale(&mut tickets, now);
    tickets.values().map(|(t, _)| t.cubes.count()).sum()
}

/// The world's limit on the cubes all tickets keep together.
pub fn limit() -> i32 {
    MAX_FORCED.get().clamp(0, i32::MAX.into()) as i32
}

/// The boxes kept loaded in `dimension`, and who keeps each.
pub fn shown(dimension: u32, now: u64) -> Vec<(Cubes, Keeper)> {
    let mut tickets = lock();
    forget_stale(&mut tickets, now);
    tickets
        .values()
        .filter(|(t, _)| t.dimension == dimension)
        .map(|(t, _)| (t.cubes, t.keeper))
        .collect()
}

fn lock() -> MutexGuard<'static, Tickets> {
    TICKETS.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lib::Pos;

    const NOW: u64 = 1000;

    fn ticket(range: i32) -> Ticket {
        Ticket {
            dimension: 0,
            cubes: Cubes::around(Pos::new(0, 64, 0), range),
            keeper: Keeper::Loader,
        }
    }

    #[test]
    fn a_ticket_is_renewed_only_when_changed_or_running_out() {
        let mut tickets = Tickets::from([(1, (ticket(0), NOW + 90))]);
        let plan_now = |tickets: &mut Tickets, t| plan(tickets, 1, t, NOW, 4096);
        assert_eq!(plan_now(&mut tickets, ticket(0)), Plan::Nothing, "fresh");
        assert_eq!(plan_now(&mut tickets, ticket(1)), Plan::Renew, "grown");
        let everything = Ticket {
            keeper: Keeper::LoaderAll,
            ..ticket(0)
        };
        assert_eq!(plan_now(&mut tickets, everything), Plan::Renew, "switched");
        tickets.insert(1, (ticket(0), NOW + RENEW_WITHIN));
        assert_eq!(
            plan_now(&mut tickets, ticket(0)),
            Plan::Renew,
            "running out"
        );
    }

    #[test]
    fn the_limit_counts_the_other_tickets_not_its_own() {
        let mut tickets = Tickets::from([(1, (ticket(3), NOW + 10)), (2, (ticket(3), NOW + 10))]);
        assert_eq!(
            plan(&mut tickets, 2, ticket(3), NOW, 98),
            Plan::Renew,
            "49 + 49"
        );
        assert_eq!(
            plan(&mut tickets, 3, ticket(0), NOW, 98),
            Plan::OverLimit,
            "one more"
        );
    }

    #[test]
    fn stale_tickets_are_forgotten() {
        let mut tickets = Tickets::from([
            (1, (ticket(0), NOW)),
            (2, (ticket(0), NOW + 1)),
            (3, (ticket(0), NOW + LIFETIME as u64 + 1)),
        ]);
        forget_stale(&mut tickets, NOW);
        assert!(!tickets.contains_key(&1), "run out");
        assert!(tickets.contains_key(&2), "still kept");
        assert!(!tickets.contains_key(&3), "from another world");
    }

    #[test]
    fn every_outcome_has_a_name() {
        for &kept in Kept::VARIANTS {
            let key = format!("cogwork.loader.{}", <&str>::from(kept));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }
}
