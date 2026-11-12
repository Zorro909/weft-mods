//! A pump's tick: bore down to a pool of liquid, leaving hose behind, then take a bucket of it
//! every `PUMP_EVERY` ticks. A puddle on the way, such as a seep's, it bores on through. Once
//! pumping, it lowers its hose after the liquid as the pool sinks, but bores no more: a pool
//! pumped dry leaves it dry.

use weft_sdk::{Dir, Output, SystemBatch, World};

use super::drain::survey;
use super::{CAPACITY, PUMP, PUMP_HOSE, Pump, Stage, Status};
use crate::harvest::{Harvest, cost, harvest};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid::{self, BUCKET, HELD};
use crate::machine::{self, Payment};
use crate::sounds;

/// Energy to bore through a block (a base and as much again per point of hardness: stone 250),
/// and to lower the hose through air or liquid.
const BORE_BASE: f32 = 100.0;
const BORE_PER_HARDNESS: f32 = 100.0;
const HOSE_COST: i64 = 20;
/// Energy to take a bucket of liquid, and the ticks between two buckets at most.
const BUCKET_COST: i64 = 2500;
const PUMP_EVERY: u32 = 20;
/// Sources it needs to reach to stop boring and pump: fewer are a puddle.
const POOL: usize = 8;
/// The most energy it puts into its work per tick.
const DRAW_RATE: i64 = 200;
/// Ticks between its boring sounds at most: it may bore through a block a tick.
const SOUND_EVERY: u64 = 8;

pub fn tick(batch: &mut SystemBatch<Pump>) {
    let world = batch.world();
    for (id, pump, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&PUMP) {
            continue;
        }
        crate::store::eject(&world, pos);
        match pump.stage() {
            Stage::Bore => bore(&world, id, pos, pump),
            Stage::Pump => pump_next(&world, id, pos, pump),
        }
        machine::show_working(&world, pos, pump.status().is_working());
    }
}

/// The block just below the end of the hose of the pump at `pos`.
fn below_hose(pos: Pos, pump: &Pump) -> Pos {
    pos.toward_by(Dir::Down, pump.depth as i32 + 1)
}

/// Bores one block further down, or starts pumping on reaching a pool.
fn bore(world: &World, id: u64, pos: Pos, pump: &mut Pump) {
    let next = below_hose(pos, pump);
    if let Some((liquid, _)) = liquid::at(world, next) {
        let pool = survey(next, liquid, |at| liquid::at(world, at));
        if pool.sources >= POOL {
            pump.set_stage(Stage::Pump);
            pump.set_status(Status::Pumping);
            return;
        }
    }
    lower(world, id, pump, next, true);
}

/// Lowers the hose of the pump `id` into `next`, through air or liquid, and through what can be
/// broken when `may_break`; paid for out of its energy. The status says what stopped it.
fn lower(world: &World, id: u64, pump: &mut Pump, next: Pos, may_break: bool) {
    let block = world.block_typed(next);
    if block.is(&PUMP_HOSE) {
        // Its own hose, left from before.
        pump.depth += 1;
        return;
    }
    let open = block.can_build_into();
    let cost = if open {
        HOSE_COST
    } else if may_break && block.can_break() {
        cost(block, BORE_BASE, BORE_PER_HARDNESS)
    } else {
        let stuck = if may_break {
            Status::Blocked
        } else {
            Status::Dry
        };
        pump.set_status(stuck);
        return;
    };
    // Following a sinking pool down is part of pumping it.
    pump.set_status(if may_break {
        Status::Boring
    } else {
        Status::Pumping
    });
    if !pay(world, id, pump, cost) {
        return;
    }
    if !open && !bore_through(world, id, pump, next, block) {
        return;
    }
    world.set_block_typed(next, BlockState::of(&PUMP_HOSE));
    if !world.block_typed(next).is(&PUMP_HOSE) {
        // Below the floor of the world, or someone wouldn't let it.
        pump.set_status(Status::Blocked);
        return;
    }
    job_done(pump, cost);
    pump.depth += 1;
    if world.game_time().is_multiple_of(SOUND_EVERY) {
        world.play_sound(&sounds::PUMP_BORE, next.center(), 1.0);
    }
}

/// Breaks `block` at `next` into the pump's output; false when it couldn't.
fn bore_through(world: &World, id: u64, pump: &mut Pump, next: Pos, block: BlockState) -> bool {
    match harvest(world, id, next, block) {
        Harvest::Done => true,
        Harvest::Full => {
            // Paid for already: it stays paid until there is room.
            pump.set_status(Status::Full);
            false
        }
        Harvest::Refused => {
            pump.set_status(Status::Blocked);
            false
        }
    }
}

/// Takes the next bucket of the liquid below the hose, or follows it down as it sinks.
fn pump_next(world: &World, id: u64, pos: Pos, pump: &mut Pump) {
    let intake = below_hose(pos, pump);
    let Some((liquid, _)) = liquid::at(world, intake) else {
        lower(world, id, pump, intake, false);
        return;
    };
    pump.wait = pump.wait.saturating_sub(1);
    if pump.wait > 0 {
        return;
    }
    let held = world.component(id, &HELD).unwrap_or_default();
    if !held.takes(liquid, CAPACITY) || held.amount + BUCKET > CAPACITY {
        pump.set_status(Status::Full);
        return;
    }
    pump.set_status(Status::Pumping);
    if !pay(world, id, pump, BUCKET_COST) {
        return;
    }
    let Some(source) = survey(intake, liquid, |at| liquid::at(world, at)).farthest else {
        pump.set_status(Status::Dry);
        return;
    };
    world.set_block_typed(source, BlockState::AIR);
    if let Some(stored) = world.component_mut(id, &HELD) {
        stored.add(liquid, BUCKET);
    }
    job_done(pump, BUCKET_COST);
    pump.wait = PUMP_EVERY;
    world.play_sound(&sounds::PUMP_DRAW, pos.center(), 1.0);
}

/// A job of `cost` is done: what was paid beyond it goes to the next.
fn job_done(pump: &mut Pump, cost: i64) {
    pump.spent = (pump.spent - cost).max(0);
}

/// Puts this tick's energy into a job of `cost`; true once it is paid.
fn pay(world: &World, id: u64, pump: &mut Pump, cost: i64) -> bool {
    match machine::pay(world, id, &mut pump.spent, cost, DRAW_RATE) {
        Payment::Paid => true,
        Payment::Paying => false,
        Payment::NoPower => {
            pump.set_status(Status::NoPower);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn energy_paid_for_one_job_goes_to_the_next() {
        // Most of a bucket paid, when the liquid flows away and the hose follows it instead.
        let mut pump = Pump {
            spent: 800,
            ..Pump::default()
        };
        job_done(&mut pump, HOSE_COST);
        assert_eq!(pump.spent, 800 - HOSE_COST, "the rest kept");
    }
}
