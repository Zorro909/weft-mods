//! Every oil engine's tick, at its controller: check the cube behind it, take in what its
//! intakes hold, run, hand the energy to its drives and push it out of them, then show it: the
//! shell formed or not, the gauges and lamp, the rig.
//!
//! A built engine shows its shell formed every tick, so where two engines share a wall and one
//! is taken apart, the other shows it formed again. One not built shows its whole cube plain once:
//! when it stops being built, and on the first tick of a new controller, in case the one before
//! went without being broken (an explosion, a command) and left its casing formed.

use weft_sdk::{ENERGY, SystemBatch, World};

use super::fuel::{Lack, oil, water};
use super::parts::{DRIVE_HOLDS, part_at, show_controller, show_formed};
use super::pattern::{self, Cube, Layout};
use super::{OIL_ENGINE_CONTROLLER, OilEngine, Status, rig};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid::HELD;

pub fn tick(batch: &mut SystemBatch<OilEngine>) {
    let world = batch.world();
    let time = world.game_time();
    for (id, engine, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&OIL_ENGINE_CONTROLLER) {
            continue;
        }
        let front = crate::machine::front(&world, pos);
        let cube = Cube::behind(pos, front);
        let shown_built = engine.status().is_built() || engine.checked == 0;
        engine.checked = 1;
        let running = match pattern::check(pos, front, |at| part_at(&world, at)) {
            Ok(layout) => {
                take_in(&world, engine, &layout);
                let running = run(engine, ticks);
                give_out(&world, engine, cube, &layout);
                form(&world, &layout.shell(), true);
                running
            }
            Err(problem) => {
                engine.set_unbuilt(problem);
                if shown_built {
                    unform(&world, cube);
                }
                false
            }
        };
        show(&world, id, pos, cube, engine, running, time);
    }
}

/// Runs the engine for `ticks` ticks as far as it can; whether it ran.
fn run(engine: &mut OilEngine, ticks: u32) -> bool {
    let (ran, lack) = engine.burn(ticks);
    let status = match lack {
        _ if ran > 0 => Status::Running,
        Some(Lack::Oil) => Status::NoOil,
        Some(Lack::Water) => Status::NoWater,
        Some(Lack::Room) => Status::Full,
        None => Status::Running,
    };
    engine.set_status(status);
    ran > 0
}

/// Takes the oil and water the intakes hold into the engine's stores, as far as they have room.
fn take_in(world: &World, engine: &mut OilEngine, layout: &Layout) {
    let (oil, water) = (oil(), water());
    for &intake in &layout.intakes {
        let Some(held) = world.component_mut(world.block_entity(intake), &HELD) else {
            continue;
        };
        let (oil_room, water_room) = engine.room();
        let (store, room) = match held.liquid {
            liquid if liquid == oil => (&mut engine.oil, oil_room),
            liquid if liquid == water => (&mut engine.water, water_room),
            _ => continue,
        };
        let taken = held.amount.min(room);
        if taken > 0 {
            *store += taken;
            held.remove(taken);
        }
    }
}

/// Fills each drive from the engine's energy, and pushes what a drive holds into the blocks it
/// faces outside the engine.
fn give_out(world: &World, engine: &mut OilEngine, cube: Cube, layout: &Layout) {
    for &drive in &layout.drives {
        let id = world.block_entity(drive);
        let Some(store) = world.component_mut(id, &ENERGY) else {
            continue;
        };
        let given = engine.energy.min(store.capacity - store.stored);
        store.stored += given;
        engine.energy -= given;
        for out in cube.outward(drive) {
            let stored = world.component(id, &ENERGY).map_or(0, |store| store.stored);
            if stored == 0 {
                break;
            }
            let moved = world.energy_move(drive.toward(out), stored.min(DRIVE_HOLDS), false);
            if let Some(store) = world.component_mut(id, &ENERGY) {
                store.stored -= moved;
            }
        }
    }
}

/// Shows the engine at `pos`: its gauges and lamp, its rig while built.
fn show(
    world: &World,
    id: u64,
    pos: Pos,
    cube: Cube,
    engine: &mut OilEngine,
    running: bool,
    time: u64,
) {
    show_controller(world, pos, engine.oil, engine.water, running);
    engine.keep_stroke(running, time);
    if engine.status().is_built() {
        rig::show(world, id, cube, engine, time);
    } else {
        engine.stroke = 0;
        rig::remove(world, engine);
    }
}

/// Shows each part at `shell` formed, or not.
fn form(world: &World, shell: &[Pos], formed: bool) {
    for &part in shell {
        show_formed(world, part, formed);
    }
}

/// Shows every part of `cube` as it is when the engine is not built.
fn unform(world: &World, cube: Cube) {
    let parts: Vec<Pos> = cube.area().blocks().collect();
    form(world, &parts, false);
}

/// The controller at `pos` was broken, its block `state` before: its engine's casing shows it
/// is not built any more. The rig goes with the controller (`owned`).
pub fn broken(world: &World, pos: Pos, state: BlockState) {
    unform(world, Cube::behind(pos, crate::machine::front_of(state)));
}
