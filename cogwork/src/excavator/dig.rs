//! An excavator's tick: plan the area, build the frame, then mine, and pass on what it mined.

use weft_sdk::*;

use super::mine::{Mined, block_cost, drill_to, floor_middle, mine_block, progress_point, sweep};
use super::power::{hold, pay};
use super::{Dig, EXCAVATOR, Frame, GANTRY_FRAME, Stage, Status, rig};
use crate::harvest::opens_into;

/// Energy to put up one block of frame.
const FRAME_COST: i64 = 100;
/// The most blocks looked at per tick for the next one to mine (air is skipped).
const SCAN: u32 = 512;
/// Ticks a layer may stay unloaded before the excavator takes it for the floor of the world.
const FLOOR_WAIT: u32 = 200;
/// The deepest layer it mines, like the floor of vanilla's worlds.
const FLOOR_Y: i32 = -64;
/// Refused tries in a row at clearing the frame's way before the excavator is jammed, and the
/// ticks between its tries while it is.
const JAM_AFTER: u32 = 3;
const JAM_RETRY: u64 = 100;

pub fn tick(b: &mut SystemBatch<Dig>) {
    let w = b.world();
    for (id, d, _ticks) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        let pos = st.block_pos();
        if !EXCAVATOR.is(w.get(pos)) {
            continue;
        }
        work(&w, id, pos, d);
        rig::update(&w, id, d);
        crate::machine::show_working(&w, pos, d.status().is_working());
    }
}

/// Passes on what the excavator `id` at `pos` mined, then works on its stage.
fn work(w: &World, id: u64, pos: [i32; 3], d: &mut Dig) {
    if d.stage() == Stage::New {
        start(w, pos, d);
    }
    crate::store::eject(w, id, pos);
    match d.stage() {
        Stage::Frame | Stage::Mine if !hold(w, id, d) => {}
        Stage::Frame => build_frame(w, id, d),
        Stage::Mine => mine_next(w, id, d),
        Stage::New | Stage::Done => {}
    }
}

fn start(w: &World, pos: [i32; 3], d: &mut Dig) {
    d.frame = Frame::plan(w, pos, crate::machine::front(w, pos));
    d.set_stage(Stage::Frame);
    d.step = 0;
}

/// One block of frame a tick: mined first if something is in the way.
fn build_frame(w: &World, id: u64, d: &mut Dig) {
    let cells = d.frame.cells();
    while let Some(&c) = cells.get(d.step as usize) {
        let Some(block_state) = w.get_loaded(c) else {
            d.set_status(Status::Blocked);
            return;
        };
        if frame_passes_over(block_state) {
            d.step += 1;
            continue;
        }
        if !opens_into(block_state) {
            clear_for_frame(w, id, d, c, block_state);
            return;
        }
        d.set_status(Status::Frame);
        d.refused = 0;
        if pay(w, id, d, FRAME_COST) {
            w.set(c, GANTRY_FRAME.id());
            d.spent = 0;
            d.step += 1;
        }
        return;
    }
    begin_mining(d);
}

/// Done with the frame: the drill starts in the middle of the first block to mine.
fn begin_mining(d: &mut Dig) {
    d.set_stage(Stage::Mine);
    d.cursor = d.frame.first();
    d.head = floor_middle(d.cursor);
}

/// Mines the block `block_state` at `c` out of the frame's way. While breaking it is refused (a
/// protected area), the excavator is jammed and tries only now and then, rather than paying for
/// the block every tick.
fn clear_for_frame(w: &World, id: u64, d: &mut Dig, c: [i32; 3], block_state: u32) {
    if !try_now(d.refused, d.spent, w.time()) {
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
fn frame_passes_over(block_state: u32) -> bool {
    GANTRY_FRAME.is(block_state)
        || EXCAVATOR.is(block_state)
        || HARDNESS.on_block(block_state).is_some_and(|h| h < 0.0)
}

/// Mines toward the next block that is there to mine.
fn mine_next(w: &World, id: u64, d: &mut Dig) {
    let Some(target) = find_target(w, d) else {
        return;
    };
    let block_state = w.get(target);
    let cost = block_cost(block_state);
    // Where the energy paid so far has the drill: into a block it has begun, it carries on from
    // there instead of going back to its edge.
    if !drill_to(w, id, d, progress_point(&d.frame, target, d.spent, cost)) {
        return;
    }
    match mine_block(w, id, d, target, block_state) {
        Mined::Done => {
            d.head = sweep(&d.frame, target, 1.0);
            d.cursor = d.frame.next(target);
        }
        Mined::Skipped => d.cursor = d.frame.next(target),
        Mined::Waiting(s) => {
            d.head = progress_point(&d.frame, target, d.spent, cost);
            d.set_status(s);
        }
    }
}

/// The next block worth mining from the cursor on, skipping air, liquids and what can't be
/// broken; `None` while waiting or once finished.
fn find_target(w: &World, d: &mut Dig) -> Option<[i32; 3]> {
    for _ in 0..SCAN {
        let c = d.cursor;
        if c[1] < FLOOR_Y {
            finish(d);
            return None;
        }
        let Some(block_state) = w.get_loaded(c) else {
            wait_for_load(d);
            return None;
        };
        d.waited = 0;
        if super::mine::worth_mining(block_state) {
            return Some(c);
        }
        d.cursor = d.frame.next(c);
    }
    d.set_status(Status::Mining);
    None
}

/// Waits a tick for the cursor's block to load; one that never does is taken for the floor of
/// the world.
fn wait_for_load(d: &mut Dig) {
    d.set_status(Status::Blocked);
    d.waited += 1;
    if d.waited > FLOOR_WAIT {
        finish(d);
    }
}

fn finish(d: &mut Dig) {
    d.set_stage(Stage::Done);
    d.set_status(Status::Done);
}

#[cfg(test)]
mod tests {
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
            frame: Frame {
                lo: [0, 4, 0],
                hi: [10, 8, 10],
            },
            ..Dig::default()
        };
        begin_mining(&mut d);
        assert_eq!(d.stage(), Stage::Mine, "mining");
        assert_eq!(d.cursor, [1, 8, 1], "a corner of the inside, at the top");
        assert_eq!(d.head, [1.5, 8.0, 1.5], "the drill over its middle");
    }

    #[test]
    fn a_layer_that_never_loads_is_the_floor_of_the_world() {
        let mut d = Dig {
            stage: Stage::Mine as u32,
            ..Dig::default()
        };
        for _ in 0..FLOOR_WAIT {
            wait_for_load(&mut d);
        }
        assert_eq!(d.status(), Status::Blocked, "waiting for it to load");
        assert_eq!(d.stage(), Stage::Mine, "still mining");
        wait_for_load(&mut d);
        assert_eq!(d.stage(), Stage::Done, "done after waiting too long");
        assert_eq!(d.status(), Status::Done, "and says so");
    }
}
