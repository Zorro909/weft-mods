//! Mining one block: the drill travels to it, energy is paid by how hard it is, and what it drops
//! goes into the excavator's output.

use weft_sdk::*;

use super::power::{pay, travel};
use super::{Dig, EXCAVATOR, GANTRY_FRAME, Status};
use crate::harvest::{Harvest, breakable, cost, harvest};

/// Energy per block: a base and as much again per point of hardness (stone 1.5: 1000).
const BASE_COST: f32 = 400.0;
const COST_PER_HARDNESS: f32 = 400.0;
/// Blocks per tick the drill moves.
const DRILL_SPEED: f32 = 0.25;

pub enum Mined {
    Done,
    /// Nothing to mine there after all, or someone wouldn't let it.
    Skipped,
    /// Not yet: the status says why.
    Waiting(Status),
}

/// Whether the drill stops for a block: not air or liquid, breakable, and not its own frame.
pub fn worth_mining(state: u32) -> bool {
    breakable(state) && !GANTRY_FRAME.is(state) && !EXCAVATOR.is(state)
}

/// Energy to mine the block `state`.
pub fn block_cost(state: u32) -> i64 {
    cost(state, BASE_COST, COST_PER_HARDNESS)
}

/// Where the drill is while it mines the block `c`, `done` of the way through (0 to 1): it
/// crosses the block along its row, from the edge it comes in by to the far one, which is where
/// the next block of the row starts.
pub fn sweep(lo: [i32; 3], c: [i32; 3], done: f32) -> [f32; 3] {
    let dir = super::area::row_dir(lo, c) as f32;
    [
        c[0] as f32 + 0.5 + dir * (done - 0.5),
        c[1] as f32,
        c[2] as f32 + 0.5,
    ]
}

/// Where the drill is in the block `c` with `spent` of its `cost` paid: past it once paid.
pub fn progress_point(lo: [i32; 3], c: [i32; 3], spent: i64, cost: i64) -> [f32; 3] {
    sweep(lo, c, (spent as f32 / cost.max(1) as f32).min(1.0))
}

/// Moves the drill toward `goal` as far as this tick's energy pays for; true once it is there.
pub fn drill_to(w: &World, id: u64, d: &mut Dig, goal: [f32; 3]) -> bool {
    let gap: [f32; 3] = std::array::from_fn(|i| goal[i] - d.head[i]);
    let length = gap.iter().map(|v| v * v).sum::<f32>().sqrt();
    let step = travel(w, id, length.min(DRILL_SPEED));
    if step >= length {
        d.head = goal;
        return true;
    }
    let status = if step > 0.0 {
        Status::Mining
    } else {
        Status::NoPower
    };
    d.status = status as u32;
    for (head, gap) in d.head.iter_mut().zip(gap) {
        *head += gap / length * step;
    }
    false
}

/// Pays for and mines the block `state` at `c` into the excavator `id`'s output.
pub fn mine_block(w: &World, id: u64, d: &mut Dig, c: [i32; 3], state: u32) -> Mined {
    if !worth_mining(state) {
        return Mined::Skipped;
    }
    d.status = Status::Mining as u32;
    if !pay(w, id, d, block_cost(state)) {
        return Mined::Waiting(Status::from_repr(d.status).unwrap_or_default());
    }
    match harvest(w, id, c, state) {
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

    #[test]
    fn the_drill_crosses_the_block_along_its_row() {
        let lo = [0, 0, 0];
        assert_eq!(
            sweep(lo, [3, 5, 1], 0.0),
            [3.0, 5.0, 1.5],
            "in at the near edge"
        );
        assert_eq!(
            sweep(lo, [3, 5, 1], 1.0),
            sweep(lo, [4, 5, 1], 0.0),
            "out where the next starts"
        );
        assert_eq!(
            sweep(lo, [3, 5, 2], 0.25),
            [3.75, 5.0, 2.5],
            "the other way on the next row"
        );
    }

    #[test]
    fn a_block_under_way_is_carried_on_from_where_the_drill_got_to() {
        let (lo, c) = ([0, 0, 0], [3, 5, 1]);
        assert_eq!(
            progress_point(lo, c, 500, 1000),
            [3.5, 5.0, 1.5],
            "half paid, half way"
        );
        assert_eq!(
            progress_point(lo, c, 1500, 1000),
            sweep(lo, c, 1.0),
            "never past the far edge"
        );
    }
}
