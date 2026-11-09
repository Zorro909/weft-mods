//! A full tube that passes nothing on drops what is stuck in it, so tubes never jam for good:
//! two full tubes each waiting for the other, or items with nowhere to go.

use weft_sdk::*;

use super::{Motion, SLOTS, Tube, clear_slot, free_slot};
use crate::lib::Pos;

/// Ticks a full tube may go without passing an item on before it drops one, and then between
/// one drop and the next while it stays jammed.
const PATIENCE: u32 = 60;
const AGAIN: u32 = 10;

/// Counts how long the tube `id` at `pos` has been full without passing anything on, and drops
/// a stuck item once that is too long.
pub fn check(w: &World, id: u64, pos: Pos, tube: &mut Tube, passed_on: bool) {
    let full = free_slot(w, id, tube).is_none();
    if !full || passed_on {
        tube.stuck = 0;
        return;
    }
    tube.stuck += 1;
    if tube.stuck < PATIENCE {
        return;
    }
    let Some(slot) = (0..SLOTS).find(|&k| waiting(&tube.moving[k])) else {
        return;
    };
    w.spawn_item(pos.center(), w.inventory(id).get(slot as u32));
    w.play_sound(&crate::sounds::TUBE_JAM, pos.center(), 1.0);
    clear_slot(w, id, tube, slot);
    tube.stuck = PATIENCE - AGAIN;
}

/// Whether an item stands still: at the hub with nowhere to go, or at a face it can't pass.
fn waiting(m: &Motion) -> bool {
    let nowhere_to_go = m.leaves_by().is_none() && m.reached_hub();
    !m.is_none() && (m.at_far_face() || nowhere_to_go)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tube::motion::NONE;

    #[test]
    fn only_items_standing_still_are_stuck() {
        let at = |pos: f32, to: u32| Motion {
            item: 1,
            pos,
            to,
            ..Motion::default()
        };
        assert!(waiting(&at(0.5, NONE)), "at the hub, nowhere to go");
        assert!(waiting(&at(1.0, 2)), "at a face it can't pass");
        assert!(!waiting(&at(0.7, 2)), "on its way out");
        assert!(!waiting(&at(0.2, NONE)), "on its way in");
    }
}
