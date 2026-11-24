//! The water wheel: a wheel three blocks across on a wooden axle, turned by flowing water at its
//! foot. It makes energy while water flows (not a still source) through any of the three cells
//! its lower paddles dip into, stores a little, and pushes it out of its axle's end into the
//! block there, and to the shafts and cables beside it. More water makes no more: one wheel, one
//! share of the stream.
//!
//! The block is the axle and its bearing, the Blockbench model `models/block/water_wheel.json`;
//! the wheel is an owned entity turning on it (`wheel`).

mod flow;
mod wheel;

use flow::Speed;
use weft_sdk::{
    Block, Component, Dir, ENERGY, Energy, Facings, Registry, SystemBatch, Tag, Toward, World,
    component_flags, tool_tier, value,
};

use crate::lib::{Pos, TypedWorld};

pub static WATER_WHEEL: Block = Block::named("water_wheel");
static WATER: Tag = Tag::named("water");

/// Energy a tick it makes while water flows at its foot.
const MAKES: i64 = 12;
/// Energy it stores, and the most it offers to energy networks a tick.
const STORES: i64 = 1_000;
const OFFERS: i64 = 50;

value! {
    /// A water wheel's wheel entity.
    #[derive(Default)]
    pub struct Wheel {
        pub entity: u64,
    }
}
static WHEEL: Component<Wheel> = Component::named("cogwork:water_wheel");

pub fn register(r: &mut Registry) {
    r.entity_component(&WHEEL, Wheel::default(), component_flags::SAVE);
    r.system(&WHEEL, tick);
    r.block(&WATER_WHEEL)
        .grid_placement()
        .model_file("water_wheel")
        .facing(Facings::HORIZONTAL, Toward::Face)
        .item_texture("water_wheel")
        .hardness(1.5)
        .mined_by("axe", tool_tier::HAND)
        .sound_group("base:block.wood")
        .tab(&crate::TAB)
        .entity(|entity| {
            entity.attach(&WHEEL).attach(&wheel::SHOWN).attach_with(
                &ENERGY,
                Energy {
                    capacity: STORES,
                    max_out: OFFERS,
                    ..Energy::default()
                },
            )
        });
    wheel::register(r);
}

/// Every water wheel: makes energy while water flows at its foot, pushes it out of its axle, and
/// turns its wheel by how it runs.
fn tick(batch: &mut SystemBatch<Wheel>) {
    let world = batch.world();
    for (id, wheel, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        let block_state = world.block_typed(pos);
        if !block_state.is(&WATER_WHEEL) {
            continue;
        }
        let out = axle_end(block_state);
        let flowing = flow::lower_cells(pos, out)
            .into_iter()
            .any(|cell| flows(&world, cell));
        let made = if flowing { MAKES * i64::from(ticks) } else { 0 };
        let took = store(&world, id, made);
        crate::engines::push_into(&world, id, pos, out);
        wheel::show(&world, id, pos, out, wheel, Speed::of(flowing, took));
    }
}

/// The side the axle gives its energy out of: away from where the wheel was placed from, into
/// the block it was placed against.
fn axle_end(block_state: crate::lib::BlockState) -> Dir {
    crate::machine::front_of(block_state).opposite()
}

/// Whether flowing water, not a source, fills `cell`.
fn flows(world: &World, cell: Pos) -> bool {
    let block_state = world.block_typed(cell);
    block_state.has(&WATER)
        && crate::liquid::in_world(block_state).is_some_and(|(_, source)| !source)
}

/// Adds `made` to the store of the wheel `id`; true when any of it went in.
fn store(world: &World, id: u64, made: i64) -> bool {
    let Some(energy) = world.component_mut(id, &ENERGY) else {
        return false;
    };
    let before = energy.stored;
    energy.stored = (before + made).min(energy.capacity);
    energy.stored > before
}
