//! The liquid tube: a glass tube holding a bucket of one liquid, joined to the liquid tubes,
//! tanks and pumps beside it. The liquid in it is drawn as it stands: a fill block (`fill`) in
//! its cell, level through its hub and sideways arms. Its shape is the Blockbench model
//! `assets/models/block/liquid_tube/hub_and_arm.json`, with an arm toward each side it joins
//! (`arms`). The timber liquid tube (`timber_liquid_tube`) is built the same way.

use std::sync::LazyLock;

use weft_sdk::{Block, BlockBuilder, Dir, Registry, World, tool_tier};

use crate::arms::{Arm, HubAndArms};
use crate::fill::{self, TUBE_LEVELS};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid::{HELD, Held, is_full, is_vessel, reaches};
use crate::timber_liquid_tube::TIMBER_LIQUID_TUBE;

pub static LIQUID_TUBE: Block = Block::named("liquid_tube");

/// Millibuckets one tube holds, of either kind.
pub const CAPACITY: u32 = 1000;

pub fn register(r: &mut Registry) {
    tube(r.block(&LIQUID_TUBE), &crate::arms::LIQUID_TUBE)
        .hardness(0.6)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.glass")
        .tab(&crate::TAB)
        .entity(|entity| entity.attach(&HELD));
}

/// Whether `block` is a liquid tube of either kind.
pub fn is_liquid_tube(block: BlockState) -> bool {
    block.is(&LIQUID_TUBE) || block.is(&TIMBER_LIQUID_TUBE)
}

/// A liquid tube drawn as `model`, with an arm toward every holder of liquid it joins, showing the
/// liquid its block entity holds: the caller gives that entity `HELD`.
pub fn tube<'a>(
    mut builder: BlockBuilder<'a>,
    model: &'static LazyLock<HubAndArms>,
) -> BlockBuilder<'a> {
    for dir in Dir::ALL {
        builder = builder.prop(dir.name(), ["false", "true"]);
    }
    builder
        .states(move |props, state| {
            // Every arm joins a holder of liquid, which the liquid goes on into.
            let arm = |dir: Dir| props.is(dir.name(), "true").then_some(Arm::Open);
            let pieces = model.pieces(arm, |name| model.texture(name).to_string());
            crate::boxes::draw(state.texture(model.texture("particle")), &pieces)
        })
        .on_place(|world, change| refresh(world, change.pos.into()))
        .on_neighbor(|event| refresh(&event.world(), event.pos().into()))
        .on_break(|world, change| fill::show_in_tube(world, change.pos.into(), None))
}

/// Sets every arm of the tube at `pos` to what its neighbors are now, and wakes it: what it
/// touches may have changed.
fn refresh(world: &World, pos: Pos) {
    let tube = world.block_typed(pos);
    if !is_liquid_tube(tube) {
        return;
    }
    let next = Dir::ALL.into_iter().fold(tube, |next, dir| {
        next.with_flag(dir.name(), joins(world, pos, dir))
    });
    if next != tube {
        world.set_block_typed(pos, next);
    }
    if let Some(held) = world.component_mut(world.block_entity(pos), &HELD) {
        held.asleep = 0;
    }
}

/// Whether the tube at `pos` reaches toward `dir`: to any holder of liquid that reaches back.
fn joins(world: &World, pos: Pos, dir: Dir) -> bool {
    let at = pos.toward(dir);
    is_vessel(world, at)
        && (is_liquid_tube(world.block_typed(at)) || reaches(world, at, dir.opposite()))
}

/// Shows how full the tube at `pos` is, when that or its arms changed since `held.shown`.
pub fn show(world: &World, pos: Pos, held: &mut Held) {
    let tube = world.block_typed(pos);
    let has_arm = |dir: Dir| tube.prop(dir.name()) == Some("true");
    let level = if is_full(held.amount, CAPACITY) {
        TUBE_LEVELS
    } else {
        fill::level_of(held.amount, CAPACITY, TUBE_LEVELS)
    };
    let shown = shown_as(level, has_arm);
    if shown == held.shown {
        return;
    }
    held.shown = shown;
    let filled = (level > 0)
        .then(|| fill::in_tube(held.liquid, level, has_arm))
        .flatten();
    fill::show_in_tube(world, pos, filled);
}

/// What a fill of `level` with `has_arm` is kept as in `Held::shown`: 0 for none.
/// Bits a fill's level takes in `Held::shown`, below those of its arms.
const LEVEL_BITS: u32 = TUBE_LEVELS.ilog2() + 1;

fn shown_as(level: u32, has_arm: impl Fn(Dir) -> bool) -> u32 {
    if level == 0 {
        return 0;
    }
    let arms = Dir::ALL
        .into_iter()
        .filter(|&dir| has_arm(dir))
        .fold(0, |arms, dir| arms | 1 << dir.face());
    level | arms << LEVEL_BITS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fill_is_shown_anew_when_its_level_or_arms_change() {
        let none = shown_as(0, |_| true);
        let low = shown_as(1, |_| false);
        let low_with_arm = shown_as(1, |dir| dir == Dir::Up);
        assert_eq!(none, 0, "nothing shown");
        assert_ne!(low, none, "a level");
        assert_ne!(low_with_arm, low, "an arm more");
        assert_ne!(shown_as(2, |_| false), low, "a level more");
    }
}
