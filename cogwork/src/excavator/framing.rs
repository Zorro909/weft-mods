//! Putting up an excavator's frame: one block a tick, each paid for, and whatever stands in the
//! way mined first. Where it may not break what is in the way, it jams and tries again only now
//! and then.

use weft_sdk::*;

use super::mine::{Mined, mine_block};
use super::power::pay;
use super::{Dig, EXCAVATOR, GANTRY_FRAME, Stage, Status};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::sounds;

/// Energy to put up one block of frame.
const FRAME_COST: i64 = 100;
/// Refused tries in a row at clearing the frame's way before the excavator is jammed, and the
/// ticks between its tries while it is.
const JAM_AFTER: u32 = 3;
const JAM_RETRY: u64 = 100;
/// Frame blocks per clank: one a tick would be a din.
const FRAME_SOUND_EVERY: u32 = 4;

/// One block of frame a tick, after those the frame passes over; mining begins once all are up.
pub fn build_frame(world: &World, id: u64, dig: &mut Dig) {
    let cells = dig.frame.cells();
    while let Some(&cell) = cells.get(dig.step as usize) {
        let Some(block_state) = world.block_loaded_typed(cell) else {
            dig.set_status(Status::Blocked);
            return;
        };
        if !frame_passes_over(block_state) {
            put_up(world, id, dig, cell, block_state);
            return;
        }
        dig.step += 1;
    }
    begin_mining(dig);
}

/// Works on the frame block at `cell`, where `block_state` is now: mined first if it is in the
/// way, then paid for and put up.
fn put_up(world: &World, id: u64, dig: &mut Dig, cell: Pos, block_state: BlockState) {
    if !block_state.can_build_into() {
        clear_for_frame(world, id, dig, cell, block_state);
        return;
    }
    dig.set_status(Status::Frame);
    dig.refused = 0;
    if !pay(world, id, dig, cell, FRAME_COST) {
        return;
    }
    world.set_block_typed(cell, BlockState::of(&GANTRY_FRAME));
    if dig.step.is_multiple_of(FRAME_SOUND_EVERY) {
        world.play_sound(&sounds::FRAME_PLACE, cell.center(), 1.0);
    }
    dig.spent = 0;
    dig.step += 1;
}

/// Done with the frame: the drill starts in the middle of the first block to mine.
fn begin_mining(dig: &mut Dig) {
    dig.set_stage(Stage::Mine);
    dig.cursor = dig.frame.first();
    dig.set_head(dig.cursor.floor_middle());
}

/// Mines the block `block_state` at `cell` out of the frame's way. While breaking it is refused (a
/// protected area), the excavator is jammed and tries only now and then, rather than paying for
/// the block every tick.
fn clear_for_frame(world: &World, id: u64, dig: &mut Dig, cell: Pos, block_state: BlockState) {
    if !try_now(dig.refused, dig.spent, world.game_time()) {
        dig.set_status(Status::Jammed);
        return;
    }
    let mined = mine_block(world, id, dig, cell, block_state);
    let status = cleared(dig, mined);
    dig.set_status(status);
}

/// The status after a try at clearing the frame's way came to `mined`: refusals in a row are
/// counted, and enough of them jam the excavator.
fn cleared(dig: &mut Dig, mined: Mined) -> Status {
    match mined {
        Mined::Done => {
            dig.refused = 0;
            Status::Frame
        }
        Mined::Skipped => {
            dig.refused += 1;
            if dig.refused >= JAM_AFTER {
                Status::Jammed
            } else {
                Status::Frame
            }
        }
        // Mining for the frame is still building it.
        Mined::Waiting(Status::Mining) => Status::Frame,
        Mined::Waiting(status) => status,
    }
}

/// Whether to try clearing the frame's way this tick, after `refused` refusals in a row and with
/// `spent` paid into the try under way: always until it is jammed, then once every `JAM_RETRY`
/// ticks, carrying a try on until it is paid for.
fn try_now(refused: u32, spent: i64, now: u64) -> bool {
    refused < JAM_AFTER || spent > 0 || now.is_multiple_of(JAM_RETRY)
}

/// Whether the frame leaves a block as it is: frame already, the excavator itself, or what
/// can't be broken.
fn frame_passes_over(block_state: BlockState) -> bool {
    block_state.is(&GANTRY_FRAME)
        || block_state.is(&EXCAVATOR)
        || block_state
            .hardness()
            .is_some_and(|hardness| hardness < 0.0)
}

#[cfg(test)]
mod tests {
    use super::super::Frame;
    use super::*;

    #[test]
    fn a_frame_refused_again_and_again_is_tried_only_now_and_then() {
        let tried = |refused| {
            (1..=JAM_RETRY)
                .filter(|&now| try_now(refused, 0, now))
                .count()
        };
        assert_eq!(
            tried(JAM_AFTER - 1),
            JAM_RETRY as usize,
            "every tick until jammed"
        );
        assert_eq!(tried(JAM_AFTER), 1, "then once a while");
        assert!(try_now(JAM_AFTER, 300, 1), "a try under way goes on");
    }

    #[test]
    fn refusals_in_a_row_jam_the_excavator() {
        let mut dig = Dig::default();
        for _ in 1..JAM_AFTER {
            assert_eq!(cleared(&mut dig, Mined::Skipped), Status::Frame, "not yet");
        }
        assert_eq!(cleared(&mut dig, Mined::Skipped), Status::Jammed, "jammed");
        assert_eq!(dig.refused, JAM_AFTER, "every refusal counted");
    }

    #[test]
    fn a_cleared_block_ends_the_jam() {
        let mut dig = Dig {
            refused: JAM_AFTER,
            ..Dig::default()
        };
        assert_eq!(
            cleared(&mut dig, Mined::Done),
            Status::Frame,
            "building again"
        );
        assert_eq!(dig.refused, 0, "the count starts over");
    }

    #[test]
    fn waiting_while_clearing_says_why() {
        let mut dig = Dig::default();
        let waiting = |dig: &mut Dig, why| cleared(dig, Mined::Waiting(why));
        assert_eq!(
            waiting(&mut dig, Status::Mining),
            Status::Frame,
            "mining for the frame is building it"
        );
        assert_eq!(waiting(&mut dig, Status::Full), Status::Full, "output full");
        assert_eq!(
            waiting(&mut dig, Status::NoPower),
            Status::NoPower,
            "no power"
        );
        assert_eq!(dig.refused, 0, "waiting is no refusal");
    }

    #[test]
    fn mining_starts_in_the_middle_of_the_first_block() {
        let mut dig = Dig {
            frame: Frame::from(([0, 4, 0], [10, 8, 10])),
            ..Dig::default()
        };
        begin_mining(&mut dig);
        assert_eq!(dig.stage(), Stage::Mine, "mining");
        assert_eq!(
            dig.cursor,
            Pos::new(1, 8, 1),
            "a corner of the inside, at the top"
        );
        let middle = Vec3::new(1.5, 8.0, 1.5);
        assert_eq!(dig.head(), middle, "the drill over its middle");
    }
}
