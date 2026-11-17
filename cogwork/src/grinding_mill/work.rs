//! A grinding mill's tick: hand on the raw ore by its front, then, with water, sand and room
//! for what it gives, put energy into the first Ore Rubble of its input until that is paid for
//! and ground, the runner turning meanwhile.

use weft_sdk::{ItemStack, SlotGroup, StackData, SystemBatch, World};

use super::supply::Supplies;
use super::{GRINDING_MILL, Grinder, Status, grinds_with, runner};
use crate::lib::{Pos, TypedWorld};
use crate::liquid::HELD;
use crate::machine::{self, Payment};
use crate::rubble::grinding::{self, WATER_PER_GRIND};
use crate::rubble::{Grade, ore_of};

/// The most energy it puts into grinding a tick, and the energy one Ore Rubble takes: a second's
/// grinding at full power.
const DRAW: i64 = 40;
const GRIND_COST: i64 = 800;

pub fn tick(batch: &mut SystemBatch<Grinder>) {
    let world = batch.world();
    for (id, grinder, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&GRINDING_MILL) {
            continue;
        }
        machine::start_meter(&world, id);
        let front = machine::front(&world, pos);
        crate::store::eject_toward(&world, pos, &[front]);
        let status = grind(&world, id, grinder);
        grinder.set_status(status);
        runner::show(&world, id, pos, grinder);
    }
}

/// The first Ore Rubble waiting in `input`: its slot, the raw ore item it grinds into (0 when no
/// loaded mod has one) and its grade.
fn waiting(input: &SlotGroup) -> Option<(u32, u32, Grade)> {
    (0..input.len()).find_map(|slot| {
        let (ore, grade) = ore_of(&input.get(slot))?;
        Some((slot, ore.raw_ore().unwrap_or(0), grade))
    })
}

/// Grinds the first Ore Rubble in the input of the grinding mill `id` once it has what that
/// takes and has paid for it.
fn grind(world: &World, id: u64, grinder: &mut Grinder) -> Status {
    let inventory = world.inventory(id);
    let input = inventory.group("input");
    let Some((slot, raw_ore, grade)) = waiting(&input) else {
        return Status::Idle;
    };
    take_sand(&inventory.group("sand"), grinder);
    let output = || inventory.group("output");
    let most = ItemStack::new(raw_ore, grade.most_raw_ore());
    let supplies = Supplies {
        water: water(world, id),
        raw_ore: raw_ore != 0,
        room: raw_ore != 0 && crate::store::insert_into(output(), most, true).is_empty(),
    };
    if let Some(lack) = grinder.lacks(supplies) {
        return lack;
    }
    match machine::pay(world, id, &mut grinder.spent, GRIND_COST, DRAW) {
        Payment::Paid => {}
        Payment::Paying => return Status::Grinding,
        Payment::NoPower => return Status::NoPower,
    }
    let count = grinding::raw_ore(grade, crate::chance::roll(world));
    crate::store::insert_into(output(), ItemStack::new(raw_ore, count), false);
    input.set(slot, input.get(slot).shrink(1));
    use_water(world, id);
    grinder.ground();
    Status::Grinding
}

/// Takes a sand out of the slot `sand` into the grinds `grinder` has, once it has none left.
fn take_sand(sand: &SlotGroup, grinder: &mut Grinder) {
    if grinder.top_up(!sand.get(0).is_empty()) {
        sand.set(0, sand.get(0).shrink(1));
    }
}

/// Uses up a grind's water in the grinding mill `id`.
fn use_water(world: &World, id: u64) {
    if let Some(held) = world.component_mut(id, &HELD) {
        held.remove(WATER_PER_GRIND);
    }
}

/// Millibuckets of water the grinding mill `id` holds.
fn water(world: &World, id: u64) -> u32 {
    let held = world.component(id, &HELD).unwrap_or_default();
    if grinds_with(held.liquid) {
        held.amount
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ore_rubble_takes_twenty_ticks_at_full_power() {
        assert_eq!(GRIND_COST / DRAW, 20, "a second at 40 a tick");
    }
}
