//! Every thumper's tick: keep its weight in place, then take its survey a step on. While the
//! survey is paid for, the weight drops every `THUMP_EVERY` ticks; then it counts up to `BUDGET`
//! blocks a tick, keeping the layer it counts loaded meanwhile; then it listens for oil fields
//! (`oil`). A block that stays unloaded for `FLOOR_WAIT` is taken for the floor of the world: the
//! count ends there. Energy goes in one tick's worth a call, so missed ticks pay for nothing.

use std::collections::HashMap;

use weft_sdk::{SystemBatch, World};

use super::{SEISMIC_THUMPER, Status, Survey, oil, weight};
use crate::core_drill::{OreKind, ore_kind};
use crate::lib::{Pos, TypedWorld};
use crate::loading::Cubes;
use crate::machine::{self, Payment};

/// The energy a survey takes, and the most that goes into it a tick: 100 ticks.
pub(super) const SURVEY_COST: i64 = 50_000;
const DRAW: i64 = 500;
/// Ticks between drops of the weight while a survey is paid for: as long as its clip.
const THUMP_EVERY: u64 = 40;
/// The most blocks a thumper counts a tick: a 64 by 64 layer takes two ticks.
const BUDGET: u32 = 2_048;
/// Ticks a block may stay unloaded before the count takes it for the floor of the world.
const FLOOR_WAIT: u32 = 200;

pub fn tick(batch: &mut SystemBatch<Survey>) {
    let world = batch.world();
    let mut kinds: HashMap<u32, OreKind> = HashMap::new();
    for (id, survey, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&SEISMIC_THUMPER) {
            continue;
        }
        machine::start_meter(&world, id);
        let front = machine::front(&world, pos);
        survey.weight = weight::place(&world, id, pos, front, survey.weight);
        match survey.status() {
            Status::Thumping | Status::NoPower => thump(&world, id, pos, survey),
            Status::Counting => count(&world, pos, survey, &mut kinds),
            Status::Listening => oil::listen(&world, pos, survey),
            Status::Idle | Status::Done => {}
        }
    }
}

/// Puts energy into the survey of the thumper `id` at `pos`, the weight dropping now and then,
/// and starts counting once it is paid for.
fn thump(world: &World, id: u64, pos: Pos, survey: &mut Survey) {
    let status = match machine::pay(world, id, &mut survey.spent, SURVEY_COST, DRAW) {
        Payment::Paid => Status::Counting,
        Payment::Paying => Status::Thumping,
        Payment::NoPower => Status::NoPower,
    };
    if status == Status::Thumping && world.game_time().is_multiple_of(THUMP_EVERY) {
        weight::drop_on(world, survey.weight, pos);
    }
    survey.set_status(status);
}

/// Counts what lies below the thumper at `pos` and is not counted yet, `BUDGET` at most, looking
/// each kind of block up in `kinds` once a tick; once all is counted it lets go of the world and
/// listens for oil.
fn count(world: &World, pos: Pos, survey: &mut Survey, kinds: &mut HashMap<u32, OreKind>) {
    let total = survey.total();
    let goal = total.min(survey.counted + BUDGET);
    while survey.counted < goal {
        let cell = survey.cell(survey.counted);
        let Some(block_state) = world.block_loaded_typed(cell) else {
            wait(world, pos, survey);
            return;
        };
        let kind = *kinds
            .entry(block_state.id())
            .or_insert_with(|| ore_kind(block_state));
        if let Some((kind, shown)) = kind {
            let band = survey.band(survey.counted);
            survey.tally(kind, shown, band);
        }
        survey.counted += 1;
        survey.waited = 0;
    }
    if survey.counted >= total {
        crate::loading::release_machine(world, pos);
        survey.set_status(Status::Listening);
        return;
    }
    let (first, last) = survey.layer_of(survey.counted);
    crate::loading::keep_machine(world, pos, Some(Cubes::spanning(first, last)));
}

/// Waits for the layer the thumper at `pos` counts to load, keeping it loaded; after
/// `FLOOR_WAIT` the count ends there.
fn wait(world: &World, pos: Pos, survey: &mut Survey) {
    survey.waited += 1;
    if survey.waited < FLOOR_WAIT {
        let (first, last) = survey.layer_of(survey.counted);
        crate::loading::keep_machine(world, pos, Some(Cubes::spanning(first, last)));
        return;
    }
    survey.counted = survey.total();
    survey.waited = 0;
    crate::loading::release_machine(world, pos);
    survey.set_status(Status::Listening);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_survey_takes_a_hundred_ticks_at_full_power() {
        assert_eq!(SURVEY_COST / DRAW, 100, "50,000 at 500 a tick");
        assert!(
            super::super::STORES >= DRAW * THUMP_EVERY as i64 / 2,
            "a full store keeps it thumping between drops"
        );
    }

    #[test]
    fn a_widest_layer_takes_two_ticks() {
        let side = super::super::survey::LARGEST_SIDE as u32;
        assert_eq!(side * side / BUDGET, 2, "64 by 64");
    }
}
