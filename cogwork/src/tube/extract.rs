//! Items coming into tubes: the timber tube pulling them out of machines with engine power, and
//! machines of this mod pushing their results into a tube next to them.

use weft_sdk::*;

use super::{BASE_SPEED, Motion, NONE, TUBE, Tube, free_slot};
use crate::neighbor;

/// Ticks between two pulls of a timber tube: an engine's stroke.
const PULL_EVERY: u32 = 18;
/// Energy a pull costs, and as much again per item: a spring engine's stroke pulls one item, an
/// ember engine's eight.
const PULL_COST: i64 = 100;
const ITEM_COST: i64 = 75;
/// The most items one pull takes, however many engines drive the tube.
const PULL_MAX: u32 = 16;

/// How many items a pull with `energy` takes.
fn items_for(energy: i64) -> u32 {
    ((energy - PULL_COST) / ITEM_COST).clamp(0, PULL_MAX as i64) as u32
}

/// A timber tube's pull, once a stroke: it takes as many items out of the machines it touches as
/// the energy of that stroke pays for. The energy is used up whether or not there was anything
/// to take, so a tube left idle doesn't save up for a burst.
pub fn pull(w: &World, id: u64, pos: [i32; 3], tube: &mut Tube) {
    tube.timer = tube.timer.saturating_sub(1);
    if tube.timer > 0 {
        return;
    }
    tube.timer = PULL_EVERY;
    let energy = w
        .energy_mut(id)
        .map_or(0, |e| std::mem::take(&mut e.stored));
    let mut left = items_for(energy);
    for face in 0..6 {
        let Some(slot) = free_slot(w, id, tube).filter(|_| left > 0) else {
            return;
        };
        let Some(from) = source(w, pos, face) else {
            continue;
        };
        let stack = crate::store::extract(w, from, left);
        if stack.is_empty() {
            continue;
        }
        left -= stack.count;
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
    let other = w.block_entity(neighbor(pos, face));
    (other != 0 && w.component(other, &TUBE).is_none()).then_some(other)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stroke_pulls_by_the_engine() {
        assert_eq!(
            items_for(10 * PULL_EVERY as i64),
            1,
            "a spring engine's stroke: one item"
        );
        assert_eq!(
            items_for(40 * PULL_EVERY as i64),
            8,
            "an ember engine's: eight"
        );
        assert_eq!(items_for(50), 0, "too little for a pull");
        assert_eq!(items_for(100_000), PULL_MAX, "never more than a pull holds");
    }
}
