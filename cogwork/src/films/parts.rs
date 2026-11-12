//! Pieces the film's builds share: base's blocks they are made of, engines, lines of tubes and
//! shafts, chests and what goes into them.

use showcase_api::{Area, Set};
use weft_sdk::{Block, Dir, Item, ItemStack};

use crate::engines::EMBER_ENGINE;
use crate::lib::{BlockState, Pos};
use crate::liquid::{Holders, Vessel, key_named};
use crate::liquid_tube::LIQUID_TUBE;
use crate::shaft::TORQUE_SHAFT;
use crate::tube::STONE_TUBE;

pub static CHEST: Block = Block::named("base:chest");
pub static OAK_LOG: Block = Block::named("base:oak_log");
pub static STONE_BRICKS: Block = Block::named("base:stone_bricks");
pub static COBBLESTONE: Block = Block::named("base:cobblestone");
pub static GLOWSTONE: Block = Block::named("base:glowstone");
pub static GRAVEL: Block = Block::named("base:gravel");
pub static GRASS: Block = Block::named("base:grass");

/// An ember engine at `at` whose trunk points `toward` (the base plate is on the other side).
pub fn engine(set: &Set, at: Pos, toward: Dir) {
    let facing = toward.opposite().name();
    set.place(at, EMBER_ENGINE.with(&[("facing", facing)]).into());
}

/// The box between two corners, both included, in the showcase API's terms.
pub fn area(a: Pos, b: Pos) -> Area {
    Area::between(a, b)
}

/// A stack of coal into the engine at `at`, which starts it.
pub fn fuel(set: &Set, at: Pos) {
    let world = set.world();
    if let Some(coal) = Item::lookup("base:coal") {
        let engine = world.inventory(set.at(at));
        engine.group("fuel").set(0, ItemStack::new(coal, 64));
    }
}

/// `stacks` full stacks of each item into the chest or machine at `at`.
pub fn stock(set: &Set, at: Pos, items: &[&str], stacks: u32) {
    let world = set.world();
    let chest = world.inventory(set.at(at));
    for item in items.iter().filter_map(|name| Item::lookup(name)) {
        for _ in 0..stacks {
            chest.insert(ItemStack::new(item, 64));
        }
    }
}

/// Stone tubes on every block from `from` to `to`, both included, along one axis.
pub fn tubes(set: &Set, from: Pos, to: Pos) {
    set.fill(area(from, to), BlockState::of(&STONE_TUBE));
}

/// Torque shafts on every block from `from` to `to`, both included, along one axis.
pub fn shafts(set: &Set, from: Pos, to: Pos) {
    set.fill(area(from, to), BlockState::of(&TORQUE_SHAFT));
}

/// A liquid tube on each of `blocks`.
pub fn liquid_tubes(set: &Set, blocks: &[Pos]) {
    for &block in blocks {
        set.place(block, BlockState::of(&LIQUID_TUBE));
    }
}

/// Puts `liquid` into each of `blocks` that holds liquid, as if it had been pumped there
/// before the film: a liquid tube full, a tank up to `tank_full[0]` parts in `tank_full[1]` of
/// what it holds, whether its blocks have joined into one tank by now or not.
pub fn prefill(set: &Set, blocks: impl Iterator<Item = Pos>, liquid: &str, tank_full: [u32; 2]) {
    let world = set.world();
    let liquid = key_named(liquid);
    let mut holders = Holders::outside(world);
    for block in blocks {
        let Some(vessel) = Vessel::at(&world, set.at(block)) else {
            continue;
        };
        let wanted = match vessel {
            Vessel::Tank(_) => vessel.capacity() * tank_full[0] / tank_full[1],
            Vessel::One { .. } => vessel.capacity(),
        };
        let missing = wanted.saturating_sub(vessel.held(&holders).amount);
        vessel.put(&mut holders, liquid, missing);
    }
}
