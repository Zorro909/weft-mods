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
pub fn build_frame(w: &World, id: u64, d: &mut Dig) {
    let cells = d.frame.cells();
    while let Some(&c) = cells.get(d.step as usize) {
        let Some(block_state) = w.block_loaded_typed(c) else {
            d.set_status(Status::Blocked);
            return;
        };
        if !frame_passes_over(block_state) {
            put_up(w, id, d, c, block_state);
            return;
        }
        d.step += 1;
    }
    begin_mining(d);
}

/// Works on the frame block at `c`, where `block_state` is now: mined first if it is in the
/// way, then paid for and put up.
fn put_up(w: &World, id: u64, d: &mut Dig, c: Pos, block_state: BlockState) {
    if !block_state.can_build_into() {
        clear_for_frame(w, id, d, c, block_state);
        return;
    }
    d.set_status(Status::Frame);
    d.refused = 0;
    if !pay(w, id, d, FRAME_COST) {
        return;
    }
    w.set_block_typed(c, BlockState::of(&GANTRY_FRAME));
    if d.step.is_multiple_of(FRAME_SOUND_EVERY) {
        w.play_sound(&sounds::FRAME_PLACE, c.center(), 1.0);
    }
    d.spent = 0;
    d.step += 1;
}

/// Done with the frame: the drill starts in the middle of the first block to mine.
fn begin_mining(d: &mut Dig) {
    d.set_stage(Stage::Mine);
    d.cursor = d.frame.first();
    d.set_head(d.cursor.floor_middle());
}

/// Mines the block `block_state` at `c` out of the frame's way. While breaking it is refused (a
/// protected area), the excavator is jammed and tries only now and then, rather than paying for
/// the block every tick.
fn clear_for_frame(w: &World, id: u64, d: &mut Dig, c: Pos, block_state: BlockState) {
    if !try_now(d.refused, d.spent, w.game_time()) {
        d.set_status(Status::Jammed);
        return;
    }
    let mined = mine_block(w, id, d, c, block_state);
    let status = cleared(d, mined);
    d.set_status(status);
}

/// The status after a try at clearing the frame's way came to `mined`: refusals in a row are
/// counted, and enough of them jam the excavator.
fn cleared(d: &mut Dig, mined: Mined) -> Status {
    match mined {
        Mined::Done => {
            d.refused = 0;
            Status::Frame
        }
        Mined::Skipped => {
            d.refused += 1;
            if d.refused >= JAM_AFTER {
                Status::Jammed
            } else {
                Status::Frame
            }
        }
        // Mining for the frame is still building it.
        Mined::Waiting(Status::Mining) => Status::Frame,
        Mined::Waiting(s) => s,
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
        || block_state.hardness().is_some_and(|h| h < 0.0)
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
        let mut d = Dig::default();
        for _ in 1..JAM_AFTER {
            assert_eq!(cleared(&mut d, Mined::Skipped), Status::Frame, "not yet");
        }
        assert_eq!(cleared(&mut d, Mined::Skipped), Status::Jammed, "jammed");
        assert_eq!(d.refused, JAM_AFTER, "every refusal counted");
    }

    #[test]
    fn a_cleared_block_ends_the_jam() {
        let mut d = Dig {
            refused: JAM_AFTER,
            ..Dig::default()
        };
        assert_eq!(
            cleared(&mut d, Mined::Done),
            Status::Frame,
            "building again"
        );
        assert_eq!(d.refused, 0, "the count starts over");
    }

    #[test]
    fn waiting_while_clearing_says_why() {
        let mut d = Dig::default();
        let waiting = |d: &mut Dig, why| cleared(d, Mined::Waiting(why));
        assert_eq!(
            waiting(&mut d, Status::Mining),
            Status::Frame,
            "mining for the frame is building it"
        );
        assert_eq!(waiting(&mut d, Status::Full), Status::Full, "output full");
        assert_eq!(
            waiting(&mut d, Status::NoPower),
            Status::NoPower,
            "no power"
        );
        assert_eq!(d.refused, 0, "waiting is no refusal");
    }

    #[test]
    fn mining_starts_in_the_middle_of_the_first_block() {
        let mut d = Dig {
            frame: Frame::from(([0, 4, 0], [10, 8, 10])),
            ..Dig::default()
        };
        begin_mining(&mut d);
        assert_eq!(d.stage(), Stage::Mine, "mining");
        assert_eq!(
            d.cursor,
            Pos::new(1, 8, 1),
            "a corner of the inside, at the top"
        );
        let middle = Vec3::new(1.5, 8.0, 1.5);
        assert_eq!(d.head(), middle, "the drill over its middle");
    }
}
