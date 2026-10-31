//! Pressure plates: on while anything stands in the plate's cell, off `PLATE_OFF` redstone ticks
//! after the last one left. Each plate's block entity remembers when it last saw someone (saved,
//! so a restart doesn't cut that short) and a system looks every game tick.

use redstone_core::{GAME_TICKS, Kind, PLATE_OFF};
use weft_sdk::*;

use crate::{blocks, driver};

pub static PLATE_DEVICE: Entity = Entity::named("plate");

value! {
    pub struct Plate { pub last_seen: u64 }
}
pub static PLATE: Component<Plate> = Component::named("plate");

pub fn register(r: &mut Registry) {
    r.entity_component(&PLATE, Plate { last_seen: 0 }, component_flags::SAVE);
    r.entity(&PLATE_DEVICE).no_egg().flags(entity_flags::NO_GRAVITY | entity_flags::INVULNERABLE).attach(&PLATE);
    r.block_entity(&blocks::PLATE, &PLATE_DEVICE, 0);
    r.system(&PLATE, |b| {
        let w = b.world();
        let now = w.time();
        for (id, plate, _) in b.iter_mut() {
            let Some(e) = w.entity(id) else { continue };
            let p = e.pos.map(|c| c.floor() as i32);
            let Some(Kind::Plate { on }) = blocks::kind_of(w.get(p)) else { continue };
            let seen = pressed(&w, id, p);
            if seen {
                plate.last_seen = now;
            }
            let want = seen || (on && now.saturating_sub(plate.last_seen) < (PLATE_OFF * GAME_TICKS) as u64);
            if want != on {
                if let Some(s) = blocks::state_of(Kind::Plate { on: want }) {
                    w.set(p, s);
                    driver::queue(p);
                }
            }
        }
    });
}

/// Whether an entity other than block entities has its feet in the cell at `p`.
fn pressed(w: &World, me: u64, p: [i32; 3]) -> bool {
    let center = [p[0] as f64 + 0.5, p[1] as f64 + 0.5, p[2] as f64 + 0.5];
    w.near(center, 1.0, None).into_iter().any(|id| {
        if id == me {
            return false;
        }
        let Some(e) = w.entity(id) else { return false };
        let at = e.pos.map(|c| c.floor() as i32);
        at == p && w.block_entity(at) != id
    })
}
