//! Runs the tick: collects what changed, and once per region and tick hands it to `Sim`.
//!
//! Weft gives mods no region or dimension identity (G6), so one `Sim` serves every region of
//! every dimension. A queued position is claimed by the first region that has redstone at it or
//! next to it; one nobody claims within 5 s is dropped. `Sim` only counts a pending change down
//! in a world where its device is.
//!
//! Locking: a region's `world_tick` holds the `Sim` for its whole call (regions take turns).
//! `on_neighbor` and event handlers only touch the small `DIRTY` and `OWN` maps, so they never
//! wait for a running solve.

use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use redstone_api::{CHANGED, STRENGTH};
use redstone_core::{Cell, Pos, Sim, View};
use weft_sdk::*;

use crate::adapter::{self, WeftWorld};
use crate::persist;

static SIM: LazyLock<Mutex<Sim>> = LazyLock::new(|| Mutex::new(Sim::default()));
/// Positions something changed at, with when they were queued.
static DIRTY: Mutex<BTreeMap<Pos, Instant>> = Mutex::new(BTreeMap::new());
/// What the tick wrote where, and in which world tick, so `on_neighbor` can ignore it.
static OWN: Mutex<BTreeMap<Pos, (u32, u64)>> = Mutex::new(BTreeMap::new());

const UNCLAIMED: Duration = Duration::from_secs(5);

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // A panicking callback is switched off by the SDK; the data it left is still fine to use.
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Something changed at `p`.
pub fn queue(p: Pos) {
    lock(&DIRTY).insert(p, Instant::now());
}

/// `on_neighbor` of every redstone block: the neighbor changed, unless that was our own write.
pub fn neighbor(e: &mut BlockEvent) {
    let (w, pos, other) = (e.world(), e.pos(), e.other());
    let state = w.get(other);
    if lock(&OWN).get(&other).is_some_and(|(s, _)| *s == state) {
        return;
    }
    queue(other);
    queue(pos);
}

pub fn register(r: &mut Registry) {
    r.on(&CHANGED, |b| b.iter().for_each(|c| queue(c.pos)));
    r.on(&BLOCK_PLACED, |b| b.iter().for_each(|c| queue(c.pos)));
    r.on(&BLOCK_BROKEN, |b| b.iter().for_each(|c| queue(c.pos)));
    r.modify(&STRENGTH, stage::ADD, |m| {
        let p = m.query().pos;
        m.value += adapter::strength_at(&m.world(), p) as f64;
    });
    r.world_tick(tick);
}

fn tick(w: &World) {
    let mut sim = lock(&SIM);
    let now = w.time();
    lock(&OWN).retain(|_, (_, at)| *at + 2 >= now);
    let mut v = WeftWorld::new(w);

    // Changes restored from saves after a restart (`persist`); the region that has the device
    // claims the position below.
    for (p, pending) in persist::take_restored() {
        if !sim.has_pending(p) {
            sim.schedule(p, pending);
        }
        queue(p);
    }

    // Claim what this world has redstone at or next to; leave the rest for other regions.
    let pending = std::mem::take(&mut *lock(&DIRTY));
    let mut left = BTreeMap::new();
    let clock = Instant::now();
    for (p, at) in pending {
        if adapter::redstone_near(&v, p) {
            if !matches!(v.cell(p), Cell::Device(_)) {
                sim.forget(p);
            }
            sim.touch(p);
        } else if clock.duration_since(at) < UNCLAIMED {
            left.insert(p, at);
        }
    }
    if !left.is_empty() {
        let mut d = lock(&DIRTY);
        for (p, at) in left {
            d.entry(p).or_insert(at);
        }
    }

    sim.tick(&mut v);
    if !v.own.is_empty() {
        let mut own = lock(&OWN);
        for (p, s) in v.own.drain(..) {
            own.insert(p, (s, now));
        }
    }
}
