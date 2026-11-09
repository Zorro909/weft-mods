//! An excavator's tick: plan the area, build the frame (`framing`), then mine, and pass on what
//! it mined.

use weft_sdk::*;

use super::framing::build_frame;
use super::mine::{Mined, block_cost, drill_to, mine_block, progress_point, sweep};
use super::power::hold;
use super::{Dig, EXCAVATOR, Frame, Stage, Status, rig};
use crate::lib::{Pos, TypedWorld};
use crate::sounds;

/// The most blocks looked at per tick for the next one to mine (air is skipped).
const SCAN: u32 = 512;
/// Ticks a layer may stay unloaded before the excavator takes it for the floor of the world.
const FLOOR_WAIT: u32 = 200;
/// The deepest layer it mines, like the floor of vanilla's worlds.
pub const FLOOR_Y: i32 = -64;

pub fn tick(b: &mut SystemBatch<Dig>) {
    let w = b.world();
    for (id, d, _ticks) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        let pos = Pos::from(st.block_pos());
        if !w.block_typed(pos).is(&EXCAVATOR) {
            continue;
        }
        work(&w, id, pos, d);
        super::keep_loaded(&w, pos, d);
        rig::update(&w, id, d);
        crate::machine::show_working(&w, pos, d.status().is_working());
    }
}

/// Passes on what the excavator `id` at `pos` mined, then works on its stage.
fn work(w: &World, id: u64, pos: Pos, d: &mut Dig) {
    if d.stage() == Stage::New {
        start(w, pos, d);
    }
    crate::store::eject(w, pos);
    match d.stage() {
        Stage::Frame | Stage::Mine if !hold(w, id, d) => {}
        Stage::Frame => build_frame(w, id, d),
        Stage::Mine => mine_next(w, id, pos, d),
        Stage::New | Stage::Done => {}
    }
}

fn start(w: &World, pos: Pos, d: &mut Dig) {
    d.frame = Frame::plan(w, pos, crate::machine::front(w, pos));
    d.set_stage(Stage::Frame);
    d.step = 0;
}

/// Mines toward the next block that is there to mine, for the excavator `id` at `pos`.
fn mine_next(w: &World, id: u64, pos: Pos, d: &mut Dig) {
    let Some(target) = find_target(w, d) else {
        if d.stage() == Stage::Done {
            w.play_sound(&sounds::EXCAVATOR_DONE, pos.center(), 1.0);
        }
        return;
    };
    let block_state = w.block_typed(target);
    let cost = block_cost(block_state);
    // Where the energy paid so far has the drill: into a block it has begun, it carries on from
    // there instead of going back to its edge.
    if !drill_to(w, id, d, progress_point(&d.frame, target, d.spent, cost)) {
        return;
    }
    match mine_block(w, id, d, target, block_state) {
        Mined::Done => {
            w.play_sound(&sounds::DRILL, target.center(), 1.0);
            d.set_head(sweep(&d.frame, target, 1.0));
            d.cursor = d.frame.next(target);
        }
        Mined::Skipped => d.cursor = d.frame.next(target),
        Mined::Waiting(s) => {
            d.set_head(progress_point(&d.frame, target, d.spent, cost));
            d.set_status(s);
        }
    }
}

/// The next block worth mining from the cursor on, skipping air, liquids and what can't be
/// broken; `None` while waiting or once finished.
fn find_target(w: &World, d: &mut Dig) -> Option<Pos> {
    for _ in 0..SCAN {
        let c = d.cursor;
        if c.y < FLOOR_Y {
            finish(d);
            return None;
        }
        let Some(block_state) = w.block_loaded_typed(c) else {
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
