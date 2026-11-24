//! Every core drill's tick: keep its rod in place, count up to `BUDGET` of the blocks its turns
//! have drilled to, keeping the layer it counts loaded meanwhile, and report to the player who
//! turned it last once it has counted everything. A block that stays unloaded for `FLOOR_WAIT`
//! is taken for the floor of the world: the count ends there.

use std::collections::HashMap;

use weft_sdk::{SystemBatch, World};

use super::survey::{self, TOTAL};
use super::{CORE_DRILL, Core, report, rod};
use crate::ghost::{ItemKey, key_of_name};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::loading::Cubes;

/// The most blocks a drill counts a tick: a turn's half band takes two ticks.
const BUDGET: u32 = 1_024;
/// Ticks a block may stay unloaded before the count takes it for the floor of the world.
const FLOOR_WAIT: u32 = 200;

/// What a block is to the count: the keys of its kind of ore and of its name, or none for no
/// ore.
type Kind = Option<(u64, ItemKey)>;

pub fn tick(batch: &mut SystemBatch<Core>) {
    let world = batch.world();
    let mut kinds: HashMap<u32, Kind> = HashMap::new();
    for (id, core, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&CORE_DRILL) {
            continue;
        }
        core.rod = rod::place(&world, id, pos, core.rod);
        count(&world, pos, core, &mut kinds);
        if core.is_done() && core.reported == 0 && world.entity(core.driller).is_some() {
            report(&world, core.driller, core);
            core.reported = 1;
        }
    }
}

/// Counts what the drill at `pos` has reached and not counted yet, `BUDGET` at most, looking
/// each kind of block up in `kinds` once a tick.
fn count(world: &World, pos: Pos, core: &mut Core, kinds: &mut HashMap<u32, Kind>) {
    let reached = survey::reached(core.turns);
    if core.counted >= reached {
        crate::loading::release_machine(world, pos);
        return;
    }
    let goal = reached.min(core.counted + BUDGET);
    while core.counted < goal {
        let cell = survey::cell(pos, core.counted);
        let Some(block_state) = world.block_loaded_typed(cell) else {
            wait(world, pos, core);
            return;
        };
        let kind = *kinds
            .entry(block_state.id())
            .or_insert_with(|| kind_of(block_state));
        if let Some((kind, shown)) = kind {
            core.tally(kind, shown, survey::band(core.counted));
        }
        core.counted += 1;
        core.waited = 0;
    }
    let (first, last) = survey::layer_of(pos, core.counted.min(TOTAL - 1));
    crate::loading::keep_machine(world, pos, Some(Cubes::spanning(first, last)));
}

/// Waits for the layer the drill at `pos` counts to load, keeping it loaded; after
/// `FLOOR_WAIT` the count ends there.
fn wait(world: &World, pos: Pos, core: &mut Core) {
    core.waited += 1;
    if core.waited < FLOOR_WAIT {
        let (first, last) = survey::layer_of(pos, core.counted);
        crate::loading::keep_machine(world, pos, Some(Cubes::spanning(first, last)));
        return;
    }
    core.counted = TOTAL;
    crate::loading::release_machine(world, pos);
}

/// What `block_state` is to the count: its kind of ore and its name, by their keys.
fn kind_of(block_state: BlockState) -> Kind {
    let ore = crate::rubble::ore_name(block_state)?;
    Some((key_of_name(ore), key_of_name(block_state.block_name())))
}
