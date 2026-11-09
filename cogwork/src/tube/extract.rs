//! Items coming into tubes: the timber tube pulling them out of machines with engine power, and
//! machines of this mod pushing their results into a tube next to them.

use weft_sdk::*;

use super::{Motion, TUBE, Tube, free_slot};
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
pub fn pull(world: &World, id: u64, pos: Pos, tube: &mut Tube) {
    tube.timer = tube.timer.saturating_sub(1);
    if tube.timer > 0 {
        return;
    }
    tube.timer = PULL_EVERY;
    let energy = world
        .component_mut(id, &ENERGY)
        .map_or(0, |store| std::mem::take(&mut store.stored));
    let paid_for = items_for(energy);
    let mut left = paid_for;
    for dir in Dir::ALL {
        let Some(slot) = free_slot(world, id, tube).filter(|_| left > 0) else {
            break;
        };
        let Some(from) = source(world, pos, dir) else {
            continue;
        };
        let stack = crate::store::extract(world, from, left);
        if stack.is_empty() {
            continue;
        }
        left -= stack.count;
        world.inventory(id).set(slot as u32, stack);
        tube.moving[slot] = Motion::entering(stack.item, dir);
    }
    if left < paid_for {
        world.play_sound(&crate::sounds::TUBE_PULL, pos.center(), 1.0);
    }
}

/// The machine (not a tube) the tube at `pos` touches toward `dir`.
fn source(world: &World, pos: Pos, dir: Dir) -> Option<Pos> {
    let other = pos.toward(dir);
    let machine = super::connects_to_tube(world, pos, dir) && !super::is_tube(world, other);
    machine.then_some(other)
}

/// Puts `stack` into the tube at `pos` at its hub, as much as it has room for; returns the rest.
/// For machines outside the tube system: their callbacks see tubes outside a batch.
pub fn inject(world: &World, pos: Pos, stack: ItemStack) -> ItemStack {
    let Some(tube) = world.component(pos, &TUBE) else {
        return stack;
    };
    let Some(slot) = free_slot(world, pos, &tube) else {
        return stack;
    };
    world.inventory(pos).set(slot as u32, stack);
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
