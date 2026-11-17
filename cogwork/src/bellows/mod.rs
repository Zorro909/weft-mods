//! Bellows: a leather bag between two boards whose nozzle points at the block it was placed
//! against. While one blows into a furnace, the furnace cooks twice as fast. The hand bellows
//! blows for a minute after each pump by hand (`hand`); the driven bellows blows while an engine
//! keeps it fed (`driven`). They are separate machines: neither does the other's job.
//!
//! The speed-up is a step on base's `SMELT_TIME` chain. Base's furnace asks the chain with its
//! own entity in the query, so the step finds the furnace's block and looks for a bellows
//! beside it pointing at it. Any mod's furnace that asks the chain the same way is blown too.
//!
//! The shapes are the Blockbench models `assets/models/block/<bellows>.json`; the bag and the
//! boards that move are `assets/models/<bellows>_bag.geo.json`.

mod driven;
mod hand;

pub use driven::BLAST_DRAW;
use weft_sdk::{
    Block, BlockBuilder, Facings, Modify, Output, Particle, Registry, Toward, Vec3, World, phase,
    tool_tier,
};

use crate::lib::{Pos, TypedWorld};

pub static HAND_BELLOWS: Block = Block::named("hand_bellows");
pub static DRIVEN_BELLOWS: Block = Block::named("driven_bellows");
static PUFF: Particle = Particle::named("bellows_puff");

/// How many times as fast a furnace cooks while a bellows blows into it.
const SPEED_UP: f64 = 2.0;
/// Blocks above a furnace's floor its puffs come out at: just over its top.
const PUFF_HEIGHT: f64 = 1.05;

pub fn register(r: &mut Registry) {
    hand::register(r);
    driven::register(r);
    r.modify(&base_api::chains::SMELT_TIME, &phase::MULTIPLY, speed_up);
    r.particle(&PUFF)
        .textures(&["core:generic_2", "core:generic_3", "core:generic_4"])
        .life(0.6, 1.0)
        .size(0.12, 0.2)
        .gravity(-0.3)
        .drag(0.9)
        .color(0xC8C0B880, 0xE0D8D000)
        .translucent();
}

/// A bellows' block, shaped by the model `name`: its nozzle points the way the player looked
/// when placing it, into the block they placed it against.
fn block<'a>(builder: BlockBuilder<'a>, name: &'static str) -> BlockBuilder<'a> {
    builder
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Away)
        .model_file(name)
        .hardness(2.0)
        .mined_by("axe", tool_tier::HAND)
        .sound_group("base:block.wood")
        .tab(&crate::TAB)
}

/// The `SMELT_TIME` step: a furnace a bellows blows into cooks in half the time.
fn speed_up(step: &mut Modify) {
    let world = step.world();
    let Some(furnace) = world.entity(step.query().entity) else {
        return;
    };
    if blows_into(&world, Pos::from(furnace.block_pos())) {
        step.value /= SPEED_UP;
    }
}

/// Whether a bellows of either kind blows into the block at `target`.
pub fn blows_into(world: &World, target: Pos) -> bool {
    pumped_into(world, target) || driven_into(world, target)
}

/// Whether a hand bellows blows into the block at `target`.
fn pumped_into(world: &World, target: Pos) -> bool {
    pointing_at(world, target)
        .any(|(bellows, id)| bellows == Kind::Hand && hand::blowing(world, id))
}

/// Whether a driven bellows blows into the block at `target`: what a furnace that can't work
/// without a steady draft asks.
pub fn driven_into(world: &World, target: Pos) -> bool {
    pointing_at(world, target)
        .any(|(bellows, id)| bellows == Kind::Driven && driven::blowing(world, id))
}

/// Which bellows it is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Hand,
    Driven,
}

/// The bellows beside `target` whose nozzles point at it, with their block entities.
fn pointing_at(world: &World, target: Pos) -> impl Iterator<Item = (Kind, u64)> + '_ {
    Facings::HORIZONTAL.dirs().filter_map(move |side| {
        let at = target.toward(side);
        let block_state = world.block_typed(at);
        let kind = if block_state.is(&HAND_BELLOWS) {
            Kind::Hand
        } else if block_state.is(&DRIVEN_BELLOWS) {
            Kind::Driven
        } else {
            return None;
        };
        let points_back = crate::machine::front_of(block_state) == side.opposite();
        points_back.then(|| (kind, world.block_entity(at)))
    })
}

/// The block the nozzle of the bellows at `pos` points into.
fn target(world: &World, pos: Pos) -> Pos {
    pos.toward(crate::machine::front(world, pos))
}

/// Whether the block at `target` is a furnace, or anything else that smelts as base's does.
fn smelts(world: &World, target: Pos) -> bool {
    let id = world.block_entity(target);
    id != 0
        && world
            .component(id, &base_api::components::SMELTING_STATE)
            .is_some()
}

/// `count` faint puffs out of the top of the furnace at `target`, while it burns.
fn puff(world: &World, target: Pos, count: u32) {
    if world.prop(target, "lit") != Some("true") {
        return;
    }
    let top = target.floor_middle() + Vec3::new(0.0, PUFF_HEIGHT, 0.0);
    let burst = world.spawn_particles(&PUFF, top).count(count);
    drop(
        burst
            .spread(0.15)
            .velocity(Vec3::new(0.0, 0.5, 0.0))
            .speed(0.2),
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_bellows_have_their_models() {
        for name in ["hand_bellows", "driven_bellows"] {
            for file in [
                format!("models/block/{name}.json"),
                format!("models/item/{name}.json"),
            ] {
                assert!(crate::has_asset(&file), "no {file}");
            }
        }
        assert!(
            crate::has_asset("textures/block/bellows_leather.png"),
            "no leather"
        );
    }
}
