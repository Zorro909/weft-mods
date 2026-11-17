//! A leaching vat's tick: hand on what it leached by its front and draw acid from the holders
//! beside it, then, with a leach's acid and room for what it gives, put energy into the first Ore
//! Rubble of its input until that is paid for and leached. The Ore Rubble leaves the input, and
//! the acid the vat, only once the leach is done and its Pure Ore Rubble is in the output; Ore
//! Rubble of another grade goes on to the output as it is. One step a call: missed ticks leach
//! nothing.

use weft_sdk::{Dir, Output, Particle, StackData, SystemBatch, Vec3, World};

use super::{ACID_HELD, LEACHING_VAT, Leach, Status};
use crate::lib::{Pos, TypedWorld};
use crate::machine::{self, Payment};
use crate::rubble::raise::{pass_on, waiting};
use crate::rubble::{Grade, ore_rubble};

/// Millibuckets of acid one leach takes.
pub const ACID_A_LEACH: u32 = 100;
/// The most energy it puts into leaching a tick, and the energy one Ore Rubble takes: 40 ticks.
const DRAW: i64 = 50;
const LEACH_COST: i64 = 2_000;
/// Millibuckets of acid it draws in a tick at most.
const ACID_IN: u32 = 50;
/// Ticks between bubbles, and between the sounds of it, while it leaches.
const BUBBLE_EVERY: u64 = 8;
const SOUND_EVERY: u64 = 40;
/// How far above its floor the acid stands, in blocks.
const SURFACE: f64 = 10.0 / 16.0;

pub fn tick(batch: &mut SystemBatch<Leach>) {
    let world = batch.world();
    for (id, leach, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&LEACHING_VAT) {
            continue;
        }
        machine::start_meter(&world, id);
        let front = machine::front(&world, pos);
        crate::store::eject_toward(&world, pos, &[front]);
        draw_acid(&world, pos, front, leach, ticks);
        let status = work(&world, id, leach);
        leach.set_status(status);
        show_acid(&world, pos, leach.acid > 0);
        if status == Status::Leaching {
            bubble(&world, pos);
        }
    }
}

/// The grade leaching Ore Rubble of `grade` gives: Rich leaches into Pure, and any other passes
/// through, Poor and Common to be washed and floated first, Pure as leached already.
pub fn leached(grade: Grade) -> Option<Grade> {
    (grade == Grade::Rich).then_some(Grade::Pure)
}

/// What keeps a vat from leaching with `acid` millibuckets in it, `room` saying whether what it
/// leaches fits in its output; none when it may leach.
fn lacks(acid: u32, room: bool) -> Option<Status> {
    if acid < ACID_A_LEACH {
        Some(Status::NoAcid)
    } else if !room {
        Some(Status::Full)
    } else {
        None
    }
}

/// Draws acid into the vat at `pos` from the holders on every side but its `front`, as much as it
/// has room for and `ACID_IN` a tick for `ticks`.
fn draw_acid(world: &World, pos: Pos, front: Dir, leach: &mut Leach, ticks: u32) {
    let mut most = ACID_IN.saturating_mul(ticks);
    for side in Dir::ALL.into_iter().filter(|&side| side != front) {
        let room = ACID_HELD.saturating_sub(leach.acid);
        let drawn = crate::liquid::draw(world, pos, side, crate::acid::key(), room.min(most));
        leach.acid += drawn;
        most -= drawn;
    }
}

/// Leaches the first Ore Rubble in the input of the vat `id` once it has its acid and has paid
/// for it, or passes it on when its grade is not leached.
fn work(world: &World, id: u64, leach: &mut Leach) -> Status {
    let inventory = world.inventory(id);
    let input = inventory.group("input");
    let Some((slot, ore, grade)) = waiting(&input) else {
        return Status::Idle;
    };
    let output = || inventory.group("output");
    let Some(grade) = leached(grade) else {
        return if pass_on(&input, slot, output()) {
            Status::Idle
        } else {
            Status::Full
        };
    };
    let product = ore_rubble(ore, grade, 1);
    let room = crate::store::insert_into(output(), product, true).is_empty();
    if let Some(lack) = lacks(leach.acid, room) {
        return lack;
    }
    match machine::pay(world, id, &mut leach.spent, LEACH_COST, DRAW) {
        Payment::Paid => {}
        Payment::Paying => return Status::Leaching,
        Payment::NoPower => return Status::NoPower,
    }
    crate::store::insert_into(output(), product, false);
    input.set(slot, input.get(slot).shrink(1));
    leach.acid -= ACID_A_LEACH;
    leach.spent = 0;
    Status::Leaching
}

/// Shows acid in the vat at `pos` while it holds some.
fn show_acid(world: &World, pos: Pos, filled: bool) {
    let value = if filled { "true" } else { "false" };
    if world.prop(pos, "acid").is_some_and(|now| now != value) {
        world.prop_set(pos, "acid", value);
    }
}

/// Bubbles rising off the acid of the vat at `pos` now and then, and the fizz of it.
fn bubble(world: &World, pos: Pos) {
    static BUBBLE: Particle = Particle::named("core:bubble");
    let now = world.game_time();
    if now.is_multiple_of(BUBBLE_EVERY) {
        let surface = pos.floor_middle() + Vec3::new(0.0, SURFACE, 0.0);
        let burst = world.spawn_particles(&BUBBLE, surface).count(3);
        drop(burst.spread(0.25).velocity(Vec3::new(0.0, 0.08, 0.0)));
    }
    if now.is_multiple_of(SOUND_EVERY) {
        world.play_sound(&crate::sounds::ACID_FIZZ, pos.center(), 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_ore_rubble_takes_forty_ticks_at_full_power() {
        assert_eq!(LEACH_COST / DRAW, 40, "2,000 at 50 a tick");
        const {
            assert!(
                ACID_IN * 40 >= ACID_A_LEACH,
                "its acid keeps up with a leach"
            )
        };
    }

    #[test]
    fn only_rich_ore_rubble_is_leached() {
        assert_eq!(leached(Grade::Rich), Some(Grade::Pure), "rich to pure");
        for grade in [Grade::Poor, Grade::Common, Grade::Pure] {
            assert_eq!(leached(grade), None, "{grade:?} passes through");
        }
    }

    #[test]
    fn it_says_what_it_lacks_first() {
        assert_eq!(lacks(0, false), Some(Status::NoAcid), "acid first");
        assert_eq!(
            lacks(ACID_A_LEACH - 1, true),
            Some(Status::NoAcid),
            "short of a leach"
        );
        assert_eq!(lacks(ACID_A_LEACH, false), Some(Status::Full), "then room");
        assert_eq!(
            lacks(ACID_A_LEACH, true),
            None,
            "one leach's acid is enough"
        );
    }

    #[test]
    fn it_holds_twenty_leaches_of_acid() {
        assert_eq!(ACID_HELD / ACID_A_LEACH, 20, "two buckets");
    }
}
