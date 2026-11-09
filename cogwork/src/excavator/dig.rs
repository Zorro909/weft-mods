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

pub fn tick(batch: &mut SystemBatch<Dig>) {
    let world = batch.world();
    for (id, dig, _ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&EXCAVATOR) {
            continue;
        }
        work(&world, id, pos, dig);
        super::keep_loaded(&world, pos, dig);
        rig::update(&world, id, dig);
        crate::machine::show_working(&world, pos, dig.status().is_working());
    }
}

/// Passes on what the excavator `id` at `pos` mined, then works on its stage.
fn work(world: &World, id: u64, pos: Pos, dig: &mut Dig) {
    if dig.stage() == Stage::New {
        start(world, pos, dig);
    }
    crate::store::eject(world, pos);
    match dig.stage() {
        Stage::Frame | Stage::Mine if !hold(world, id, dig) => {}
        Stage::Frame => build_frame(world, id, dig),
        Stage::Mine => mine_next(world, id, pos, dig),
        Stage::New | Stage::Done => {}
    }
}

fn start(world: &World, pos: Pos, dig: &mut Dig) {
    dig.frame = Frame::plan(world, pos, crate::machine::front(world, pos));
    dig.set_stage(Stage::Frame);
    dig.step = 0;
}

/// Mines toward the next block that is there to mine, for the excavator `id` at `pos`.
fn mine_next(world: &World, id: u64, pos: Pos, dig: &mut Dig) {
    let Some(target) = find_target(world, dig) else {
        if dig.stage() == Stage::Done {
            world.play_sound(&sounds::EXCAVATOR_DONE, pos.center(), 1.0);
        }
        return;
    };
    let block_state = world.block_typed(target);
    let cost = block_cost(block_state);
    // Where the energy paid so far has the drill: into a block it has begun, it carries on from
    // there instead of going back to its edge.
    let reached = progress_point(&dig.frame, target, dig.spent, cost);
    if !drill_to(world, id, dig, reached) {
        return;
    }
    match mine_block(world, id, dig, target, block_state) {
        Mined::Done => {
            world.play_sound(&sounds::DRILL, target.center(), 1.0);
            dig.set_head(sweep(&dig.frame, target, 1.0));
            dig.cursor = dig.frame.next(target);
        }
        Mined::Skipped => dig.cursor = dig.frame.next(target),
        Mined::Waiting(status) => {
            dig.set_head(progress_point(&dig.frame, target, dig.spent, cost));
            dig.set_status(status);
        }
    }
}

/// The next block worth mining from the cursor on, skipping air, liquids and what can't be
/// broken; `None` while waiting or once finished.
fn find_target(world: &World, dig: &mut Dig) -> Option<Pos> {
    for _ in 0..SCAN {
        let cell = dig.cursor;
        if cell.y < FLOOR_Y {
            finish(dig);
            return None;
        }
        let Some(block_state) = world.block_loaded_typed(cell) else {
            wait_for_load(dig);
            return None;
        };
        dig.waited = 0;
        if super::mine::worth_mining(block_state) {
            return Some(cell);
        }
        dig.cursor = dig.frame.next(cell);
    }
    dig.set_status(Status::Mining);
    None
}

/// Waits a tick for the cursor's block to load; one that never does is taken for the floor of
/// the world.
fn wait_for_load(dig: &mut Dig) {
    dig.set_status(Status::Blocked);
    dig.waited += 1;
    if dig.waited > FLOOR_WAIT {
        finish(dig);
    }
}

fn finish(dig: &mut Dig) {
    dig.set_stage(Stage::Done);
    dig.set_status(Status::Done);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_layer_that_never_loads_is_the_floor_of_the_world() {
        let mut dig = Dig {
            stage: Stage::Mine as u32,
            ..Dig::default()
        };
        for _ in 0..FLOOR_WAIT {
            wait_for_load(&mut dig);
        }
        assert_eq!(dig.status(), Status::Blocked, "waiting for it to load");
        assert_eq!(dig.stage(), Stage::Mine, "still mining");
        wait_for_load(&mut dig);
        assert_eq!(dig.stage(), Stage::Done, "done after waiting too long");
        assert_eq!(dig.status(), Status::Done, "and says so");
    }
}
