//! An acid tank's tick: draw water from the holders beside it and let its acid out of its front,
//! then, with a sulfur, a batch's water and room for its acid, put energy into the batch until it
//! is paid for. The sulfur leaves the input and the water the tank, and the acid comes in, only
//! once the batch is done. One step a call: missed ticks make nothing, though the liquid they
//! would have moved still moves.

use weft_sdk::{Dir, Output, StackData, SystemBatch, Tag, World};

use super::brew::{Tanks, WATER_HELD};
use super::{ACID_TANK, Brew, Status};
use crate::lib::{Pos, TypedWorld};
use crate::machine::{self, Payment};

static SULFUR: Tag = Tag::named("dusts/sulfur");

/// The most energy it puts into a batch a tick, and the energy a batch takes: 50 ticks.
const DRAW: i64 = 40;
const BATCH_COST: i64 = 2_000;
/// Millibuckets of water it draws in, and of acid it lets out, a tick at most.
const WATER_IN: u32 = 100;
const ACID_OUT: u32 = 100;
/// Ticks between the sounds of it while it works.
const SOUND_EVERY: u64 = 40;

pub fn tick(batch: &mut SystemBatch<Brew>) {
    let world = batch.world();
    for (id, brew, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&ACID_TANK) {
            continue;
        }
        machine::start_meter(&world, id);
        let front = machine::front(&world, pos);
        draw_water(&world, pos, front, brew, ticks);
        let_out_acid(&world, pos, front, brew, ticks);
        let status = work(&world, id, brew);
        brew.set_status(status);
        if status == Status::Brewing && world.game_time().is_multiple_of(SOUND_EVERY) {
            world.play_sound(&crate::sounds::ACID_FIZZ, pos.center(), 0.8);
        }
    }
}

/// Whether `item` is sulfur, which its input takes.
pub fn is_sulfur(item: u32) -> bool {
    SULFUR.on_item(item)
}

/// Draws water into the tank at `pos` from the holders on every side but its `front`, as much
/// as it has room for and `WATER_IN` a tick for `ticks`.
fn draw_water(world: &World, pos: Pos, front: Dir, brew: &mut Brew, ticks: u32) {
    let water = crate::liquid::key_named("base:water");
    let mut most = WATER_IN.saturating_mul(ticks);
    for side in Dir::ALL.into_iter().filter(|&side| side != front) {
        let room = WATER_HELD.saturating_sub(brew.water);
        let drawn = crate::liquid::draw(world, pos, side, water, room.min(most));
        brew.water += drawn;
        most -= drawn;
    }
}

/// Lets the acid of the tank at `pos` out of its `front`, `ACID_OUT` a tick for `ticks`.
fn let_out_acid(world: &World, pos: Pos, front: Dir, brew: &mut Brew, ticks: u32) {
    let most = brew.acid.min(ACID_OUT.saturating_mul(ticks));
    let acid = crate::acid::key();
    brew.acid -= crate::liquid::pour(world, pos, front, acid, most);
}

/// Works on the batch of the tank `id` once it has what that takes, and finishes it once it is
/// paid for.
fn work(world: &World, id: u64, brew: &mut Brew) -> Status {
    let input = world.inventory(id).group("input");
    let sulfur = (0..input.len()).find(|&slot| is_sulfur(input.get(slot).item));
    let tanks = Tanks {
        water: brew.water,
        acid: brew.acid,
    };
    if let Some(lack) = tanks.lacks(sulfur.is_some()) {
        return lack;
    }
    match machine::pay(world, id, &mut brew.spent, BATCH_COST, DRAW) {
        Payment::Paid => {}
        Payment::Paying => return Status::Brewing,
        Payment::NoPower => return Status::NoPower,
    }
    if let Some(slot) = sulfur {
        input.set(slot, input.get(slot).shrink(1));
    }
    let after = tanks.brewed();
    (brew.water, brew.acid) = (after.water, after.acid);
    brew.spent = 0;
    Status::Brewing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_batch_takes_fifty_ticks_at_full_power() {
        assert_eq!(BATCH_COST / DRAW, 50, "2,000 at 40 a tick");
    }

    #[test]
    fn its_pipes_keep_up_with_a_batch() {
        let ticks = (BATCH_COST / DRAW) as u32;
        assert!(
            WATER_IN * ticks >= super::super::brew::WATER_A_BATCH,
            "water in"
        );
        assert!(
            ACID_OUT * ticks >= super::super::brew::ACID_A_BATCH,
            "acid out"
        );
    }
}
