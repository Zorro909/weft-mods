//! The blocks that hold liquid, and the tick that passes it between them. A holder that is awake
//! takes a turn: it works out the push coming into it from its neighbours, hands liquid toward
//! each side it reaches, where a holder reaches back and stands lower (`flow`), and tells its
//! neighbours its head. When nothing changed it sleeps until a neighbour wakes it, by changing or
//! handing it liquid; so a settled line costs nearly nothing, and pressure spreads a block a tick
//! with no walk over the whole line. A merged tank is one holder however many blocks it has.
//!
//! A system works on copies of its values, written back when it returns, so a holder changed
//! during the tick goes through `Holders`, which changes the copy when there is one.

use std::collections::HashMap;

use weft_sdk::{Dir, SystemBatch, World};

use super::flow::{LIFT, LOSS, UNIT};
use super::{DRIBBLE, Held, Level, LiquidKey, MOST_A_TURN, flow, is_full};
use crate::lib::{Pos, TypedWorld};
use crate::liquid_tube::LIQUID_TUBE;
use crate::pump::PUMP;
use crate::tank::{LIQUID_TANK, Tank};

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
    /// A liquid tube or a pump: its block entity, where it is, how much it holds, and whether it
    /// is a pump, which pushes and never takes.
    One {
        id: u64,
        pos: Pos,
        capacity: u32,
        pump: bool,
    },
    Tank(Tank),
}

impl Vessel {
    /// The holder whose block is at `pos`, if one is.
    pub fn at(world: &World, pos: Pos) -> Option<Vessel> {
        let block = world.block_typed(pos);
        let (capacity, pump) = if block.is(&LIQUID_TUBE) {
            (crate::liquid_tube::CAPACITY, false)
        } else if block.is(&PUMP) {
            (crate::pump::CAPACITY, true)
        } else if block.is(&LIQUID_TANK) {
            return Tank::at(world, pos).map(Vessel::Tank);
        } else {
            return None;
        };
        let id = world.block_entity(pos);
        (id != 0).then_some(Vessel::One {
            id,
            pos,
            capacity,
            pump,
        })
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

    /// The head of `amount` in it, with `pushed` the push coming into it from beside: a pump
    /// holding liquid pushes `LIFT` blocks up; another holder takes on the push when it is full
    /// (`is_full`); otherwise its liquid stands where its surface is.
    fn head(&self, amount: u32, pushed: i64) -> i64 {
        let (floor, per_block) = match self {
            Vessel::One {
                pos, pump: true, ..
            } if amount >= DRIBBLE => return (pos.y as i64 + LIFT) * UNIT,
            Vessel::One { pos, capacity, .. } => (pos.y, *capacity),
            Vessel::Tank(tank) => (tank.floor(), tank.per_block()),
        };
        let surface = flow::surface(floor, amount, per_block);
        if is_full(amount, self.capacity()) {
            surface.max(pushed)
        } else {
            surface
        }
    }

    /// Millibuckets that raise its liquid a block.
    fn per_block(&self) -> u32 {
        match self {
            Vessel::One { capacity, .. } => *capacity,
            Vessel::Tank(tank) => tank.per_block(),
        }
    }

    /// The head it last told its neighbours.
    fn told(&self, holders: &Holders) -> i64 {
        match self {
            Vessel::One { id, .. } => holders.get(*id).head,
            Vessel::Tank(tank) => tank.ids().next().map_or(0, |id| holders.get(id).head),
        }
    }

    /// The head it stands at now, as its neighbours see it.
    fn head_now(&self, holders: &Holders) -> i64 {
        self.head(self.held(holders).amount, self.told(holders))
    }

    /// Tells its neighbours its head is `head`.
    fn tell(&self, holders: &mut Holders, head: i64) {
        self.each_block(holders, |held| held.head = head);
    }

    /// Wakes it, every block of it.
    fn wake(&self, holders: &mut Holders) {
        self.each_block(holders, |held| held.asleep = 0);
    }

    fn each_block(&self, holders: &mut Holders, change: impl Fn(&mut Held)) {
        let ids: Vec<u64> = match self {
            Vessel::One { id, .. } => vec![*id],
            Vessel::Tank(tank) => tank.ids().collect(),
        };
        for id in ids {
            let mut held = holders.get(id);
            change(&mut held);
            holders.set(id, held);
        }
    }

    /// Every side of it that touches another holder, by the block it is on.
    fn sides(&self, world: &World) -> Vec<(Pos, Dir)> {
        let blocks: Vec<Pos> = match self {
            Vessel::One { pos, .. } => vec![*pos],
            Vessel::Tank(tank) => tank.blocks().collect(),
        };
        let side = |&pos: &Pos| touching(world, pos).into_iter().map(move |dir| (pos, dir));
        blocks.iter().flat_map(side).collect()
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

/// The sides of the holder at `pos` where it and a holder beside reach each other.
fn touching(world: &World, pos: Pos) -> Vec<Dir> {
    let both =
        |&dir: &Dir| reaches(world, pos, dir) && reaches(world, pos.toward(dir), dir.opposite());
    Dir::ALL.into_iter().filter(both).collect()
}

/// Every awake holder's turn, then a liquid tube shows its fill.
pub fn tick(batch: &mut SystemBatch<Held>) {
    let world = batch.world();
    let ids = batch.ids().to_vec();
    let mut holders = Holders::of_batch(world, &ids, batch.values());
    for &id in &ids {
        if holders.get(id).asleep != 0 {
            continue;
        }
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        turn(&mut holders, id, pos);
        if world.block_typed(pos).is(&LIQUID_TUBE) {
            let mut held = holders.get(id);
            crate::liquid_tube::show(&world, pos, &mut held);
            holders.set(id, held);
        }
    }
}

/// The turn of the holder `id` at `pos`: liquid toward the sides of its block, then its head
/// told to its neighbours. With nothing moved or changed, it sleeps. Most blocks of a big tank
/// touch nothing, and sleep without finding the rest of their tank.
fn turn(holders: &mut Holders, id: u64, pos: Pos) {
    let world = *holders.world();
    let own = touching(&world, pos);
    let me = Vessel::at(&world, pos).filter(|_| !own.is_empty());
    let Some(me) = me else {
        sleep(holders, id);
        return;
    };
    let sides = me.sides(&world);
    let beside = |(at, dir): &(Pos, Dir)| Vessel::at(&world, at.toward(*dir));
    let neighbours: Vec<Vessel> = sides.iter().filter_map(beside).collect();
    let pushed = neighbours
        .iter()
        .map(|vessel| vessel.head_now(holders) - LOSS)
        .max()
        .unwrap_or(i64::MIN);
    let mut moved = false;
    for dir in own {
        if let Some(to) = Vessel::at(&world, pos.toward(dir)) {
            moved |= pass(holders, &me, pushed, &to);
        }
    }
    let head = me.head(me.held(holders).amount, pushed);
    let changed = head != me.told(holders);
    if changed {
        me.tell(holders, head);
    }
    if moved || changed {
        for vessel in &neighbours {
            vessel.wake(holders);
        }
    } else {
        sleep(holders, id);
    }
}

fn sleep(holders: &mut Holders, id: u64) {
    let mut held = holders.get(id);
    held.asleep = 1;
    holders.set(id, held);
}

/// Hands from `from`, pushed by `pushed`, to `to` what evens out their heads; whether any went.
/// A pump never takes: liquid run back into it would only be pushed out again.
fn pass(holders: &mut Holders, from: &Vessel, pushed: i64, to: &Vessel) -> bool {
    if matches!(to, Vessel::One { pump: true, .. }) {
        return false;
    }
    let (giving, taking) = (from.held(holders), to.held(holders));
    if giving.is_empty() || !taking.takes(giving.liquid, to.capacity()) {
        return false;
    }
    // A dribble of another liquid counts for nothing: it goes when this comes in.
    let had = if taking.liquid == giving.liquid {
        taking.amount
    } else {
        0
    };
    let higher = Level {
        head: from.head(giving.amount, pushed),
        per_block: from.per_block(),
    };
    let lower = Level {
        head: to.head(had, to.told(holders)),
        per_block: to.per_block(),
    };
    let room = to.capacity().saturating_sub(had);
    let amount = flow(higher, lower, MOST_A_TURN)
        .min(room)
        .min(giving.amount);
    if amount == 0 {
        return false;
    }
    from.take(holders, amount);
    to.put(holders, giving.liquid, amount);
    true
}
