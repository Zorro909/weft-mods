//! A washer's tick: hand on what it washed by its front, then, with water and room for what it
//! gives, put energy into the first Ore Rubble of its input until that is paid for and washed,
//! the paddle turning meanwhile. The Ore Rubble leaves the input, and the water the trough, only
//! once the wash is done and its Common Ore Rubble is in the output; Ore Rubble already washed
//! goes on to the output as it is.

use weft_sdk::{SlotGroup, StackData, SystemBatch, World};

use super::{Status, WASHER, Wash, paddle};
use crate::lib::{Pos, TypedWorld};
use crate::liquid::HELD;
use crate::machine::{self, Payment};
use crate::rubble::{Grade, Ore, ore_of, ore_rubble};

/// Millibuckets of water one wash takes.
pub const WATER_PER_WASH: u32 = 250;
/// The most energy it puts into washing a tick, and the energy one Ore Rubble takes.
const DRAW: i64 = 30;
const WASH_COST: i64 = 400;

pub fn tick(batch: &mut SystemBatch<Wash>) {
    let world = batch.world();
    for (id, wash, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&WASHER) {
            continue;
        }
        machine::start_meter(&world, id);
        let front = machine::front(&world, pos);
        crate::store::eject_toward(&world, pos, &[front]);
        let status = work(&world, id, wash);
        wash.set_status(status);
        show_water(&world, pos, water(&world, id) > 0);
        paddle::show(&world, id, pos, front, wash);
    }
}

/// The first Ore Rubble waiting in `input`: its slot, its metal and its grade.
fn waiting(input: &SlotGroup) -> Option<(u32, Ore, Grade)> {
    (0..input.len()).find_map(|slot| {
        let (ore, grade) = ore_of(&input.get(slot))?;
        Some((slot, ore, grade))
    })
}

/// The grade washing Ore Rubble of `grade` gives: Poor washes into Common, and any other is
/// washed already.
fn washed(grade: Grade) -> Option<Grade> {
    (grade == Grade::Poor).then_some(Grade::Common)
}

/// What keeps a washer from washing with `water` millibuckets in its trough, `room` saying
/// whether what it washes fits in its output; none when it may wash.
fn lacks(water: u32, room: bool) -> Option<Status> {
    if water < WATER_PER_WASH {
        Some(Status::NoWater)
    } else if !room {
        Some(Status::Full)
    } else {
        None
    }
}

/// Washes the first Ore Rubble in the input of the washer `id` once it has what that takes and
/// has paid for it, or passes it on when it is washed already.
fn work(world: &World, id: u64, wash: &mut Wash) -> Status {
    let inventory = world.inventory(id);
    let input = inventory.group("input");
    let Some((slot, ore, grade)) = waiting(&input) else {
        return Status::Idle;
    };
    let output = || inventory.group("output");
    let Some(grade) = washed(grade) else {
        return pass_on(&input, slot, output());
    };
    let product = ore_rubble(ore, grade, 1);
    let room = crate::store::insert_into(output(), product, true).is_empty();
    if let Some(lack) = lacks(water(world, id), room) {
        return lack;
    }
    match machine::pay(world, id, &mut wash.spent, WASH_COST, DRAW) {
        Payment::Paid => {}
        Payment::Paying => return Status::Washing,
        Payment::NoPower => return Status::NoPower,
    }
    crate::store::insert_into(output(), product, false);
    input.set(slot, input.get(slot).shrink(1));
    use_water(world, id);
    wash.spent = 0;
    Status::Washing
}

/// Moves the Ore Rubble in `slot` of `input`, washed already, into `output` as far as it fits.
fn pass_on(input: &SlotGroup, slot: u32, output: SlotGroup<'static>) -> Status {
    let stack = input.get(slot);
    let left = crate::store::insert_into(output, stack, false);
    input.set(slot, left);
    if left == stack {
        Status::Full
    } else {
        Status::Idle
    }
}

/// Uses up a wash's water in the washer `id`.
fn use_water(world: &World, id: u64) {
    if let Some(held) = world.component_mut(id, &HELD) {
        held.remove(WATER_PER_WASH);
    }
}

/// Millibuckets of water the washer `id` holds.
fn water(world: &World, id: u64) -> u32 {
    let held = world.component(id, &HELD).unwrap_or_default();
    if crate::liquid::is_water(held.liquid) {
        held.amount
    } else {
        0
    }
}

/// Shows water in the trough of the washer at `pos` while it holds some.
fn show_water(world: &World, pos: Pos, filled: bool) {
    let value = if filled { "true" } else { "false" };
    if world.prop(pos, "water").is_some_and(|now| now != value) {
        world.prop_set(pos, "water", value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ore_rubble_takes_fourteen_ticks_at_full_power() {
        let ticks = (WASH_COST + DRAW - 1) / DRAW;
        assert_eq!(ticks, 14, "400 at 30 a tick, the last tick short");
    }

    #[test]
    fn only_poor_ore_rubble_is_washed() {
        assert_eq!(washed(Grade::Poor), Some(Grade::Common), "poor to common");
        for grade in [Grade::Common, Grade::Rich, Grade::Pure] {
            assert_eq!(washed(grade), None, "{grade:?} passes through");
        }
    }

    #[test]
    fn it_says_what_it_lacks_first() {
        assert_eq!(lacks(0, false), Some(Status::NoWater), "water first");
        assert_eq!(
            lacks(WATER_PER_WASH - 1, true),
            Some(Status::NoWater),
            "short of a wash"
        );
        assert_eq!(
            lacks(WATER_PER_WASH, false),
            Some(Status::Full),
            "then room"
        );
        assert_eq!(
            lacks(WATER_PER_WASH, true),
            None,
            "one wash's water is enough"
        );
    }
}
