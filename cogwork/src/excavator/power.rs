//! What an excavator's energy pays for. Holding up the gantry costs a standby draw every tick,
//! by the size of the area; short of that, nothing moves. Energy beyond it pays for the work,
//! so the more engines drive an excavator, the faster it digs.

use weft_sdk::*;

use super::{Dig, status};

/// Standby energy per tick: a base, and one more per four blocks of area (9 by 9: 30).
const STANDBY_BASE: i64 = 10;
const CELLS_PER_STANDBY: i64 = 4;
/// The most energy it puts into its work per tick beyond standby: stone takes at least 8 ticks.
const DRAW_RATE: i64 = 120;
/// Energy per block the drill travels.
const TRAVEL_COST: i64 = 80;

/// Standby energy per tick for the frame from `lo` to `hi`.
pub fn standby(lo: [i32; 3], hi: [i32; 3]) -> i64 {
    let inside = |axis: usize| (hi[axis] - lo[axis] - 1).max(0) as i64;
    STANDBY_BASE + inside(0) * inside(2) / CELLS_PER_STANDBY
}

/// Draws this tick's standby; false when there isn't enough. What there was is used up all the
/// same, so a weak supply can't save up to run in bursts.
pub fn hold(w: &World, id: u64, d: &mut Dig) -> bool {
    let need = standby(d.lo, d.hi);
    let Some(e) = w.energy_mut(id) else {
        return false;
    };
    let drawn = e.stored.min(need);
    e.stored -= drawn;
    if drawn < need {
        d.status = status::NO_POWER;
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
        d.status = status::NO_POWER;
    }
    false
}

/// Pays for up to `max` blocks of drill travel; returns how far the energy took it.
pub fn travel(w: &World, id: u64, max: f32) -> f32 {
    let Some(e) = w.energy_mut(id) else {
        return 0.0;
    };
    let cost = e.stored.min((max * TRAVEL_COST as f32).ceil() as i64);
    e.stored -= cost;
    (cost as f32 / TRAVEL_COST as f32).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standby_grows_with_the_area() {
        let small = standby([0, 0, 0], [10, 4, 10]);
        let large = standby([0, 0, 0], [34, 4, 34]);
        assert_eq!(small, 30, "9 by 9 needs one ember engine");
        assert_eq!(large, 10 + 33 * 33 / 4, "33 by 33 needs many more");
    }
}
