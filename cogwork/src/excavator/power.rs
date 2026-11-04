//! What an excavator's energy pays for. Holding up the gantry costs a standby draw every tick,
//! by the size of the area; short of that, nothing moves. Energy beyond it pays for the work,
//! so the more engines drive an excavator, the faster it digs.

use weft_sdk::*;

use super::{Dig, Frame, Status};

/// Standby energy per tick: a base, and one more per four blocks of area (9 by 9: 30).
const STANDBY_BASE: i64 = 10;
const CELLS_PER_STANDBY: i64 = 4;
/// The most energy it puts into its work per tick beyond standby: stone takes at least 8 ticks.
const DRAW_RATE: i64 = 120;
/// Energy per block the drill travels.
const TRAVEL_COST: i64 = 80;

impl Frame {
    /// Standby energy per tick to hold up this frame's gantry.
    pub fn standby(&self) -> i64 {
        let [x, z] = self.inside();
        STANDBY_BASE + x as i64 * z as i64 / CELLS_PER_STANDBY
    }
}

/// Draws this tick's standby; false when there isn't enough. What there was is used up all the
/// same, so a weak supply can't save up to run in bursts.
pub fn hold(w: &World, id: u64, d: &mut Dig) -> bool {
    let need = d.frame.standby();
    let Some(e) = w.energy_mut(id) else {
        return false;
    };
    let drawn = e.stored.min(need);
    e.stored -= drawn;
    if drawn < need {
        d.set_status(Status::NoPower);
        return false;
    }
    true
}

/// Puts stored energy into a job of `cost` until `d.spent` covers it; true once it does. The
/// caller starts `spent` over when the job is done.
pub fn pay(w: &World, id: u64, d: &mut Dig, cost: i64) -> bool {
    let drawn = crate::machine::draw(w, id, &mut d.spent, cost, DRAW_RATE);
    if d.spent >= cost {
        return true;
    }
    if drawn == 0 {
        d.set_status(Status::NoPower);
    }
    false
}

/// Pays for up to `max` blocks of drill travel; returns how far the energy took it.
pub fn travel(w: &World, id: u64, max: f64) -> f64 {
    let Some(e) = w.energy_mut(id) else {
        return 0.0;
    };
    let cost = e.stored.min((max * TRAVEL_COST as f64).ceil() as i64);
    e.stored -= cost;
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
}
