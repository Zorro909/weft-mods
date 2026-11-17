//! A quarry's tick: plan the area, build the frame (`framing`), then mine, and pass on what it
//! mined. What its bit can't break it leaves standing, a pillar in the pit.

use weft_sdk::{Output, SystemBatch, World};

use super::depth::Floor;
use super::framing::build_frame;
use super::kind::{Drill, Kind};
use super::mine::{Mined, block_cost, drill_to, mine_block, progress_point, sweep};
use super::power::{fit_store, hold};
use super::{Dig, Frame, Stage, Status, bit, rig};
use crate::lib::{Pos, TypedWorld};
use crate::sounds;

/// The most blocks looked at per tick for the next one to mine (air is skipped).
const SCAN: u32 = 512;
/// Ticks a layer may stay unloaded before the excavator takes it for the floor of the world.
const FLOOR_WAIT: u32 = 200;

pub fn tick(batch: &mut SystemBatch<Dig>) {
    let world = batch.world();
    for (id, dig, _ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        let Some(kind) = Kind::of(world.block_typed(pos)) else {
            continue;
        };
        crate::machine::start_meter(&world, id);
        let floor = kind.floor(&dig.frame, Floor::now());
        let drill = kind.drill(bit::fitted(&world, id));
        work(&world, Quarry { id, pos, kind }, dig, drill, floor);
        super::keep_loaded(&world, pos, dig, floor);
        rig::update(&world, id, dig, kind.rig, drill);
        crate::machine::show_working(&world, pos, dig.status().is_working());
    }
}

/// The quarry a tick works on: its block entity, where it stands and its kind.
#[derive(Clone, Copy)]
pub struct Quarry {
    pub id: u64,
    pub pos: Pos,
    pub kind: &'static Kind,
}

/// Passes on what `quarry` mined, then works on its stage, digging with `drill` if it has one
/// down to `floor`.
fn work(world: &World, quarry: Quarry, dig: &mut Dig, drill: Option<Drill>, floor: Floor) {
    if dig.stage() == Stage::New {
        start(world, quarry, dig);
    }
    crate::store::eject(world, quarry.pos);
    match dig.stage() {
        Stage::Frame | Stage::Mine if !hold(world, quarry, dig) => {}
        Stage::Frame => build_frame(world, quarry, dig, drill),
        Stage::Mine => match drill {
            Some(drill) => mine_next(world, quarry, dig, drill, floor),
            None => dig.set_status(Status::NeedsBit),
        },
        Stage::New | Stage::Done => {}
    }
}

/// Plans the area of `quarry` and sizes its store to it.
fn start(world: &World, quarry: Quarry, dig: &mut Dig) {
    let Quarry { pos, kind, .. } = quarry;
    dig.frame = Frame::plan(world, pos, crate::machine::front(world, pos), kind);
    fit_store(
        world,
        quarry,
        &dig.frame,
        dig.frame.below(dig.frame.first()),
        0,
    );
    dig.set_stage(Stage::Frame);
    dig.step = 0;
}

/// Mines toward the next block there is to mine with `drill`, for `quarry`, down to `floor`.
fn mine_next(world: &World, quarry: Quarry, dig: &mut Dig, drill: Drill, floor: Floor) {
    let Quarry { id, pos, kind } = quarry;
    fit_store(
        world,
        quarry,
        &dig.frame,
        dig.frame.below(dig.cursor),
        drill.draw,
    );
    let Some(target) = find_target(world, dig, drill, floor) else {
        if dig.stage() == Stage::Done {
            world.play_sound(&sounds::EXCAVATOR_DONE, pos.center(), 1.0);
        }
        return;
    };
    let block_state = world.block_typed(target);
    let cost = block_cost(&dig.frame, target, block_state);
    // Where the energy paid so far has the drill: into a block it has begun, it carries on from
    // there instead of going back to its edge.
    let reached = progress_point(&dig.frame, target, dig.spent, cost);
    if !drill_to(world, id, dig, reached) {
        return;
    }
    match mine_block(world, quarry, dig, target, block_state, drill.draw) {
        Mined::Done => {
            bit::wear(world, id);
            world.play_sound(kind.dig_sound, target.center(), 1.0);
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

/// The next block worth mining from the cursor on, skipping air, liquids, what can't be broken
/// and what `drill` doesn't break; `None` while waiting or once finished at `floor`.
fn find_target(world: &World, dig: &mut Dig, drill: Drill, floor: Floor) -> Option<Pos> {
    for _ in 0..SCAN {
        let cell = dig.cursor;
        if floor.is_above(cell.y) {
            finish(dig);
            return None;
        }
        let Some(block_state) = world.block_loaded_typed(cell) else {
            wait_for_load(dig);
            return None;
        };
        dig.waited = 0;
        if super::mine::worth_mining(block_state) && bit::breaks(drill.tier, block_state) {
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
