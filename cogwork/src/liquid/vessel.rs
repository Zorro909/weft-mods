//! The blocks that hold liquid, and the tick that passes it between them. Every holder takes a
//! turn each tick: toward each side it reaches, where a holder reaches back, it hands over what
//! evens out how full the two are. A merged tank is one holder however many blocks it has.
//!
//! A system works on copies of its values, written back when it returns, so a holder changed
//! during the tick goes through `Holders`, which changes the copy when there is one.

use std::collections::HashMap;

use weft_sdk::{Dir, SystemBatch, World};

use super::{Held, Level, LiquidKey, share};
use crate::lib::{Pos, TypedWorld};
use crate::liquid_tube::LIQUID_TUBE;
use crate::pump::PUMP;
use crate::tank::{LIQUID_TANK, Tank};

/// The most millibuckets one holder hands one neighbour a turn.
const MOST_A_TURN: u32 = 100;

/// The `Held` of every holder, read and written through this tick's copies where they are.
pub struct Holders<'a> {
    world: World,
    index: HashMap<u64, usize>,
    values: &'a mut [Held],
}

impl<'a> Holders<'a> {
    /// The holders of a system's batch, the others reached through the world.
    fn of_batch(world: World, ids: &[u64], values: &'a mut [Held]) -> Holders<'a> {
        let index = ids.iter().copied().zip(0..).collect();
        Holders {
            world,
            index,
            values,
        }
    }

    /// Every holder through the world: outside a system's tick.
    pub fn outside(world: World) -> Holders<'static> {
        Holders {
            world,
            index: HashMap::new(),
            values: &mut [],
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// What the holder `id` keeps now.
    pub fn get(&self, id: u64) -> Held {
        match self.index.get(&id) {
            Some(&i) => self.values[i],
            None => self.world.component(id, &super::HELD).unwrap_or_default(),
        }
    }

    /// Makes the holder `id` keep `held`.
    pub fn set(&mut self, id: u64, held: Held) {
        match self.index.get(&id) {
            Some(&i) => self.values[i] = held,
            None => {
                if let Some(stored) = self.world.component_mut(id, &super::HELD) {
                    *stored = held;
                }
            }
        }
    }
}

/// A holder of liquid: one block, or a tank of several.
pub enum Vessel {
    /// A liquid tube or a pump: its block entity.
    One {
        id: u64,
        capacity: u32,
    },
    Tank(Tank),
}

impl Vessel {
    /// The holder whose block is at `pos`, if one is.
    pub fn at(world: &World, pos: Pos) -> Option<Vessel> {
        let block = world.block_typed(pos);
        let capacity = if block.is(&LIQUID_TUBE) {
            crate::liquid_tube::CAPACITY
        } else if block.is(&PUMP) {
            crate::pump::CAPACITY
        } else if block.is(&LIQUID_TANK) {
            return Tank::at(world, pos).map(Vessel::Tank);
        } else {
            return None;
        };
        let id = world.block_entity(pos);
        (id != 0).then_some(Vessel::One { id, capacity })
    }

    /// Its liquid and how much of it, all its blocks together.
    pub fn held(&self, holders: &Holders) -> Held {
        match self {
            Vessel::One { id, .. } => holders.get(*id),
            Vessel::Tank(tank) => tank.held(holders),
        }
    }

    pub fn capacity(&self) -> u32 {
        match self {
            Vessel::One { capacity, .. } => *capacity,
            Vessel::Tank(tank) => tank.capacity(),
        }
    }

    /// Puts `amount` of `liquid` in, which its `held().takes(liquid, capacity())` allowed.
    pub fn put(&self, holders: &mut Holders, liquid: LiquidKey, amount: u32) {
        let mut held = self.held(holders);
        held.add(liquid, amount);
        self.store(holders, held);
    }

    /// Takes `amount` out, as much of it as there is.
    pub fn take(&self, holders: &mut Holders, amount: u32) {
        let mut held = self.held(holders);
        held.remove(amount);
        self.store(holders, held);
    }

    fn store(&self, holders: &mut Holders, held: Held) {
        match self {
            Vessel::One { id, .. } => holders.set(*id, held),
            Vessel::Tank(tank) => tank.settle(holders, held),
        }
    }
}

/// Whether the block at `pos` holds liquid.
pub fn is_vessel(world: &World, pos: Pos) -> bool {
    let block = world.block_typed(pos);
    block.is(&LIQUID_TUBE) || block.is(&PUMP) || block.is(&LIQUID_TANK)
}

/// Whether the holder at `pos` passes liquid toward `dir`: a liquid tube along its arms, a pump
/// on every side but its hose's, a tank on every side it has no more of itself.
pub fn reaches(world: &World, pos: Pos, dir: Dir) -> bool {
    let block = world.block_typed(pos);
    if block.is(&PUMP) {
        dir != Dir::Down
    } else if block.is(&LIQUID_TUBE) {
        block.prop(dir.name()) == Some("true")
    } else {
        block.is(&LIQUID_TANK) && block.prop(dir.name()) == Some("false")
    }
}

/// Every holder's turn: liquid toward its neighbours, then a liquid tube shows its fill.
pub fn tick(batch: &mut SystemBatch<Held>) {
    let world = batch.world();
    let ids = batch.ids().to_vec();
    let mut holders = Holders::of_batch(world, &ids, batch.values());
    for &id in &ids {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        pass_on(&mut holders, pos);
        if world.block_typed(pos).is(&LIQUID_TUBE) {
            let mut held = holders.get(id);
            crate::liquid_tube::show(&world, pos, &mut held);
            holders.set(id, held);
        }
    }
}

/// Hands liquid from the holder at `pos` to each holder it touches. Most blocks of a big tank
/// touch none, and are done without finding the rest of their tank.
fn pass_on(holders: &mut Holders, pos: Pos) {
    let world = *holders.world();
    let touching =
        |&dir: &Dir| reaches(&world, pos, dir) && reaches(&world, pos.toward(dir), dir.opposite());
    let sides: Vec<Dir> = Dir::ALL.into_iter().filter(touching).collect();
    if sides.is_empty() {
        return;
    }
    let Some(from) = Vessel::at(&world, pos) else {
        return;
    };
    for dir in sides {
        if let Some(to) = Vessel::at(&world, pos.toward(dir)) {
            pass(holders, &from, &to);
        }
    }
}

/// Hands from `from` to `to` what evens out how full the two are.
fn pass(holders: &mut Holders, from: &Vessel, to: &Vessel) {
    let (giving, taking) = (from.held(holders), to.held(holders));
    if giving.is_empty() || !taking.takes(giving.liquid, to.capacity()) {
        return;
    }
    // A dribble of another liquid counts for nothing: it goes when this comes in.
    let had = if taking.liquid == giving.liquid {
        taking.amount
    } else {
        0
    };
    let amount = share(
        Level {
            amount: giving.amount,
            capacity: from.capacity(),
        },
        Level {
            amount: had,
            capacity: to.capacity(),
        },
        MOST_A_TURN,
    );
    if amount > 0 {
        from.take(holders, amount);
        to.put(holders, giving.liquid, amount);
    }
}
