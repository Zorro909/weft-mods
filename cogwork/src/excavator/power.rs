//! What an excavator's energy pays for. Holding up the gantry costs a standby draw every tick,
//! by the length of its sides; short of that, nothing moves. Energy beyond it pays for the work,
//! so the more engines drive an excavator, the faster it digs, up to a block in the same time
//! at any depth.

use weft_sdk::{ENERGY, Energy, World};

use super::depth::at_depth;
use super::{Dig, Frame, Status};
use crate::lib::Pos;
use crate::machine::{self, Payment};

/// Standby energy per tick: a base, and one more per block across the inside, along x and along
/// z (9 by 9: 30). By the sides, not the area: the drill mines no faster on a large area, so
/// standby by area would cost each block mined as much again per block of area.
const STANDBY_BASE: i64 = 12;
const STANDBY_PER_BLOCK_ACROSS: i64 = 1;
/// The most energy it puts into its work per tick beyond standby, near the top: stone takes at
/// least 8 ticks. Deeper down it grows as a block's price does (`draw_rate`).
const DRAW_RATE: i64 = 120;
/// Energy per block the drill travels.
const TRAVEL_COST: i64 = 80;
/// The store of an excavator with a small area: what it holds, and the most it takes in a tick.
const SMALLEST_STORE: i64 = 20_000;
/// Ticks of standby and work the store holds, so it outlasts a gap in the supply.
const STORE_TICKS: i64 = 20;

impl Frame {
    /// Standby energy per tick to hold up this frame's gantry.
    pub fn standby(&self) -> i64 {
        let [x, z] = self.inside();
        STANDBY_BASE + STANDBY_PER_BLOCK_ACROSS * (x as i64 + z as i64)
    }

    /// The energy an excavator working this frame `below` blocks under it stores, and takes in
    /// at most in a tick.
    pub fn store(&self, below: i64) -> i64 {
        SMALLEST_STORE.max(STORE_TICKS * (self.standby() + draw_rate(below)))
    }
}

/// The most energy put into a block `below` blocks under the excavator per tick: as much more
/// as the block costs more, so depth costs energy, not time.
fn draw_rate(below: i64) -> i64 {
    at_depth(DRAW_RATE, below)
}

/// An excavator's store before it has planned its area: in from any number of engines.
pub fn smallest_store() -> Energy {
    machine::energy(SMALLEST_STORE, SMALLEST_STORE)
}

/// Sizes the store of the excavator `id` to `frame`, working `below` blocks under it. What it
/// held stays, up to the new size.
pub fn fit_store(world: &World, id: u64, frame: &Frame, below: i64) {
    let size = frame.store(below);
    let Some(energy) = world.component_mut(id, &ENERGY) else {
        return;
    };
    if energy.capacity == size {
        return;
    }
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

/// Puts stored energy into work on `cell` (mining it, or putting up a frame block, which is never
/// below the excavator) for `cost` until `dig.spent` covers it; true once it does. The caller
/// starts `spent` over when the job is done.
pub fn pay(world: &World, id: u64, dig: &mut Dig, cell: Pos, cost: i64) -> bool {
    let rate = draw_rate(dig.frame.below(cell));
    let payment = machine::pay(world, id, &mut dig.spent, cost, rate);
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
        assert_eq!(large, 78, "33 by 33 needs two");
        let largest = frame([256, 4, 256]).standby();
        assert_eq!(largest, 522, "255 by 255 by its sides, not its area");
    }

    #[test]
    fn the_store_grows_with_a_large_area_and_with_depth() {
        let frame = |hi| Frame::from(([0, 0, 0], hi));
        let small = frame([10, 4, 10]);
        assert_eq!(small.store(0), 20_000, "9 by 9 near the top as before");
        assert_eq!(
            small.store(1024),
            20 * (30 + 64 * 120),
            "20 ticks of its draw deep down"
        );
        let largest = frame([256, 4, 256]);
        assert_eq!(
            largest.store(384),
            20 * (522 + 480),
            "20 ticks of standby and work"
        );
    }

    #[test]
    fn a_block_takes_as_long_at_any_depth() {
        let ticks = |below| {
            let cost = at_depth(1000, below);
            (cost + draw_rate(below) - 1) / draw_rate(below)
        };
        assert_eq!(ticks(0), 9, "stone near the top");
        assert_eq!(ticks(384), 9, "at 4 times the price");
        assert_eq!(ticks(1024), 9, "at 64 times the price");
    }
}
