//! Every clarifier's tick: hand its clay to the tubes and machines on its sides and its clear
//! water out of its front, settle its muddy water for the ticks the call stands for (`settle`),
//! then show it: water in the tank while it holds any, and the rake turning while it settles.

use weft_sdk::{Dir, SystemBatch, World};

use super::settle::Basin;
use super::{CLARIFIER, Settle, Status, rake};
use crate::lib::{Pos, TypedWorld};
use crate::liquid::HELD;

/// Millibuckets of clear water it lets out a tick at most.
const CLEAR_OUT: u32 = 100;
static CLAY_BALL: weft_sdk::Item = weft_sdk::Item::named("base:clay_ball");

pub fn tick(batch: &mut SystemBatch<Settle>) {
    let world = batch.world();
    for (id, settle, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&CLARIFIER) {
            continue;
        }
        let front = crate::machine::front(&world, pos);
        let sides: Vec<Dir> = Dir::ALL.into_iter().filter(|&side| side != front).collect();
        crate::store::eject_toward(&world, pos, &sides);
        let most = settle.clear.min(CLEAR_OUT.saturating_mul(ticks));
        let water = crate::liquid::key_named("base:water");
        settle.clear -= crate::liquid::pour(&world, pos, front, water, most);
        let status = work(&world, id, settle, ticks);
        settle.set_status(status);
        show_water(&world, pos, muddy(&world, id) > 0 || settle.clear > 0);
        rake::show(&world, id, pos, settle);
    }
}

/// Settles the muddy water of the clarifier `id` for `ticks`, a clay ball into its output when a
/// bucket gives one.
fn work(world: &World, id: u64, settle: &mut Settle, ticks: u32) -> Status {
    let output = || world.inventory(id).group("output");
    let clay = CLAY_BALL.stack(1);
    let clay_room = crate::store::insert_into(output(), clay, true).is_empty();
    let mut basin = Basin {
        muddy: muddy(world, id),
        clear: settle.clear,
    };
    let had = basin.muddy;
    let (balls, status) = settle.run(ticks, &mut basin, clay_room, || crate::chance::roll(world));
    if balls > 0 {
        crate::store::insert_into(output(), clay, false);
    }
    // Only when it settled some: taking liquid out wakes the holder.
    if basin.muddy < had
        && let Some(held) = world.component_mut(id, &HELD)
    {
        held.remove(had - basin.muddy);
    }
    settle.clear = basin.clear;
    status
}

/// Millibuckets of muddy water the clarifier `id` holds.
fn muddy(world: &World, id: u64) -> u32 {
    let held = world.component(id, &HELD).unwrap_or_default();
    if held.liquid == crate::muddy_water::key() {
        held.amount
    } else {
        0
    }
}

/// Shows water in the tank of the clarifier at `pos` while it holds any.
fn show_water(world: &World, pos: Pos, filled: bool) {
    let value = if filled { "true" } else { "false" };
    if world.prop(pos, "water").is_some_and(|now| now != value) {
        world.prop_set(pos, "water", value);
    }
}
