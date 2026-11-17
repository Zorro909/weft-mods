//! A flotation cell's tick: hand on what it floated by its front, draw creosote from the holders
//! beside it and let its muddy water out of its back, then, with all a float takes there and room
//! for all it leaves, put energy into the first Ore Rubble of its input until that is paid for and
//! floated, the rotor stirring meanwhile. The Ore Rubble leaves the input, and the water and
//! creosote the cell, only once the float is done and its Rich Ore Rubble is in the output; Ore
//! Rubble of another grade goes on to the output as it is.

use weft_sdk::{Dir, StackData, SystemBatch, World};

use super::cell::{Tanks, floated};
use super::{FLOTATION_CELL, Float, Status, rotor};
use crate::lib::{Pos, TypedWorld};
use crate::liquid::{HELD, LiquidKey, key_named};
use crate::machine::{self, Payment};
use crate::rubble::ore_rubble;
use crate::rubble::raise::{pass_on, waiting};

/// The most energy it puts into floating a tick, and the energy one Ore Rubble takes.
const DRAW: i64 = 40;
const FLOAT_COST: i64 = 1_200;
/// Millibuckets of creosote it draws in, and of muddy water it lets out, a tick at most.
const CREOSOTE_IN: u32 = 50;
const MUDDY_OUT: u32 = 100;

pub fn tick(batch: &mut SystemBatch<Float>) {
    let world = batch.world();
    for (id, float, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&FLOTATION_CELL) {
            continue;
        }
        machine::start_meter(&world, id);
        let front = machine::front(&world, pos);
        crate::store::eject_toward(&world, pos, &[front]);
        draw_creosote(&world, pos, front.opposite(), float, ticks);
        let_out_muddy(&world, pos, front.opposite(), float, ticks);
        let status = work(&world, id, float);
        float.set_status(status);
        show_froth(&world, pos, status == Status::Floating);
        rotor::show(&world, id, pos, front, float);
    }
}

/// The key of creosote, as holders keep liquids.
fn creosote() -> LiquidKey {
    key_named("cogwork:creosote")
}

/// Draws creosote into the cell at `pos` from the holders on every side but its `back`, as much
/// as it has room for and `CREOSOTE_IN` a tick for `ticks`.
fn draw_creosote(world: &World, pos: Pos, back: Dir, float: &mut Float, ticks: u32) {
    let mut most = CREOSOTE_IN.saturating_mul(ticks);
    for side in Dir::ALL.into_iter().filter(|&side| side != back) {
        let room = super::cell::CREOSOTE_HELD.saturating_sub(float.creosote);
        let drawn = crate::liquid::draw(world, pos, side, creosote(), room.min(most));
        float.creosote += drawn;
        most -= drawn;
    }
}

/// Lets the muddy water of the cell at `pos` out of its `back`, `MUDDY_OUT` a tick for `ticks`.
fn let_out_muddy(world: &World, pos: Pos, back: Dir, float: &mut Float, ticks: u32) {
    let most = float.muddy.min(MUDDY_OUT.saturating_mul(ticks));
    let muddy = crate::muddy_water::key();
    float.muddy -= crate::liquid::pour(world, pos, back, muddy, most);
}

/// Floats the first Ore Rubble in the input of the cell `id` once it has all that takes and has
/// paid for it, or passes it on when its grade is not floated.
fn work(world: &World, id: u64, float: &mut Float) -> Status {
    let inventory = world.inventory(id);
    let input = inventory.group("input");
    let Some((slot, ore, grade)) = waiting(&input) else {
        return Status::Idle;
    };
    let output = || inventory.group("output");
    let Some(grade) = floated(grade) else {
        return if pass_on(&input, slot, output()) {
            Status::Idle
        } else {
            Status::Full
        };
    };
    let product = ore_rubble(ore, grade, 1);
    let room = crate::store::insert_into(output(), product, true).is_empty();
    let tanks = Tanks {
        water: water(world, id),
        creosote: float.creosote,
        muddy: float.muddy,
    };
    if let Some(lack) = tanks.lacks(room) {
        return lack;
    }
    match machine::pay(world, id, &mut float.spent, FLOAT_COST, DRAW) {
        Payment::Paid => {}
        Payment::Paying => return Status::Floating,
        Payment::NoPower => return Status::NoPower,
    }
    crate::store::insert_into(output(), product, false);
    input.set(slot, input.get(slot).shrink(1));
    let after = tanks.floated();
    if let Some(held) = world.component_mut(id, &HELD) {
        held.remove(tanks.water - after.water);
    }
    (float.creosote, float.muddy) = (after.creosote, after.muddy);
    float.spent = 0;
    Status::Floating
}

/// Millibuckets of water the cell `id` holds.
fn water(world: &World, id: u64) -> u32 {
    let held = world.component(id, &HELD).unwrap_or_default();
    if crate::liquid::is_water(held.liquid) {
        held.amount
    } else {
        0
    }
}

/// Shows froth on the cell at `pos` while it floats.
fn show_froth(world: &World, pos: Pos, frothing: bool) {
    let value = if frothing { "true" } else { "false" };
    if world.prop(pos, "froth").is_some_and(|now| now != value) {
        world.prop_set(pos, "froth", value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ore_rubble_takes_thirty_ticks_at_full_power() {
        assert_eq!(FLOAT_COST / DRAW, 30, "1,200 at 40 a tick");
    }

    #[test]
    fn its_pipes_keep_up_with_a_float_every_thirty_ticks() {
        let ticks = (FLOAT_COST / DRAW) as u32;
        assert!(
            CREOSOTE_IN * ticks >= super::super::cell::CREOSOTE_PER_FLOAT,
            "creosote in"
        );
        assert!(
            MUDDY_OUT * ticks >= super::super::cell::MUDDY_PER_FLOAT,
            "muddy water out"
        );
    }
}
