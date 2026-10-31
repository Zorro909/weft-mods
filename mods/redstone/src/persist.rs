//! Pending device changes kept with the device, so they survive restarts (spec §7.2).
//!
//! Every built-in device that can have a change pending carries a `redstone:device` block entity.
//! Its saved `PENDING` holds the change as it stands: the `Sim` saves what is left every tick it
//! counts, and stops when its cube unloads or its circuit freezes, like the count itself. A
//! `WAKE` timer fires when the change is due (frozen: in a retry's time); one finding the change
//! still saved is set again a retry later, so every saved change has a timer.
//! After a restart memory is empty: the timer, overdue by then, fires once the area is loaded, and
//! the change continues with what was left (at least a tick: an overdue change is never applied
//! on the spot). A change the `Sim` lost in a running game (G6) comes back the same way.

use std::sync::Mutex;

use redstone_core::{GAME_TICKS, Pending, Pos, decode, encode};
use weft_sdk::*;

pub static DEVICE: Entity = Entity::named("device");

value! {
    /// `next`: the state it changes to (`redstone_core::encode`). `remaining` 0: none. `from_tick`
    /// and `frozen` say when it was saved and whether it was counting then.
    pub struct PendingV { pub next: u32, pub remaining: u32, pub from_tick: u64, pub frozen: u8 }
}
pub static PENDING: Component<PendingV> = Component::named("pending");

value! {
    pub struct Wake { pub pos: [i32; 3] }
}
pub static WAKE: Event<Wake> = Event::named("wake");

const NONE: PendingV = PendingV { next: 0, remaining: 0, from_tick: 0, frozen: 0 };

/// Game ticks until a frozen change's timer: the core's retry.
const RETRY: u32 = redstone_core::RETRY * GAME_TICKS;

/// Changes restored from saves with their device's block entity, for the driver's next tick.
static RESTORED: Mutex<Vec<(Pos, Pending, u64)>> = Mutex::new(Vec::new());

pub fn register(r: &mut Registry, devices: &[&Block]) {
    r.entity_component(&PENDING, NONE, component_flags::SAVE);
    r.entity(&DEVICE).no_egg().flags(entity_flags::NO_GRAVITY | entity_flags::INVULNERABLE).attach(&PENDING);
    for b in devices {
        r.block_entity(b, &DEVICE, 0);
    }
    r.on(&WAKE, |b| {
        let w = b.world();
        let found: Vec<_> = b.iter().filter_map(|e| restore(&w, e.pos).map(|(p, be)| (e.pos, p, be))).collect();
        // Still saved, so still pending: the next timer, until the change is done.
        for (pos, _, be) in &found {
            w.after(*be, RETRY, &WAKE, Wake { pos: *pos });
        }
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
            // A new change gets its timer; later saves of it keep that one.
            let new = v.remaining == 0;
            *v = PendingV { next: encode(pd.next), remaining: pd.remaining, from_tick: w.time(), frozen: frozen as u8 };
            if new {
                w.after(be, if frozen { RETRY } else { pd.remaining * GAME_TICKS }, &WAKE, Wake { pos: p });
            }
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
    // The driver schedules it next tick, from which `remaining` counts (`Sim::schedule`). The
    // save is at most a tick old wherever the change was counting.
    Some((Pending { next, remaining: v.remaining.max(1) }, be))
}

/// What timers found since the last call; the driver schedules those the `Sim` doesn't know.
pub fn take_restored() -> Vec<(Pos, Pending, u64)> {
    std::mem::take(&mut *RESTORED.lock().unwrap_or_else(|e| e.into_inner()))
}
