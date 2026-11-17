//! Every trip hammer's tick: hand on its ingots by its front, take a step on its first bloom
//! (`strike`), paying a strike's steam as it strikes and putting the ingot out and taking the
//! bloom in with the last, then show it: the bloom glowing on the anvil and the hammer falling.

use weft_sdk::{StackData, SystemBatch, World};

use super::strike::{STEAM_A_STRIKE, Step};
use super::{Hammering, TRIP_HAMMER, hammer};
use crate::lib::{Pos, TypedWorld};
use crate::liquid::HELD;
use crate::steel::{STEEL_BLOOM, STEEL_INGOT};

pub fn tick(batch: &mut SystemBatch<Hammering>) {
    let world = batch.world();
    for (id, hammering, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&TRIP_HAMMER) {
            continue;
        }
        let front = crate::machine::front(&world, pos);
        crate::store::eject_toward(&world, pos, &[front]);
        let bloom = bloom_slot(&world, id);
        let room = crate::store::insert_into(output(&world, id), STEEL_INGOT.stack(1), true);
        let step = hammering.step(bloom.is_some(), steam(&world, id), room.is_empty());
        if matches!(step, Step::Struck | Step::Forged) {
            use_steam(&world, id);
        }
        if let (Step::Forged, Some(slot)) = (step, bloom) {
            forge(&world, id, slot);
        }
        show_bloom(&world, pos, bloom.is_some());
        hammer::show(&world, id, pos, front, hammering, step);
    }
}

/// The first slot of the input of the trip hammer `id` that holds a steel bloom.
fn bloom_slot(world: &World, id: u64) -> Option<u32> {
    let input = world.inventory(id).group("input");
    (0..input.len()).find(|&slot| input.get(slot).item == STEEL_BLOOM.id())
}

fn output(world: &World, id: u64) -> weft_sdk::SlotGroup<'static> {
    world.inventory(id).group("output")
}

/// Puts the ingot the trip hammer `id` forged into its output, then takes its bloom out of
/// `slot` of the input.
fn forge(world: &World, id: u64, slot: u32) {
    crate::store::insert_into(output(world, id), STEEL_INGOT.stack(1), false);
    let input = world.inventory(id).group("input");
    input.set(slot, input.get(slot).shrink(1));
}

/// Millibuckets of steam the trip hammer `id` holds.
fn steam(world: &World, id: u64) -> u32 {
    let held = world.component(id, &HELD).unwrap_or_default();
    if held.liquid == crate::steam::key() {
        held.amount
    } else {
        0
    }
}

/// Uses up a strike's steam in the trip hammer `id`.
fn use_steam(world: &World, id: u64) {
    if let Some(held) = world.component_mut(id, &HELD) {
        held.remove(STEAM_A_STRIKE);
    }
}

/// Shows a hot bloom on the anvil of the trip hammer at `pos` while one waits in its input.
fn show_bloom(world: &World, pos: Pos, bloom: bool) {
    let value = if bloom { "true" } else { "false" };
    if world.prop(pos, "bloom").is_some_and(|now| now != value) {
        world.prop_set(pos, "bloom", value);
    }
}
