//! Putting up a quarry's frame: one block a tick, each paid for in energy and in what its kind
//! builds frames of, and whatever stands in the way mined first with the quarry's drill. Where it
//! may not break what is in the way, it jams and tries again only now and then.

use weft_sdk::{Output, World};

use super::dig::Quarry;
use super::kind::{Drill, Kind};
use super::mine::{Mined, mine_block, wear_down};
use super::power::pay;
use super::{Dig, Stage, Status, bit, frames, materials};
use crate::lib::{BlockState, Pos, TypedWorld};

/// Energy to put up one block of frame.
const FRAME_COST: i64 = 100;
/// Refused tries in a row at clearing the frame's way before the excavator is jammed, and the
/// ticks between its tries while it is.
const JAM_AFTER: u32 = 3;
const JAM_RETRY: u64 = 100;
/// Frame blocks per clank: one a tick would be a din.
const FRAME_SOUND_EVERY: u32 = 4;

/// What the frame does about a block that stands in its way.
enum Way {
    /// Mines it with this drill.
    Clear(Drill),
    /// Leaves it standing, as mining leaves a pillar: the drill can't break it.
    Leave,
    /// Waits for a bit that fits.
    NeedsBit,
}

/// One block of frame a tick, after those the frame passes over; mining begins once all are up.
/// What stands in the way is cleared with `drill`, if the quarry has one.
pub fn build_frame(world: &World, quarry: Quarry, dig: &mut Dig, drill: Option<Drill>) {
    let cells = dig.frame.cells();
    while let Some(&cell) = cells.get(dig.step as usize) {
        let Some(block_state) = world.block_loaded_typed(cell) else {
            dig.set_status(Status::Blocked);
            return;
        };
        if frame_passes_over(block_state) {
            dig.step += 1;
            continue;
        }
        if block_state.can_build_into() {
            put_up(world, quarry, dig, cell);
            return;
        }
        match way(drill, bit::needed(block_state)) {
            Way::Clear(drill) => clear_for_frame(world, quarry, dig, cell, block_state, drill),
            Way::NeedsBit => dig.set_status(Status::NeedsBit),
            Way::Leave => {
                leave(dig);
                continue;
            }
        }
        return;
    }
    begin_mining(dig);
}

/// What to do about a block in the frame's way that a drill of tier `needs` breaks, with `drill`.
fn way(drill: Option<Drill>, needs: u32) -> Way {
    match drill {
        None => Way::NeedsBit,
        Some(drill) if needs <= drill.tier => Way::Clear(drill),
        Some(_) => Way::Leave,
    }
}

/// Passes over the block in the frame's way, dropping any try at it under way.
fn leave(dig: &mut Dig) {
    dig.refused = 0;
    dig.spent = 0;
    dig.step += 1;
}

/// Puts up the frame block at `cell`, which is clear: paid for, and its batch's materials taken
/// if nothing is paid ahead.
fn put_up(world: &World, quarry: Quarry, dig: &mut Dig, cell: Pos) {
    dig.refused = 0;
    let takes_batch = materials::takes_batch(dig.paid_ahead);
    if takes_batch && !materials::take(world, quarry, true) {
        dig.set_status(Status::NeedsMaterials);
        return;
    }
    dig.set_status(Status::Frame);
    let draw = quarry.kind.build_draw();
    if !pay(world, quarry.id, dig, cell, FRAME_COST, draw)
        || (takes_batch && !materials::take(world, quarry, false))
    {
        return;
    }
    frames::put(world, cell, quarry.kind.frame, takes_batch);
    dig.paid_ahead = materials::after_putting_up(dig.paid_ahead, quarry.kind.frame_batch);
    if dig.step.is_multiple_of(FRAME_SOUND_EVERY) {
        world.play_sound(quarry.kind.frame_sound, cell.center(), 1.0);
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

/// Mines the block `block_state` at `cell` out of the frame's way with `drill`, wearing its bit as
/// mining does. While breaking it is refused (a protected area), the excavator is jammed and
/// tries only now and then, rather than paying for the block every tick.
fn clear_for_frame(
    world: &World,
    quarry: Quarry,
    dig: &mut Dig,
    cell: Pos,
    block_state: BlockState,
    drill: Drill,
) {
    if !try_now(dig.refused, dig.spent, world.game_time()) {
        dig.set_status(Status::Jammed);
        return;
    }
    let mined = mine_block(world, quarry, dig, cell, block_state, drill.draw);
    if matches!(mined, Mined::Done) {
        wear_down(world, quarry, dig, drill);
    }
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

/// Whether the frame leaves a block as it is: a frame already, a quarry, or what can't be broken.
fn frame_passes_over(block_state: BlockState) -> bool {
    Kind::is_part(block_state)
        || block_state
            .hardness()
            .is_some_and(|hardness| hardness < 0.0)
}

#[cfg(test)]
mod tests {
    use weft_sdk::Vec3;
    use weft_sdk::tool_tier;

    use super::super::Frame;
    use super::super::bit::{IRON_BIT, STONE_BIT};
    use super::super::kind::rows::{GANTRY, TIMBER};
    use super::*;

    #[test]
    fn a_timber_quarry_with_no_bit_clears_nothing() {
        let way = way(TIMBER.drill(None), tool_tier::WOOD);
        assert!(matches!(way, Way::NeedsBit), "not even dirt");
    }

    #[test]
    fn the_way_is_cleared_with_the_fitted_bit() {
        let Way::Clear(stone) = way(TIMBER.drill(Some(&STONE_BIT)), tool_tier::STONE) else {
            panic!("a stone bit takes iron ore");
        };
        assert_eq!(stone.draw, 12, "at the stone bit's draw, not the best");
        assert!(stone.bit.is_some(), "and wears the bit");
    }

    #[test]
    fn what_the_bit_cannot_break_is_left_standing() {
        let gold = tool_tier::IRON;
        let stone = way(TIMBER.drill(Some(&STONE_BIT)), gold);
        assert!(
            matches!(stone, Way::Leave),
            "gold stands against a stone bit"
        );
        let iron = way(TIMBER.drill(Some(&IRON_BIT)), gold);
        assert!(matches!(iron, Way::Clear(_)), "an iron bit takes it");
    }

    #[test]
    fn the_gantry_clears_anything_with_no_bit() {
        let way = way(GANTRY.drill(None), tool_tier::DIAMOND);
        let Way::Clear(bare) = way else {
            panic!("the gantry digs bare");
        };
        assert_eq!(bare.draw, 120, "as today");
    }

    #[test]
    fn leaving_a_block_drops_the_try_at_it() {
        let mut dig = Dig {
            refused: 1,
            spent: 40,
            step: 3,
            ..Dig::default()
        };
        leave(&mut dig);
        assert_eq!(
            (dig.refused, dig.spent, dig.step),
            (0, 0, 4),
            "on to the next"
        );
    }

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
