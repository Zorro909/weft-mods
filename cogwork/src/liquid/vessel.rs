//! The blocks that hold liquid: a liquid tube, a timber liquid tube or a pump, one block each, or
//! a tank, one holder however many blocks it has. `turn` passes liquid between them.
//!
//! A system works on copies of its values, written back when it returns, so a holder changed
//! during the tick goes through `Holders`, which changes the copy when there is one.

use std::collections::HashMap;

use weft_sdk::{Dir, World};

use super::flow::{LIFT, UNIT};
use super::{DRIBBLE, Held, LiquidKey, flow, is_full};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid_tube::is_liquid_tube;
use crate::pump::PUMP;
use crate::tank::{LIQUID_TANK, Tank};
use crate::timber_liquid_tube::{self, TIMBER_LIQUID_TUBE};

/// The `Held` of every holder, read and written through this tick's copies where they are.
pub struct Holders<'a> {
    world: World,
    index: HashMap<u64, usize>,
    values: &'a mut [Held],
}

impl<'a> Holders<'a> {
    /// The holders of a system's batch, the others reached through the world.
    pub(super) fn of_batch(world: World, ids: &[u64], values: &'a mut [Held]) -> Holders<'a> {
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
    /// A liquid tube or a pump: its block entity, where it is, how much it holds, and how it
    /// passes liquid on.
    One {
        id: u64,
        pos: Pos,
        capacity: u32,
        kind: Kind,
    },
    Tank(Tank),
}

/// How a holder of one block passes liquid on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A liquid tube, or a timber liquid tube no engine drives: by height and pressure.
    Tube,
    /// The pump: it pushes what it holds `LIFT` blocks up and never takes liquid.
    Pump,
    /// A timber liquid tube an engine drives: it pushes as the pump does, and hands nothing to a
    /// tank, the tanks it touches being where it pulls liquid from.
    Driven,
}

impl Kind {
    /// Whether it pushes what it holds up, and so never takes liquid.
    pub fn pushes(self) -> bool {
        matches!(self, Kind::Pump | Kind::Driven)
    }
}

impl Vessel {
    /// The holder whose block is at `pos`, if one is.
    pub fn at(world: &World, pos: Pos) -> Option<Vessel> {
        let block = world.block_typed(pos);
        let id = world.block_entity(pos);
        let (capacity, kind) = if block.is(&TIMBER_LIQUID_TUBE) {
            let driven = timber_liquid_tube::is_driven(world, id);
            let kind = if driven { Kind::Driven } else { Kind::Tube };
            (crate::liquid_tube::CAPACITY, kind)
        } else if is_liquid_tube(block) {
            (crate::liquid_tube::CAPACITY, Kind::Tube)
        } else if block.is(&PUMP) {
            (crate::pump::CAPACITY, Kind::Pump)
        } else if block.is(&LIQUID_TANK) {
            return Tank::at(world, pos).map(Vessel::Tank);
        } else {
            return None;
        };
        (id != 0).then_some(Vessel::One {
            id,
            pos,
            capacity,
            kind,
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

    /// How it passes liquid on: a tank as a tube does.
    pub fn kind(&self) -> Kind {
        match self {
            Vessel::One { kind, .. } => *kind,
            Vessel::Tank(_) => Kind::Tube,
        }
    }

    /// The head of `amount` in it, with `pushed` the push coming into it from beside: a holder
    /// that pushes (`Kind::pushes`) pushes what it holds `LIFT` blocks up; another holder takes
    /// on the push when it is full (`is_full`); otherwise its liquid stands where its surface is.
    pub(super) fn head(&self, amount: u32, pushed: i64) -> i64 {
        let (floor, per_block) = match self {
            Vessel::One { pos, kind, .. } if kind.pushes() && amount >= DRIBBLE => {
                return (pos.y as i64 + LIFT) * UNIT;
            }
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
    pub(super) fn per_block(&self) -> u32 {
        match self {
            Vessel::One { capacity, .. } => *capacity,
            Vessel::Tank(tank) => tank.per_block(),
        }
    }

    /// The head it stands at now, as a neighbour sees it at its block `id`, which keeps the head
    /// it last told there.
    pub(super) fn head_now(&self, holders: &Holders, id: u64) -> i64 {
        self.head(self.held(holders).amount, holders.get(id).head)
    }

    /// Tells its neighbours its head is `head`.
    pub(super) fn tell(&self, holders: &mut Holders, head: i64) {
        self.each_block(holders, |held| held.head = head);
    }

    /// Wakes it, every block of it.
    pub(super) fn wake(&self, holders: &mut Holders) {
        self.each_block(holders, |held| held.asleep = 0);
    }

    fn each_block(&self, holders: &mut Holders, change: impl Fn(&mut Held)) {
        for (id, _) in self.blocks() {
            let was = holders.get(id);
            let mut held = was;
            change(&mut held);
            if held != was {
                holders.set(id, held);
            }
        }
    }

    /// Its blocks, each with its block entity.
    pub(super) fn blocks(&self) -> Vec<(u64, Pos)> {
        match self {
            Vessel::One { id, pos, .. } => vec![(*id, *pos)],
            Vessel::Tank(tank) => tank.ids().zip(tank.blocks()).collect(),
        }
    }

    /// Every side of it that touches another holder, by the block it is on.
    pub(super) fn sides(&self, world: &World) -> Vec<(Pos, Dir)> {
        let reached = |&(pos, dir): &(Pos, Dir)| reaches(world, pos.toward(dir), dir.opposite());
        match self {
            Vessel::One { pos, .. } => {
                let own = touching(world, *pos).into_iter();
                own.map(|dir| (*pos, dir)).collect()
            }
            Vessel::Tank(tank) => tank.open_sides().filter(reached).collect(),
        }
    }
}

/// Whether the block at `pos` holds liquid.
pub fn is_vessel(world: &World, pos: Pos) -> bool {
    let block = world.block_typed(pos);
    is_liquid_tube(block) || block.is(&PUMP) || block.is(&LIQUID_TANK)
}

/// Whether the holder at `pos` passes liquid toward `dir`.
pub fn reaches(world: &World, pos: Pos, dir: Dir) -> bool {
    passes(world.block_typed(pos), dir)
}

/// Whether a holder whose block is `block` passes liquid toward `dir`: a liquid tube along its
/// arms, a pump on every side but its hose's, a tank on every side it has no more of itself.
fn passes(block: BlockState, dir: Dir) -> bool {
    if block.is(&PUMP) {
        dir != Dir::Down
    } else if is_liquid_tube(block) {
        block.prop(dir.name()) == Some("true")
    } else {
        block.is(&LIQUID_TANK) && block.prop(dir.name()) == Some("false")
    }
}

/// The sides of the holder at `pos` where it and a holder beside reach each other.
fn touching(world: &World, pos: Pos) -> Vec<Dir> {
    let block = world.block_typed(pos);
    let both = |&dir: &Dir| passes(block, dir) && reaches(world, pos.toward(dir), dir.opposite());
    Dir::ALL.into_iter().filter(both).collect()
}
