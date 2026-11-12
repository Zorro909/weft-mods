//! The timber liquid tube: a liquid tube with a small wooden pump in its middle. Driven by an
//! engine, it pulls liquid out of the tanks it touches and pushes it up the liquid tubes beside
//! it as the pump does, its paddle wheel turning while liquid goes through (`wheel`). While the
//! wheel turns the tube shows itself full of what it pumps, however little stays in it, so the
//! wheel turns in liquid. Undriven it is a liquid tube like any other.
//!
//! Its shape is the Blockbench model `assets/models/block/timber_liquid_tube/hub_and_arm.json`,
//! drawn as the liquid tube's is (`liquid_tube::tube`): a wooden pump body a texel wider than the
//! tube on each side, windowed where it has no arm.

mod pull;
mod wheel;

use weft_sdk::{
    Block, Component, ENERGY, Registry, SystemBatch, World, component_flags, tool_tier, value,
};

use crate::lib::{Pos, TypedWorld};
use crate::liquid::HELD;

pub static TIMBER_LIQUID_TUBE: Block = Block::named("timber_liquid_tube");

/// Energy the tube stores, and the most it takes in a tick: an ember engine's push.
const ENERGY_STORED: i64 = 2_000;
const ENERGY_IN: i64 = 200;

value! {
    /// A timber liquid tube's pump: its wheel (an entity), `driven`, 1 while it has energy to
    /// pull with, the millibuckets it pulled this stroke and their `liquid`, and the liquid it
    /// is `pumping` while its wheel turns, through the stroke after one it pulled in
    /// (`NO_LIQUID` for none).
    #[derive(Default)]
    pub struct Drive {
        pub wheel: u64,
        pub driven: u32,
        pub pulled: u32,
        pub liquid: u64,
        pub pumping: u64,
    }
}
static DRIVE: Component<Drive> = Component::named("cogwork:timber_drive");

pub fn register(r: &mut Registry) {
    r.entity_component(&DRIVE, Drive::default(), component_flags::SAVE);
    r.system(&DRIVE, tick);
    crate::liquid_tube::tube(
        r.block(&TIMBER_LIQUID_TUBE),
        &crate::arms::TIMBER_LIQUID_TUBE,
    )
    .hardness(1.0)
    .mined_by("axe", tool_tier::HAND)
    .sound_group("base:block.wood")
    .tab(&crate::TAB)
    .entity(|entity| {
        entity
            .attach(&HELD)
            .attach(&DRIVE)
            .attach_with(&ENERGY, crate::machine::energy(ENERGY_STORED, ENERGY_IN))
    });
    wheel::register(r);
}

/// Whether the timber liquid tube whose block entity is `id` is driven: it pushes, and takes no
/// liquid but what it pulls.
pub fn is_driven(world: &World, id: u64) -> bool {
    world
        .component(id, &DRIVE)
        .is_some_and(|drive| drive.driven != 0)
}

/// Every timber liquid tube: driven while it has energy, it pulls what that pays for, its wheel
/// turns through a stroke after one it pulled in, and it shows its fill.
fn tick(batch: &mut SystemBatch<Drive>) {
    let world = batch.world();
    let time = world.game_time();
    for (id, drive, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&TIMBER_LIQUID_TUBE) {
            continue;
        }
        let energy = world.component(id, &ENERGY).map_or(0, |store| store.stored);
        let driven = u32::from(energy > 0);
        if driven != drive.driven {
            drive.driven = driven;
            wake(&world, id);
        }
        if driven != 0 {
            pull(&world, id, pos, energy, drive);
        }
        wheel::show(&world, id, pos, drive, time);
        if let Some(held) = world.component_mut(id, &HELD) {
            crate::liquid_tube::show(&world, pos, held, drive.pumping);
        }
    }
}

/// Pulls into the tube `id` at `pos` what `energy` pays for, and keeps what `drive` pulled.
fn pull(world: &World, id: u64, pos: Pos, energy: i64, drive: &mut Drive) {
    let pulled = pull::from_tanks(world, id, pos, energy);
    if pulled == 0 {
        return;
    }
    drive.pulled += pulled;
    if let Some(held) = world.component(id, &HELD) {
        drive.liquid = held.liquid;
    }
}

/// Wakes the liquid of the tube `id`: it now passes liquid on another way.
fn wake(world: &World, id: u64) {
    if let Some(held) = world.component_mut(id, &HELD) {
        held.asleep = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_a_drive_keeps_is_saved_as_laid_out() {
        assert_eq!(std::mem::offset_of!(Drive, wheel), 0, "its wheel");
        assert_eq!(std::mem::offset_of!(Drive, driven), 8, "whether driven");
        assert_eq!(
            std::mem::offset_of!(Drive, pulled),
            12,
            "pulled this stroke"
        );
        assert_eq!(std::mem::offset_of!(Drive, liquid), 16, "the liquid pulled");
        assert_eq!(
            std::mem::offset_of!(Drive, pumping),
            24,
            "the liquid pumped"
        );
    }
}
