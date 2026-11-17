//! The blocks that hold liquid: a liquid tube, a timber liquid tube, a pump, an oil engine's
//! intake, a grinding mill, a washer, a coke oven's door or drain, a firebox, a boiler, a steam
//! engine or a trip hammer, one block each, or a tank, one holder however many blocks it has.
//! `turn` passes liquid between them. What each holder keeps is read and written through `Holders`.

mod holders;

pub use holders::Holders;
use weft_sdk::{Dir, World};

use super::flow::{LIFT, UNIT};
use super::{DRIBBLE, Held, Level, LiquidKey, flow, is_full};
use crate::coke_oven;
use crate::firebox::{self, BOILER, FIREBOX};
use crate::grinding_mill::{self, GRINDING_MILL};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid_tube::is_liquid_tube;
use crate::oil_engine::OIL_INTAKE;
use crate::pump::PUMP;
use crate::steam_engine::{self, STEAM_ENGINE};
use crate::tank::{LIQUID_TANK, Tank};
use crate::timber_liquid_tube::{self, TIMBER_LIQUID_TUBE};
use crate::trip_hammer::{self, TRIP_HAMMER};
use crate::washer::{self, WASHER};

/// A holder of liquid: one block, or a tank of several.
pub enum Vessel {
    /// A liquid tube, a pump, an intake, a machine or a coke oven's door or drain: its block
    /// entity, where it is, how much it holds, and how it passes liquid on.
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
    /// An oil engine's intake: it takes oil and water for the engine and hands nothing on.
    Intake,
    /// A grinding mill, a washer or a boiler: it takes water for its work and hands nothing on.
    Mill,
    /// A coke oven's door or drain: it hands on the creosote the oven makes, by height as a tube
    /// does, and takes nothing.
    Drain,
    /// A firebox: it takes lava to burn and hands nothing on.
    Firebox,
    /// A steam engine or a trip hammer: it takes steam for its work and hands nothing on.
    Steam,
}

impl Kind {
    /// Whether it pushes what it holds up, and so never takes liquid.
    pub fn pushes(self) -> bool {
        matches!(self, Kind::Pump | Kind::Driven)
    }

    /// Whether it keeps what it takes for a machine, and hands nothing on.
    pub fn keeps(self) -> bool {
        matches!(
            self,
            Kind::Intake | Kind::Mill | Kind::Firebox | Kind::Steam
        )
    }

    /// Whether `liquid` may go into a holder of this kind at all.
    fn admits(self, liquid: LiquidKey) -> bool {
        match self {
            Kind::Intake => crate::oil_engine::burns(liquid),
            Kind::Mill => super::is_water(liquid),
            Kind::Drain => false,
            Kind::Firebox => liquid == super::key_named("base:lava"),
            Kind::Steam => super::rises(liquid),
            Kind::Tube | Kind::Pump | Kind::Driven => true,
        }
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
        } else if block.is(&OIL_INTAKE) {
            (crate::oil_engine::INTAKE_CAPACITY, Kind::Intake)
        } else if block.is(&GRINDING_MILL) {
            (grinding_mill::WATER_CAPACITY, Kind::Mill)
        } else if block.is(&WASHER) {
            (washer::WATER_CAPACITY, Kind::Mill)
        } else if let Some(capacity) = coke_oven::creosote_capacity(block) {
            (capacity, Kind::Drain)
        } else if block.is(&BOILER) {
            (firebox::WATER_CAPACITY, Kind::Mill)
        } else if block.is(&FIREBOX) {
            (firebox::LAVA_CAPACITY, Kind::Firebox)
        } else if block.is(&STEAM_ENGINE) {
            (steam_engine::STEAM_CAPACITY, Kind::Steam)
        } else if block.is(&TRIP_HAMMER) {
            (trip_hammer::STEAM_CAPACITY, Kind::Steam)
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

    /// Whether `liquid` may go into it at all: an intake takes only what an engine burns, a
    /// grinding mill, a washer or a boiler only water, a firebox only lava, a steam machine only
    /// steam, a coke oven's drain nothing.
    pub fn admits(&self, liquid: LiquidKey) -> bool {
        self.kind().admits(liquid)
    }

    /// How it passes liquid on: a tank as a tube does.
    pub fn kind(&self) -> Kind {
        match self {
            Vessel::One { kind, .. } => *kind,
            Vessel::Tank(_) => Kind::Tube,
        }
    }

    /// Where `amount` of `liquid` stands in it, with `pushed` the push coming into it from beside:
    /// a holder that pushes (`Kind::pushes`) pushes what it holds `LIFT` blocks up; another holder
    /// takes on the push when it is full (`is_full`); otherwise its liquid stands where its
    /// surface is. Steam has no height to it: it stands by how full the holder is
    /// (`flow::pressure`), and a millibucket of it raises a holder by one of its capacity.
    pub(super) fn level(&self, liquid: LiquidKey, amount: u32, pushed: i64) -> Level {
        let rises = super::rises(liquid);
        let per_block = if rises {
            self.capacity()
        } else {
            self.per_block()
        };
        Level {
            head: self.head(rises, amount, pushed),
            per_block,
        }
    }

    fn head(&self, rises: bool, amount: u32, pushed: i64) -> i64 {
        let (floor, per_block) = match self {
            Vessel::One { pos, kind, .. } if kind.pushes() && amount >= DRIBBLE => {
                let floor = if rises { 0 } else { pos.y as i64 };
                return (floor + LIFT) * UNIT;
            }
            Vessel::One { pos, capacity, .. } => (pos.y, *capacity),
            Vessel::Tank(tank) => (tank.floor(), tank.per_block()),
        };
        let surface = if rises {
            flow::pressure(amount, self.capacity())
        } else {
            flow::surface(floor, amount, per_block)
        };
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

    /// The head it stands at now, as a neighbour sees it at its block `id`, which keeps the head
    /// it last told there.
    pub(super) fn head_now(&self, holders: &Holders, id: u64) -> i64 {
        let held = self.held(holders);
        self.level(held.liquid, held.amount, holders.get(id).head)
            .head
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
    is_liquid_tube(block) || block.is(&PUMP) || is_machine(block) || block.is(&LIQUID_TANK)
}

/// Whether `block` is a machine that holds liquid for its work and reaches it on every side: an
/// intake, a grinding mill, a washer, a coke oven's door or drain, a firebox, a boiler, a steam
/// engine or a trip hammer.
fn is_machine(block: BlockState) -> bool {
    let machines: [&weft_sdk::Block; 7] = [
        &OIL_INTAKE,
        &GRINDING_MILL,
        &WASHER,
        &FIREBOX,
        &BOILER,
        &STEAM_ENGINE,
        &TRIP_HAMMER,
    ];
    machines.iter().any(|machine| block.is(machine))
        || coke_oven::creosote_capacity(block).is_some()
}

/// Whether the holder at `pos` passes liquid toward `dir`.
pub fn reaches(world: &World, pos: Pos, dir: Dir) -> bool {
    passes(world.block_typed(pos), dir)
}

/// Whether a holder whose block is `block` passes liquid toward `dir`: a liquid tube along its
/// arms, a pump on every side but its hose's, a machine on every side, a tank on every side it
/// has no more of itself.
fn passes(block: BlockState, dir: Dir) -> bool {
    if is_machine(block) {
        true
    } else if block.is(&PUMP) {
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
