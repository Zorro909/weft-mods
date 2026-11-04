//! Mining one block: the drill travels to it, energy is paid by how hard it is, and what it drops
//! goes into the excavator's output.

use weft_sdk::*;

use super::power::{pay, travel};
use super::{Dig, EXCAVATOR, Frame, GANTRY_FRAME, Status};
use crate::block_state::BlockState;
use crate::float_pos::FloatPos;
use crate::harvest::{Harvest, cost, harvest};
use crate::pos::Pos;

/// Energy per block: a base and as much again per point of hardness (stone 1.5: 1000).
const BASE_COST: f32 = 400.0;
const COST_PER_HARDNESS: f32 = 400.0;
/// Blocks per tick the drill moves.
const DRILL_SPEED: f64 = 0.25;

pub enum Mined {
    Done,
    /// Nothing to mine there after all, or someone wouldn't let it.
    Skipped,
    /// Not yet: the status says why.
    Waiting(Status),
}

/// Whether the drill stops for a block: not air or liquid, breakable, and not its own frame.
pub fn worth_mining(block_state: BlockState) -> bool {
    block_state.can_break() && !block_state.is(&GANTRY_FRAME) && !block_state.is(&EXCAVATOR)
}

/// Energy to mine `block_state`.
pub fn block_cost(block_state: BlockState) -> i64 {
    cost(block_state, BASE_COST, COST_PER_HARDNESS)
}

/// Where the drill is while it mines the block `c` inside `frame`, `done` of the way through (0
/// to 1): it crosses the block along its row, from the edge it comes in by to the far one, which
/// is where the next block of the row starts.
pub fn sweep(frame: &Frame, c: Pos, done: f64) -> FloatPos {
    // Half a block before the middle at 0, half a block past it at 1.
    let along_row = frame.row_dir(c) as f64 * (done - 0.5);
    c.floor_middle().add_x(along_row)
}

/// Where the drill is in the block `c` with `spent` of its `cost` paid: past it once paid.
pub fn progress_point(frame: &Frame, c: Pos, spent: i64, cost: i64) -> FloatPos {
    sweep(frame, c, (spent as f64 / cost.max(1) as f64).min(1.0))
}

/// Moves the drill toward `goal` as far as this tick's energy pays for; true once it is there.
pub fn drill_to(w: &World, id: u64, d: &mut Dig, goal: FloatPos) -> bool {
    let length = d.head().distance_to(goal);
    let step = travel(w, id, length.min(DRILL_SPEED));
    if step >= length {
        d.set_head(goal);
        return true;
    }
    d.set_status(if step > 0.0 {
        Status::Mining
    } else {
        Status::NoPower
    });
    d.set_head(d.head().moved_toward(goal, step));
    false
}

/// Pays for and mines the block `state` at `c` into the excavator `id`'s output.
pub fn mine_block(w: &World, id: u64, d: &mut Dig, c: Pos, block_state: BlockState) -> Mined {
    if !worth_mining(block_state) {
        return Mined::Skipped;
    }
    d.set_status(Status::Mining);
    if !pay(w, id, d, block_cost(block_state)) {
        return Mined::Waiting(d.status());
    }
    match harvest(w, id, c, block_state) {
        // Paid for already: it stays paid until there is room.
        Harvest::Full => Mined::Waiting(Status::Full),
        Harvest::Refused => {
            d.spent = 0;
            Mined::Skipped
        }
        Harvest::Done => {
            d.spent = 0;
            Mined::Done
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Frame = Frame {
        lo: Pos::new(0, 0, 0),
        hi: Pos::new(10, 8, 10),
    };

    #[test]
    fn the_drill_crosses_the_block_along_its_row() {
        assert_eq!(
            sweep(&FRAME, Pos::new(3, 5, 1), 0.0),
            FloatPos::new(3.0, 5.0, 1.5),
            "in at the near edge"
        );
        assert_eq!(
            sweep(&FRAME, Pos::new(3, 5, 1), 0.5),
            Pos::new(3, 5, 1).floor_middle(),
            "the middle half way"
        );
        assert_eq!(
            sweep(&FRAME, Pos::new(3, 5, 1), 1.0),
            sweep(&FRAME, Pos::new(4, 5, 1), 0.0),
            "out where the next starts"
        );
        assert_eq!(
            sweep(&FRAME, Pos::new(3, 5, 2), 0.25),
            FloatPos::new(3.75, 5.0, 2.5),
            "the other way on the next row"
        );
    }

    #[test]
    fn a_block_under_way_is_carried_on_from_where_the_drill_got_to() {
        let c = Pos::new(3, 5, 1);
        assert_eq!(
            progress_point(&FRAME, c, 500, 1000),
            FloatPos::new(3.5, 5.0, 1.5),
            "half paid, half way"
        );
        assert_eq!(
            progress_point(&FRAME, c, 1500, 1000),
            sweep(&FRAME, c, 1.0),
            "never past the far edge"
        );
    }
}
