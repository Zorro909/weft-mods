//! Every firebox's tick: find the boiler on it, let that boiler's steam out of its top, boil it
//! for the ticks the call stands for, lighting fuel from the fuel slot or the lava it holds as it
//! needs (`burn`), then show it: the door glowing and a hiss from the boiler while it boils.

use base_api::FUEL;
use weft_sdk::{Dir, ItemStack, Output, REMAINDER, StackData, SystemBatch, World};

use super::boiler::BOIL;
use super::burn::Kettle;
use super::{BOILER, FIREBOX, Fire, Status};
use crate::lib::{Pos, TypedWorld};
use crate::liquid::{HELD, Holders, LiquidKey, Vessel, key_named};

/// Millibuckets of lava lit at a time, and the ticks of fuel each millibucket gives: a bucket
/// burns as long by liquid tube as it does in the fuel slot.
const LAVA_SIP: u32 = 10;
const LAVA_BURNS: u32 = 20;
/// The most steam a boiler lets out of its top a tick, in millibuckets.
const VENT: u32 = 200;
/// Ticks between hisses of a boiling boiler: about as long as one lasts.
const HISS_EVERY: u64 = 80;

pub fn tick(batch: &mut SystemBatch<Fire>) {
    let world = batch.world();
    for (id, fire, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&FIREBOX) {
            continue;
        }
        crate::store::eject_toward(&world, pos, &[crate::machine::front(&world, pos)]);
        let above = pos.toward(Dir::Up);
        let boiled = if world.block_typed(above).is(&BOILER) {
            let boiler = world.block_entity(above);
            vent(&world, boiler, above, ticks);
            boil(&world, id, boiler, fire, ticks)
        } else {
            fire.set_status(Status::NoBoiler);
            false
        };
        show(&world, pos, boiled);
    }
}

/// The key of lava, as holders keep liquids.
fn lava() -> LiquidKey {
    key_named("base:lava")
}

/// Boils the boiler `boiler` on the firebox `id` for `ticks`; whether it boiled at all.
fn boil(world: &World, id: u64, boiler: u64, fire: &mut Fire, ticks: u32) -> bool {
    let held = world.component(boiler, &HELD).unwrap_or_default();
    let water = if crate::liquid::is_water(held.liquid) {
        held.amount
    } else {
        0
    };
    let steam = world.component(boiler, &BOIL).unwrap_or_default().steam;
    let mut kettle = Kettle { water, steam };
    let (boiled, lack) = fire.boil(ticks, &mut kettle, || light(world, id));
    fire.set_status(match lack {
        _ if boiled > 0 => Status::Boiling,
        Some(lack) => lack,
        None => Status::Boiling,
    });
    if boiled == 0 {
        return false;
    }
    if let Some(held) = world.component_mut(boiler, &HELD) {
        held.remove(water - kettle.water);
    }
    if let Some(boil) = world.component_mut(boiler, &BOIL) {
        boil.steam = kettle.steam;
    }
    true
}

/// Lights the next fuel of the firebox `id`: an item from its fuel slot, else lava it holds. Its
/// ticks of burning, or why there is none.
fn light(world: &World, id: u64) -> Result<u32, Status> {
    let item = light_item(world, id);
    if item.is_ok() {
        return item;
    }
    light_lava(world, id).ok_or(item.err().unwrap_or(Status::NoFuel))
}

/// Burns one item of the fuel slot of the firebox `id`, putting what it leaves (a lava bucket's
/// bucket) in the output, and only when that has room.
fn light_item(world: &World, id: u64) -> Result<u32, Status> {
    let inventory = world.inventory(id);
    let slot = inventory.group("fuel");
    let stack = slot.get(0);
    let Some(ticks) = FUEL.on_item(stack.item).filter(|_| !stack.is_empty()) else {
        return Err(Status::NoFuel);
    };
    let left = match REMAINDER.on_item(stack.item) {
        Some(empty) => ItemStack::new(empty.0, 1),
        None => ItemStack::EMPTY,
    };
    if !crate::store::insert_all(inventory.group("output"), &[left]) {
        return Err(Status::NoRoom);
    }
    slot.set(0, stack.shrink(1));
    Ok(ticks)
}

/// Takes a sip of the lava the firebox `id` holds; its ticks of burning, none without enough.
fn light_lava(world: &World, id: u64) -> Option<u32> {
    let held = world.component_mut(id, &HELD)?;
    if held.liquid != lava() || held.amount < LAVA_SIP {
        return None;
    }
    held.remove(LAVA_SIP);
    Some(LAVA_SIP * LAVA_BURNS)
}

/// Lets the steam of the boiler `boiler` at `pos` out of its top, into a liquid tube or a steam
/// machine there that takes it, at most `VENT` a tick for `ticks`.
fn vent(world: &World, boiler: u64, pos: Pos, ticks: u32) {
    let steam = crate::steam::key();
    let Some(out) = Vessel::at(world, pos.toward(Dir::Up)) else {
        return;
    };
    let mut holders = Holders::outside(*world);
    let has = out.held(&holders);
    if !out.admits(steam) || !has.takes(steam, out.capacity()) {
        return;
    }
    let had = if has.liquid == steam { has.amount } else { 0 };
    let room = out.capacity().saturating_sub(had);
    let Some(boil) = world.component_mut(boiler, &BOIL) else {
        return;
    };
    let moved = boil.steam.min(room).min(VENT.saturating_mul(ticks));
    if moved > 0 {
        boil.steam -= moved;
        out.put(&mut holders, steam, moved);
    }
}

/// Shows the firebox at `pos` burning or not: its door glowing, and the boiler above hissing.
fn show(world: &World, pos: Pos, boiling: bool) {
    crate::machine::show_working(world, pos, boiling);
    if boiling && world.game_time().is_multiple_of(HISS_EVERY) {
        let boiler = pos.toward(Dir::Up).center();
        world.play_sound(&crate::sounds::BOILER_HISS, boiler, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bucket_of_lava_burns_as_long_piped_as_carried() {
        let piped = crate::liquid::BUCKET * LAVA_BURNS;
        assert_eq!(piped, 20_000, "base's lava bucket burns 20,000 ticks");
        assert_eq!(
            piped / super::super::burn::BURN_A_TICK,
            10_000,
            "10 ticks of boiling a millibucket"
        );
    }
}
