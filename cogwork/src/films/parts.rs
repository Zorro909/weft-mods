//! Pieces the film's builds share: engines, tube lines, chests and what goes into them.

use showcase_api::{Area, Set};
use weft_sdk::*;

use crate::engines::{EMBER_ENGINE, SPRING_ENGINE};
use crate::lib::{BlockState, Pos};
use crate::tube::STONE_TUBE;

pub static CHEST: Block = Block::named("base:chest");

/// An ember engine at `at` whose trunk points `toward` (the base plate is on the other side).
pub fn engine(set: &Set, at: Pos, toward: Dir) {
    let facing = toward.opposite().name();
    set.place(at, EMBER_ENGINE.with(&[("facing", facing)]).into());
}

/// A spring engine at `at`, which runs while it has redstone power.
pub fn spring(set: &Set, at: Pos, toward: Dir) {
    let facing = toward.opposite().name();
    set.place(at, SPRING_ENGINE.with(&[("facing", facing)]).into());
}

/// `offset` from the set's origin, in the world.
pub fn in_world(set: &Set, offset: Pos) -> Pos {
    set.at(offset)
}

/// The box between two corners, both included, in the showcase API's terms.
pub fn area(a: Pos, b: Pos) -> Area {
    Area::between(a, b)
}

/// A stack of coal into the engine at `at`, which starts it.
pub fn fuel(set: &Set, at: Pos) {
    let world = set.world();
    if let Some(coal) = Item::lookup("base:coal") {
        let engine = world.inventory(in_world(set, at));
        engine.group("fuel").set(0, ItemStack::new(coal, 64));
    }
}

/// `stacks` full stacks of each item into the chest or machine at `at`.
pub fn stock(set: &Set, at: Pos, items: &[&str], stacks: u32) {
    let world = set.world();
    let chest = world.inventory(in_world(set, at));
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
    set.fill(area(from, to), block_state);
}
