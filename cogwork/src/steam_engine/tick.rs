//! Every steam engine's tick: run on the steam it holds for the ticks the call stands for, push
//! its energy out of both ends of its flywheel's axle, note on its meter what left it, then show
//! it: the rig running, chuffing and letting out puffs of steam.

use weft_sdk::{ENERGY, SystemBatch, World};

use super::{STEAM_ENGINE, Status, SteamEngine, rig, run};
use crate::lib::{Pos, TypedWorld};
use crate::liquid::HELD;

pub fn tick(batch: &mut SystemBatch<SteamEngine>) {
    let world = batch.world();
    for (id, engine, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&STEAM_ENGINE) {
            continue;
        }
        crate::machine::start_meter(&world, id);
        let made = run_for(&world, id, engine, ticks);
        let front = crate::machine::front(&world, pos);
        for out in [front.clockwise(), front.counter_clockwise()] {
            crate::engines::push_into(&world, id, pos, out);
        }
        let after = stored(&world, id);
        let given = (engine.stored_after + made - after).max(0) / i64::from(ticks.max(1));
        crate::machine::note_drawn(&world, id, given, given);
        engine.stored_after = after;
        let running = engine.status() == Status::Running;
        rig::show(&world, id, pos, front, engine, running);
    }
}

/// Runs the engine `id` on its steam for `ticks`, as far as it can; the energy it made.
fn run_for(world: &World, id: u64, engine: &mut SteamEngine, ticks: u32) -> i64 {
    let held = world.component(id, &HELD).unwrap_or_default();
    let steam = if held.liquid == crate::steam::key() {
        held.amount
    } else {
        0
    };
    let (ran, status) = engine.run(ticks, steam, stored(world, id));
    engine.set_status(status);
    if ran == 0 {
        return 0;
    }
    if let Some(held) = world.component_mut(id, &HELD) {
        held.remove(ran * run::STEAM_A_TICK);
    }
    let made = i64::from(ran) * run::MAKES;
    if let Some(energy) = world.component_mut(id, &ENERGY) {
        energy.stored = (energy.stored + made).min(energy.capacity);
    }
    made
}

/// The energy the engine `id` holds.
fn stored(world: &World, id: u64) -> i64 {
    world
        .component(id, &ENERGY)
        .map_or(0, |energy| energy.stored)
}
