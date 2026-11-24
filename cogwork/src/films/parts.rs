//! Pieces the film's builds share: base's blocks they are made of, engines, lines of tubes,
//! axles and shafts, quarry frames, chests and what goes into them.

use std::collections::HashSet;

use showcase_api::{Area, Set};
use weft_sdk::{Block, Dir, Item, ItemStack, StackData};

use crate::engines::{EMBER_ENGINE, ENGINE, SPRING_ENGINE};
use crate::excavator::{DIG, Frame, Stage, Status, bit, kind::Kind};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid::{Holders, Vessel, key_named};
use crate::liquid_tube::{LIQUID_TUBE, PRESSURE_TUBE, SEALED_LIQUID_TUBE};
use crate::rubble::{Grade, Ore, ore_rubble};
use crate::shaft::{TORQUE_SHAFT, WOODEN_AXLE};
use crate::tube::STONE_TUBE;

pub static CHEST: Block = Block::named("base:chest");
pub static FURNACE: Block = Block::named("base:furnace");
pub static OAK_LOG: Block = Block::named("base:oak_log");
pub static OAK_PLANKS: Block = Block::named("base:oak_planks");
pub static OAK_SLAB: Block = Block::named("base:oak_planks_slab");
pub static STONE_BRICKS: Block = Block::named("base:stone_bricks");
pub static COBBLESTONE: Block = Block::named("base:cobblestone");
pub static GLOWSTONE: Block = Block::named("base:glowstone");
pub static GRAVEL: Block = Block::named("base:gravel");
pub static GRASS: Block = Block::named("base:grass");
pub static WATER: Block = Block::named("base:water");

/// Day time set at the start of long sections so the sun stays near noon throughout the film.
const NOON: u64 = 6_000;

/// Resets the sun to noon so a multi-minute film never drifts into dusk.
pub fn noon(set: &Set) {
    set.world().set_day_time(NOON);
}

/// An ember engine at `at` whose trunk points `toward` (the base plate is on the other side).
pub fn engine(set: &Set, at: Pos, toward: Dir) {
    let facing = toward.opposite().name();
    set.place(at, EMBER_ENGINE.with(&[("facing", facing)]).into());
}

/// A spring engine at `at` whose trunk points `toward`.
pub fn spring_engine(set: &Set, at: Pos, toward: Dir) {
    let facing = toward.opposite().name();
    set.place(at, SPRING_ENGINE.with(&[("facing", facing)]).into());
}

/// Sets the stored spring energy of the spring engine at `at`.
pub fn wind_spring(set: &Set, at: Pos, energy: u32) {
    let world = set.world();
    let id = world.block_entity(set.at(at));
    if let Some(engine) = world.component_mut(id, &ENGINE) {
        engine.wound = energy;
    }
}

/// The box between two corners, both included, in the showcase API's terms.
pub fn area(a: Pos, b: Pos) -> Area {
    Area::between(a, b)
}

/// A stack of coal into the engine at `at`, which starts it.
pub fn fuel(set: &Set, at: Pos) {
    fuel_item(set, at, "base:coal");
}

/// A stack of `item` into the `fuel` group of the engine or firebox at `at`.
pub fn fuel_item(set: &Set, at: Pos, item: &str) {
    let world = set.world();
    if let Some(item) = Item::lookup(item) {
        let holder = world.inventory(set.at(at));
        holder.group("fuel").set(0, ItemStack::new(item, 64));
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

/// Sets `items` (name and count) into consecutive slots of `group` on the block at `at`.
pub fn stock_group(set: &Set, at: Pos, group: &str, items: &[(&str, u32)]) {
    let world = set.world();
    let slots = world.inventory(set.at(at)).group(group);
    for (slot, &(name, count)) in items.iter().enumerate() {
        if let Some(item) = Item::lookup(name) {
            slots.set(slot as u32, ItemStack::new(item, count));
        }
    }
}

/// Sets `stacks` full stacks of Ore Rubble of `metal` at `grade` into `group` on the machine at `at`.
pub fn stock_rubble(set: &Set, at: Pos, group: &str, metal: &str, grade: Grade, stacks: u32) {
    let Some(ore) = Ore::of(metal) else {
        return;
    };
    let world = set.world();
    let slots = world.inventory(set.at(at)).group(group);
    for slot in 0..stacks {
        slots.set(slot, ore_rubble(ore, grade, 64));
    }
}

/// Fits a drill bit named `item` with `wear` uses already spent into the quarry at `at`.
pub fn fit_bit(set: &Set, at: Pos, item: &str, wear: u32) {
    let Some(item) = Item::lookup(item) else {
        return;
    };
    let world = set.world();
    let stack = ItemStack::new(item, 1).add_wear(wear);
    world.inventory(set.at(at)).group(bit::GROUP).set(0, stack);
}

/// Places a pre-built quarry `frame` of `block`, joining pole-style frames toward their neighbors.
pub fn build_frame(set: &Set, frame: &Frame, block: &Block) {
    let world = set.world();
    let cells: Vec<Pos> = frame.cells();
    let poles = BlockState::of(block).prop(Dir::Up.name()).is_some();
    let set_cells: HashSet<Pos> = cells.iter().copied().collect();
    for cell in cells {
        let mut state = BlockState::of(block);
        if poles {
            for dir in Dir::ALL {
                state = state.with_flag(dir.name(), set_cells.contains(&cell.toward(dir)));
            }
        }
        world.set_block_typed(cell, state);
    }
}

/// Puts the quarry of `kind` at `at` (facing `front`) straight into mining its pre-built frame
/// from `start_y` (in set coordinates).
pub fn start_quarry(set: &Set, at: Pos, front: Dir, kind: &Kind, start_y: i32) {
    let world = set.world();
    let pos = set.at(at);
    let frame = Frame::plan(&world, pos, front, kind);
    let Some(dig) = world.component_mut(world.block_entity(pos), &DIG) else {
        return;
    };
    dig.frame = frame;
    dig.step = frame.cells().len() as u32;
    dig.set_stage(Stage::Mine);
    dig.set_status(Status::NoPower);
    dig.cursor = frame.first().with_y(set.at(Pos::new(0, start_y, 0)).y);
    dig.set_head(dig.cursor.floor_middle());
}

/// Stone tubes on every block from `from` to `to`, both included, along one axis.
pub fn tubes(set: &Set, from: Pos, to: Pos) {
    set.fill(area(from, to), BlockState::of(&STONE_TUBE));
}

/// Torque shafts on every block from `from` to `to`, both included, along one axis.
pub fn shafts(set: &Set, from: Pos, to: Pos) {
    set.fill(area(from, to), BlockState::of(&TORQUE_SHAFT));
}

/// Wooden axles on every block from `from` to `to`, both included, along one axis.
pub fn axles(set: &Set, from: Pos, to: Pos) {
    set.fill(area(from, to), BlockState::of(&WOODEN_AXLE));
}

/// A liquid tube on each of `blocks`.
pub fn liquid_tubes(set: &Set, blocks: &[Pos]) {
    for &block in blocks {
        set.place(block, BlockState::of(&LIQUID_TUBE));
    }
}

/// A sealed liquid tube on each of `blocks`.
pub fn sealed_tubes(set: &Set, blocks: &[Pos]) {
    for &block in blocks {
        set.place(block, BlockState::of(&SEALED_LIQUID_TUBE));
    }
}

/// A pressure tube on each of `blocks`.
pub fn pressure_tubes(set: &Set, blocks: &[Pos]) {
    for &block in blocks {
        set.place(block, BlockState::of(&PRESSURE_TUBE));
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
