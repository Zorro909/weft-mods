//! Items coming into tubes: the timber tube pulling them out of machines with engine power, and
//! machines of this mod pushing their results into a tube next to them.

use weft_sdk::*;

use super::{BASE_SPEED, Motion, NONE, TUBE, Tube, free_slot};
use crate::lib::Pos;

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
pub fn pull(w: &World, id: u64, pos: Pos, tube: &mut Tube) {
    tube.timer = tube.timer.saturating_sub(1);
    if tube.timer > 0 {
        return;
    }
    tube.timer = PULL_EVERY;
    let energy = w
        .component_mut(id, &ENERGY)
        .map_or(0, |e| std::mem::take(&mut e.stored));
    let paid_for = items_for(energy);
    let mut left = paid_for;
    for d in Dir::ALL {
        let Some(slot) = free_slot(w, id, tube).filter(|_| left > 0) else {
            break;
        };
        let Some(from) = source(w, pos, d) else {
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
            from: d.face(),
            to: NONE,
        };
    }
    if left < paid_for {
        w.play_sound(&crate::sounds::TUBE_PULL, pos.center(), 1.0);
    }
}

/// The machine (not a tube) the tube at `pos` touches toward `d`.
fn source(w: &World, pos: Pos, d: Dir) -> Option<Pos> {
    let other = pos.toward(d);
    let machine = super::connects_to_tube(w, pos, d) && !super::is_tube(w, other);
    machine.then_some(other)
}

/// Puts `stack` into the tube at `pos` at its hub, as much as it has room for; returns the rest.
/// For machines outside the tube system: their callbacks see tubes outside a batch.
pub fn inject(w: &World, pos: Pos, stack: ItemStack) -> ItemStack {
    let Some(tube) = w.component(pos, &TUBE) else {
        return stack;
    };
    let Some(slot) = free_slot(w, pos, &tube) else {
        return stack;
    };
    w.inventory(pos).set(slot as u32, stack);
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
