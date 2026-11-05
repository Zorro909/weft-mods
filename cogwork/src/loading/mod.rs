//! Cubes cogwork keeps loaded with nobody near: those of its cuboid loaders, and of its
//! excavators and levelers while they work. The engine tells mods neither which cubes are kept
//! nor how many, so each ticket is noted here too: for the cuboid lens to show, and for loaders
//! to count against the world's limit, since the engine drops one past it without a word.

mod cubes;
mod lens;
mod loader;

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

pub use cubes::Cubes;
use strum::{FromRepr, IntoStaticStr, VariantArray};
use twox_hash::XxHash3_64;
use weft_sdk::*;

use crate::lib::Pos;

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

/// Which ticket of a block is meant: a block may hold more than one.
#[derive(Clone, Copy)]
#[repr(u8)]
enum Role {
    /// A loader's cubes, or a machine's own and those around it.
    Home,
    /// The cubes a machine works in.
    Work,
    /// The cubes between those two, when its work is above or below them.
    Link,
}

const ROLES: [Role; 3] = [Role::Home, Role::Work, Role::Link];

pub fn register(r: &mut Registry) {
    loader::register(r);
    lens::register(r);
}

/// The key of the ticket in `role` of the block at `pos`: the same for that block across
/// restarts, so its ticket can be let go when it breaks.
fn key(w: &World, pos: Pos, role: Role) -> u64 {
    let numbers = [w.dimension() as i32, pos.x, pos.y, pos.z, role as i32];
    let bytes: Vec<u8> = numbers.iter().flat_map(|n| n.to_le_bytes()).collect();
    XxHash3_64::oneshot(&bytes)
}

/// Keeps `cubes` loaded for `keeper` under ticket `key`, renewed as it runs out.
fn keep(w: &World, key: u64, cubes: Cubes, keeper: Keeper) -> Kept {
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
fn plan(tickets: &mut Tickets, key: u64, wanted: Ticket, now: u64, limit: i64) -> Plan {
    forget_stale(tickets, now);
    if let Some(&(held, until)) = tickets.get(&key)
        && held == wanted
        && until > now + RENEW_WITHIN
    {
        return Plan::Nothing;
    }
    let others: i64 = tickets
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

/// Lets ticket `key` go, if cogwork holds it.
fn release(w: &World, key: u64) {
    if lock().remove(&key).is_some() {
        w.release_loaded(key);
    }
}

/// Keeps what a working machine at `pos` needs loaded: its own cube and the eight around it,
/// and the cubes it works in, `work`, when they reach outside them.
///
/// A machine's `work` should not let go of a cube it comes back to later: unloaded, a cube is
/// written to disk at once, and read back from it when wanted again.
pub fn keep_machine(w: &World, pos: Pos, work: Option<Cubes>) {
    let home = Cubes::around(pos, 1);
    keep(w, key(w, pos, Role::Home), home, Keeper::Machine);
    let work = work.filter(|cubes| !home.contains(cubes));
    keep_or_release(w, key(w, pos, Role::Work), work);
    keep_or_release(w, key(w, pos, Role::Link), work.and_then(|c| link(pos, c)));
}

/// A column of the cubes between the machine's own at `pos` and the height of its `work`, if
/// any. A machine sees only the loaded cubes joined to its own, so work far below it, as an
/// excavator's, would be out of its reach once the cubes between unload.
fn link(pos: Pos, work: Cubes) -> Option<Cubes> {
    let own = Cubes::around(pos, 0).lo;
    let (lo, hi) = if work.lo.y > own.y {
        (own.y + 1, work.lo.y - 1)
    } else {
        (work.hi.y + 1, own.y - 1)
    };
    (lo <= hi).then(|| Cubes {
        lo: own.with_y(lo),
        hi: own.with_y(hi),
    })
}

fn keep_or_release(w: &World, key: u64, cubes: Option<Cubes>) {
    match cubes {
        Some(cubes) => drop(keep(w, key, cubes, Keeper::Machine)),
        None => release(w, key),
    }
}

/// Lets go of what the machine at `pos` kept loaded: it is done.
pub fn release_machine(w: &World, pos: Pos) {
    for role in ROLES {
        release(w, key(w, pos, role));
    }
}

/// A block that keeps cubes loaded was broken: its tickets go now, even ones restored with the
/// world that cogwork has not noted again yet.
pub fn let_go(w: &World, change: &BlockChange) {
    let pos = Pos::from(change.pos);
    for role in ROLES {
        let key = key(w, pos, role);
        lock().remove(&key);
        w.release_loaded(key);
    }
}

/// Cubes cogwork's tickets keep in all, counted like the engine counts them: each ticket's own.
fn used(now: u64) -> i64 {
    let mut tickets = lock();
    forget_stale(&mut tickets, now);
    tickets.values().map(|(t, _)| t.cubes.count()).sum()
}

fn limit() -> i64 {
    MAX_FORCED.get()
}

/// The boxes kept loaded in `dimension`, and who keeps each.
fn shown(dimension: u32, now: u64) -> Vec<(Cubes, Keeper)> {
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

/// For machines' tests: the first cube that `kept`, the boxes a machine keeps one after another,
/// lets go of and later keeps again.
#[cfg(test)]
pub fn kept_again(kept: impl IntoIterator<Item = Cubes>) -> Option<Pos> {
    use std::collections::HashSet;
    let (mut held, mut gone): (HashSet<Pos>, HashSet<Pos>) = Default::default();
    let mut last = None;
    for cubes in kept {
        if last == Some(cubes) {
            continue;
        }
        last = Some(cubes);
        if let Some(back) = cubes.each().find(|cube| gone.contains(cube)) {
            return Some(back);
        }
        let now: HashSet<Pos> = cubes.each().collect();
        gone.extend(held.difference(&now));
        held = now;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn work_far_below_is_linked_to_the_machine_by_the_cubes_between() {
        let pos = Pos::new(5, 320, 5);
        let below = Cubes::around(Pos::new(30, 60, 30), 2);
        let column = link(pos, below).expect("cubes between");
        assert_eq!(column.lo, Pos::new(0, 4, 0), "from just above the work");
        assert_eq!(column.hi, Pos::new(0, 19, 0), "to just below the machine");
        let above = Cubes::around(Pos::new(30, 400, 30), 0);
        assert_eq!(link(pos, above).map(|c| c.lo.y), Some(21), "work above it");
        let beside = Cubes::around(Pos::new(30, 330, 30), 2);
        assert_eq!(link(pos, beside), None, "at its height");
        let next_cube_down = Cubes::around(Pos::new(30, 310, 30), 2);
        assert_eq!(link(pos, next_cube_down), None, "joined already");
    }

    #[test]
    fn a_cube_let_go_and_kept_again_is_found() {
        let layer = |y| Cubes::spanning(Pos::new(0, y, 0), Pos::new(40, y, 40));
        let down = [70, 70, 60, 50, 40].map(layer);
        assert_eq!(kept_again(down), None, "one layer after another");
        let back = [70, 60, 70].map(layer);
        assert_eq!(kept_again(back), Some(Pos::new(0, 64, 0)), "back up");
    }

    #[test]
    fn every_outcome_has_a_name() {
        for &kept in Kept::VARIANTS {
            let key = format!("cogwork.loader.{}", <&str>::from(kept));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }
}
