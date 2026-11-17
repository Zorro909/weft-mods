//! Mining one block: the drill travels to it, energy is paid by how hard it is, a fifth less with
//! lubricant (`lube`), and what it drops goes into the quarry's output, stone and ore as rubble
//! (`rubble::quarried`), as much Ore Rubble as its kind brings up. A block mined wears the bit
//! and uses its lubricant.

use weft_sdk::{Vec3, World};

use super::depth::at_depth;
use super::dig::Quarry;
use super::kind::{Drill, Kind};
use super::power::{pay, travel};
use super::{Dig, Frame, Status, bit, lube};
use crate::harvest::{Harvest, Keep, cost, harvest};
use crate::lib::{BlockState, Pos, point};

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

/// Whether the drill stops for a block: not air or liquid, breakable, and no quarry or frame.
pub fn worth_mining(block_state: BlockState) -> bool {
    block_state.can_break() && !Kind::is_part(block_state)
}

/// Energy to mine `block_state` at `cell` inside `frame`: dearer the deeper it lies below the
/// excavator, which stands at the bottom of its frame, and a fifth less `lubed`.
pub fn block_cost(frame: &Frame, cell: Pos, block_state: BlockState, lubed: bool) -> i64 {
    let plain = cost(block_state, BASE_COST, COST_PER_HARDNESS);
    let deep = at_depth(plain, frame.below(cell));
    if lubed { lube::eased(deep) } else { deep }
}

/// What a block mined by `quarry` with `drill` costs it besides energy: wear on the drill's bit,
/// half as often with lubricant, and the lubricant. A bit in the slot the drill doesn't dig with
/// (one the Gantry doesn't take, or one held back for a swap) wears nothing.
pub fn wear_down(world: &World, quarry: Quarry, dig: &mut Dig, drill: Drill) {
    if lube::wears(quarry.lubed, &mut dig.lube_skipped) && drill.bit.is_some() {
        bit::wear(world, quarry.id);
    }
    if quarry.lubed {
        lube::use_up(world, quarry.id);
    }
}

/// Where the drill is while it mines the block `cell` inside `frame`, `done` of the way through (0
/// to 1): it crosses the block along its row, from the edge it comes in by to the far one, which
/// is where the next block of the row starts.
pub fn sweep(frame: &Frame, cell: Pos, done: f64) -> Vec3 {
    // Half a block before the middle at 0, half a block past it at 1.
    let along_row = frame.row_dir(cell) as f64 * (done - 0.5);
    cell.floor_middle().offset(along_row, 0.0, 0.0)
}

/// Where the drill is in the block `cell` with `spent` of its `cost` paid: past it once paid.
pub fn progress_point(frame: &Frame, cell: Pos, spent: i64, cost: i64) -> Vec3 {
    sweep(frame, cell, (spent as f64 / cost.max(1) as f64).min(1.0))
}

/// Moves the drill toward `goal` as far as this tick's energy pays for; true once it is there.
pub fn drill_to(world: &World, id: u64, dig: &mut Dig, goal: Vec3) -> bool {
    let length = point::distance(dig.head(), goal);
    let step = travel(world, id, length.min(DRILL_SPEED));
    if step >= length {
        dig.set_head(goal);
        return true;
    }
    dig.set_status(if step > 0.0 {
        Status::Mining
    } else {
        Status::NoPower
    });
    dig.set_head(point::moved_toward(dig.head(), goal, step));
    false
}

/// Pays for, at up to `draw` a tick, and mines the block `block_state` at `cell` into the
/// output of `quarry`.
pub fn mine_block(
    world: &World,
    quarry: Quarry,
    dig: &mut Dig,
    cell: Pos,
    block_state: BlockState,
    draw: i64,
) -> Mined {
    if !worth_mining(block_state) {
        return Mined::Skipped;
    }
    dig.set_status(Status::Mining);
    let id = quarry.id;
    let cost = block_cost(&dig.frame, cell, block_state, quarry.lubed);
    let draw = if quarry.lubed {
        lube::eased(draw)
    } else {
        draw
    };
    if !pay(world, id, dig, cell, cost, draw) {
        return Mined::Waiting(dig.status());
    }
    let keep = Keep::Quarry {
        multiplier: quarry.kind.rubble_per_ore,
    };
    match harvest(world, id, cell, block_state, keep) {
        // Paid for already: it stays paid until there is room.
        Harvest::Full => Mined::Waiting(Status::Full),
        Harvest::Refused => {
            dig.spent = 0;
            Mined::Skipped
        }
        Harvest::Done => {
            dig.spent = 0;
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
            Vec3::new(3.0, 5.0, 1.5),
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
            Vec3::new(3.75, 5.0, 2.5),
            "the other way on the next row"
        );
    }

    #[test]
    fn a_block_under_way_is_carried_on_from_where_the_drill_got_to() {
        let cell = Pos::new(3, 5, 1);
        assert_eq!(
            progress_point(&FRAME, cell, 500, 1000),
            Vec3::new(3.5, 5.0, 1.5),
            "half paid, half way"
        );
        assert_eq!(
            progress_point(&FRAME, cell, 1500, 1000),
            sweep(&FRAME, cell, 1.0),
            "never past the far edge"
        );
    }
}
