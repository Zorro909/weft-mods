//! The liquid tube: a glass tube holding a bucket of one liquid, joined to the liquid tubes,
//! tanks and pumps beside it. The liquid in it is drawn as it stands: a fill block (`fill`) in
//! its cell, level through its hub and sideways arms. Its shape is the Blockbench model of its
//! set of arms, `assets/models/block/liquid_tube/<arms>.json`, written by `arms::files`.

use weft_sdk::*;

use crate::fill::{self, TUBE_LEVELS};
use crate::lib::{Pos, TypedWorld};
use crate::liquid::{Held, is_vessel, reaches};

pub static LIQUID_TUBE: Block = Block::named("liquid_tube");

/// Millibuckets one tube holds.
pub const CAPACITY: u32 = 1000;

pub fn register(r: &mut Registry) {
    let mut builder = r
        .block(&LIQUID_TUBE)
        .hardness(0.6)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.glass")
        .tab(&crate::TAB);
    for dir in Dir::ALL {
        builder = builder.prop(dir.name(), ["false", "true"]);
    }
    builder
        .states(|props, state| {
            let has_arm = |dir: Dir| props.is(dir.name(), "true");
            let model = &crate::arms::LIQUID_TUBE;
            let pieces = model.pieces(has_arm, |name| model.texture(name).to_string());
            crate::boxes::draw(state.texture(model.texture("particle")), &pieces)
        })
        .on_place(|world, change| refresh(world, change.pos.into()))
        .on_neighbor(|event| refresh(&event.world(), event.pos().into()))
        .on_break(|world, change| fill::show_in_tube(world, change.pos.into(), None))
        .entity(|entity| entity.attach(&crate::liquid::HELD));
}

/// Sets every arm of the tube at `pos` to what its neighbors are now.
fn refresh(world: &World, pos: Pos) {
    let tube = world.block_typed(pos);
    if !tube.is(&LIQUID_TUBE) {
        return;
    }
    let next = Dir::ALL.into_iter().fold(tube, |next, dir| {
        next.with_flag(dir.name(), joins(world, pos, dir))
    });
    if next != tube {
        world.set_block_typed(pos, next);
    }
}

/// Whether the tube at `pos` reaches toward `dir`: to any holder of liquid that reaches back.
fn joins(world: &World, pos: Pos, dir: Dir) -> bool {
    let at = pos.toward(dir);
    is_vessel(world, at)
        && (world.block_typed(at).is(&LIQUID_TUBE) || reaches(world, at, dir.opposite()))
}

/// Shows how full the tube at `pos` is, when that or its arms changed since `held.shown`.
pub fn show(world: &World, pos: Pos, held: &mut Held) {
    let tube = world.block_typed(pos);
    let has_arm = |dir: Dir| tube.prop(dir.name()) == Some("true");
    let level = fill::level_of(held.amount, CAPACITY, TUBE_LEVELS);
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
