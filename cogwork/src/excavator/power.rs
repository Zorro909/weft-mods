//! What an excavator's energy pays for. Holding up the gantry costs a standby draw every tick,
//! by the size of the area; short of that, nothing moves. Energy beyond it pays for the work,
//! so the more engines drive an excavator, the faster it digs.

use weft_sdk::{ENERGY, Energy, World};

use super::{Dig, Frame, Status};
use crate::machine::{self, Payment};

/// Standby energy per tick: a base, and one more per four blocks of area (9 by 9: 30).
const STANDBY_BASE: i64 = 10;
const CELLS_PER_STANDBY: i64 = 4;
/// The most energy it puts into its work per tick beyond standby: stone takes at least 8 ticks.
const DRAW_RATE: i64 = 120;
/// Energy per block the drill travels.
const TRAVEL_COST: i64 = 80;
/// The store of an excavator with a small area: what it holds, and the most it takes in a tick.
const SMALLEST_STORE: i64 = 20_000;
/// Ticks of standby the store holds on a large area, so it outlasts a gap in the supply.
const STORE_TICKS: i64 = 20;

impl Frame {
    /// Standby energy per tick to hold up this frame's gantry.
    pub fn standby(&self) -> i64 {
        let [x, z] = self.inside();
        STANDBY_BASE + x as i64 * z as i64 / CELLS_PER_STANDBY
    }

    /// The energy an excavator working this frame stores, and takes in at most in a tick.
    pub fn store(&self) -> i64 {
        SMALLEST_STORE.max(STORE_TICKS * self.standby())
    }
}

/// An excavator's store before it has planned its area: in from any number of engines, as
/// standby alone can pass 1000 on a large area.
pub fn smallest_store() -> Energy {
    machine::energy(SMALLEST_STORE, SMALLEST_STORE)
}

/// Sizes the store of the excavator `id` to `frame`. What it held stays, up to the new size.
pub fn fit_store(world: &World, id: u64, frame: &Frame) {
    let Some(energy) = world.component_mut(id, &ENERGY) else {
        return;
    };
    let size = frame.store();
    *energy = Energy {
        stored: energy.stored.min(size),
        ..machine::energy(size, size)
    };
}

/// Draws this tick's standby; false when there isn't enough. What there was is used up all the
/// same, so a weak supply can't save up to run in bursts.
pub fn hold(world: &World, id: u64, dig: &mut Dig) -> bool {
    let need = dig.frame.standby();
    let Some(energy) = world.component_mut(id, &ENERGY) else {
        return false;
    };
    let drawn = energy.stored.min(need);
    energy.stored -= drawn;
    if drawn < need {
        dig.set_status(Status::NoPower);
        return false;
    }
    true
}

/// Puts stored energy into a job of `cost` until `dig.spent` covers it; true once it does. The
/// caller starts `spent` over when the job is done.
pub fn pay(world: &World, id: u64, dig: &mut Dig, cost: i64) -> bool {
    let payment = machine::pay(world, id, &mut dig.spent, cost, DRAW_RATE);
    if payment == Payment::NoPower {
        dig.set_status(Status::NoPower);
    }
    payment == Payment::Paid
}

/// Pays for up to `max` blocks of drill travel; returns how far the energy took it.
pub fn travel(world: &World, id: u64, max: f64) -> f64 {
    let Some(energy) = world.component_mut(id, &ENERGY) else {
        return 0.0;
    };
    let cost = energy.stored.min((max * TRAVEL_COST as f64).ceil() as i64);
    energy.stored -= cost;
    (cost as f64 / TRAVEL_COST as f64).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standby_grows_with_the_area() {
        let frame = |hi| Frame::from(([0, 0, 0], hi));
        let small = frame([10, 4, 10]).standby();
        let large = frame([34, 4, 34]).standby();
        assert_eq!(small, 30, "9 by 9 needs one ember engine");
        assert_eq!(large, 10 + 33 * 33 / 4, "33 by 33 needs many more");
    }

    #[test]
    fn the_store_grows_with_a_large_area() {
        let frame = |hi| Frame::from(([0, 0, 0], hi));
        assert_eq!(frame([10, 4, 10]).store(), 20_000, "9 by 9 as before");
        let largest = frame([256, 4, 256]);
        assert_eq!(
            largest.store(),
            20 * largest.standby(),
            "20 ticks of standby"
        );
        assert!(
            largest.store() > 300_000,
            "{} for 255 by 255",
            largest.store()
        );
    }
}
