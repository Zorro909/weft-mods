//! Pieces the film's builds share: engines, tube lines, chests and what goes into them.

use showcase_api::Set;
use weft_sdk::*;

use crate::block_state::BlockState;
use crate::engines::{EMBER_ENGINE, SPRING_ENGINE};
use crate::pos::Pos;
use crate::tube::STONE_TUBE;

pub static CHEST: Block = Block::named("base:chest");

/// An ember engine at `at` whose trunk points `toward` (the base plate is on the other side).
pub fn engine(set: &Set, at: Pos, toward: Dir) {
    set.place(
        at.into(),
        EMBER_ENGINE.with(&[("facing", toward.opposite().name())]),
    );
}

/// A spring engine at `at`, which runs while it has redstone power.
pub fn spring(set: &Set, at: Pos, toward: Dir) {
    set.place(
        at.into(),
        SPRING_ENGINE.with(&[("facing", toward.opposite().name())]),
    );
}

/// A stack of coal into the engine at `at`, which starts it.
pub fn fuel(set: &Set, at: Pos) {
    let world = set.world();
    if let Some(coal) = Item::lookup("base:coal") {
        let engine = world.inventory(world.block_entity(set.at(at.into())));
        engine.group("fuel").set(0, ItemStack::new(coal, 64));
    }
}

/// `stacks` full stacks of each item into the chest or machine at `at`.
pub fn stock(set: &Set, at: Pos, items: &[&str], stacks: u32) {
    let world = set.world();
    let chest = world.inventory(world.block_entity(set.at(at.into())));
    for item in items.iter().filter_map(|name| Item::lookup(name)) {
        for _ in 0..stacks {
            chest.insert(ItemStack::new(item, 64));
        }
    }
}

/// Stone tubes on every block from `from` to `to`, both included, along one axis.
pub fn line(set: &Set, from: Pos, to: Pos) {
    line_of(set, from, to, BlockState::of(&STONE_TUBE));
}

/// `block_state` on every block from `from` to `to`, both included, along one axis.
pub fn line_of(set: &Set, from: Pos, to: Pos, block_state: BlockState) {
    set.fill(from.into(), to.into(), block_state.id());
}
