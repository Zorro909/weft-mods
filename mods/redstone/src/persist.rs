//! Pending device changes kept with the device, so they survive restarts (spec §7.2).
//!
//! Every built-in device that can have a change pending carries a `redstone:device` block entity.
//! Its saved `PENDING` holds the change, and a `WAKE` timer fires when it is due (frozen: in a
//! retry's time). In a running game the `Sim` has every change in memory, unloaded areas
//! included, and a timer finds nothing to do. After a restart memory is empty: a device's timer,
//! overdue by then, fires once its area is loaded, and the change is restored with the time that
//! passed taken off (none for a frozen one) and at least a tick left. An overdue change is never
//! applied on the spot.

use std::sync::Mutex;

use redstone_core::{Pending, Pos, decode, encode};
use weft_sdk::*;

pub static DEVICE: Entity = Entity::named("device");

value! {
    /// `next`: the state it changes to (`redstone_core::encode`). `remaining` 0: none.
    pub struct PendingV { pub next: u32, pub remaining: u32, pub from_tick: u64, pub frozen: u8 }
}
pub static PENDING: Component<PendingV> = Component::named("pending");

value! {
    pub struct Wake { pub pos: [i32; 3] }
}
pub static WAKE: Event<Wake> = Event::named("wake");

const NONE: PendingV = PendingV { next: 0, remaining: 0, from_tick: 0, frozen: 0 };

/// Ticks until a frozen change's timer: the core's retry.
const RETRY: u32 = 20;

/// Changes restored from saves with their device's block entity, for the driver's next tick.
static RESTORED: Mutex<Vec<(Pos, Pending, u64)>> = Mutex::new(Vec::new());

pub fn register(r: &mut Registry, devices: &[&Block]) {
    r.entity_component(&PENDING, NONE, component_flags::SAVE);
    r.entity(&DEVICE).flags(entity_flags::NO_GRAVITY | entity_flags::INVULNERABLE).attach(&PENDING);
    for b in devices {
        r.block_entity(b, &DEVICE, 0);
    }
    r.on(&WAKE, |b| {
        let w = b.world();
        let found: Vec<_> = b.iter().filter_map(|e| restore(&w, e.pos).map(|(p, be)| (e.pos, p, be))).collect();
        if !found.is_empty() {
            RESTORED.lock().unwrap_or_else(|e| e.into_inner()).extend(found);
        }
    });
}

/// `redstone_core::World::save_pending`.
pub fn save(w: &World, p: Pos, pending: Option<(Pending, bool)>) {
    let be = w.block_entity(p);
    if be == 0 {
        return;
    }
    let Some(v) = w.component_mut(be, &PENDING) else { return };
    match pending {
        None => *v = NONE,
        Some((pd, frozen)) => {
            *v = PendingV { next: encode(pd.next), remaining: pd.remaining, from_tick: w.time(), frozen: frozen as u8 };
            w.after(be, if frozen { RETRY } else { pd.remaining }, &WAKE, Wake { pos: p });
        }
    }
}

/// The change saved at `p`, as it stands now, and the block entity it is saved on.
fn restore(w: &World, p: Pos) -> Option<(Pending, u64)> {
    let be = w.block_entity(p);
    if be == 0 {
        return None;
    }
    let v = *w.component(be, &PENDING)?;
    if v.remaining == 0 {
        return None;
    }
    let next = decode(v.next)?;
    // The driver schedules it next tick, from which `remaining` counts (`Sim::schedule`).
    let passed = if v.frozen != 0 { 0 } else { (w.time() + 1).saturating_sub(v.from_tick) };
    Some((Pending { next, remaining: (v.remaining as u64).saturating_sub(passed).max(1) as u32 }, be))
}

/// What timers found since the last call; the driver schedules those the `Sim` doesn't know.
pub fn take_restored() -> Vec<(Pos, Pending, u64)> {
    std::mem::take(&mut *RESTORED.lock().unwrap_or_else(|e| e.into_inner()))
}
