//! Items coming into tubes: the timber tube pulling them out of machines with engine power, and
//! machines of this mod pushing their results into a tube next to them.

use weft_sdk::*;

use super::{BASE_SPEED, Motion, NONE, SLOTS, TUBE, Tube, step};

/// Ticks between two pulls of a timber tube.
const PULL_EVERY: u32 = 10;
/// Energy a pull costs, plus as much again per item.
const PULL_COST: i64 = 20;
const ITEM_COST: i64 = 10;
/// The most items one pull takes.
const PULL_MAX: u32 = 16;

/// A timber tube's pull: with energy to spend, it takes items out of the machines it touches,
/// as many as the energy pays for.
pub fn pull(w: &World, id: u64, pos: [i32; 3], tube: &mut Tube) {
    tube.timer = tube.timer.saturating_sub(1);
    if tube.timer > 0 {
        return;
    }
    tube.timer = PULL_EVERY;
    for face in 0..6 {
        let stored = w.energy(id).map_or(0, |e| e.stored);
        let max = ((stored - PULL_COST) / ITEM_COST).clamp(0, PULL_MAX as i64) as u32;
        let Some(slot) = free_slot(w, id, tube) else {
            return;
        };
        if max == 0 {
            return;
        }
        let Some(from) = source(w, pos, face) else {
            continue;
        };
        let stack = crate::store::extract(w, from, max);
        if stack.is_empty() {
            continue;
        }
        if let Some(e) = w.energy_mut(id) {
            e.stored -= PULL_COST + ITEM_COST * stack.count as i64;
        }
        w.inventory(id).set(slot as u32, stack);
        tube.moving[slot] = Motion {
            item: stack.item,
            visual: 0,
            pos: 0.0,
            speed: BASE_SPEED,
            from: face,
            to: NONE,
        };
    }
}

/// The machine (not a tube) the tube at `pos` touches through `face`.
fn source(w: &World, pos: [i32; 3], face: u32) -> Option<u64> {
    if !super::connects_to_tube(w, pos, face) {
        return None;
    }
    let other = w.block_entity(step(pos, face));
    (other != 0 && w.component(other, &TUBE).is_none()).then_some(other)
}

fn free_slot(w: &World, id: u64, tube: &Tube) -> Option<usize> {
    let inv = w.inventory(id);
    (0..SLOTS).find(|&k| inv.get(k as u32).is_empty() && tube.moving[k].item == 0)
}

/// Puts `stack` into the tube `id` at its hub, as much as it has room for; returns the rest.
/// For machines outside the tube system: their callbacks see tubes outside a batch.
pub fn inject(w: &World, id: u64, stack: ItemStack) -> ItemStack {
    let Some(tube) = w.component(id, &TUBE).copied() else {
        return stack;
    };
    let Some(slot) = free_slot(w, id, &tube) else {
        return stack;
    };
    w.inventory(id).set(slot as u32, stack);
    ItemStack::EMPTY
}
