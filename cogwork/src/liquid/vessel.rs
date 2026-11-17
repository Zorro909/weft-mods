//! The blocks that hold liquid: a liquid tube, a timber liquid tube, a pump, an oil engine's
//! intake, a grinding mill, a washer, a coke oven's door or drain, a firebox, a boiler, a steam
//! engine, a trip hammer, a flotation cell, a clarifier, a refinery's controller, heater,
//! condenser or tap, or a quarry, one block each, or a tank, one holder however many blocks it
//! has.
//! `turn` passes liquid between them. What each holder keeps is read and written through `Holders`.

mod discover;
mod holders;

pub use discover::{is_vessel, reaches};
pub use holders::Holders;

use super::flow::{LIFT, UNIT};
use super::{DRIBBLE, Held, Level, LiquidKey, flow, is_full};
use crate::lib::Pos;
use crate::tank::Tank;

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
    /// A grinding mill, a washer, a boiler, a flotation cell or a refinery's condenser: it takes
    /// water for its work and hands nothing on.
    Mill,
    /// A coke oven's door or drain, or a refinery's tap: it hands on what the machine makes, by
    /// height as a tube does, and takes nothing.
    Drain,
    /// A firebox: it takes lava to burn and hands nothing on.
    Firebox,
    /// A steam engine, a trip hammer or a refinery's heater: it takes steam for its work and
    /// hands nothing on.
    Steam,
    /// A clarifier: it takes muddy water to settle and hands nothing on.
    Settler,
    /// A refinery's controller: it takes crude oil to distil and hands nothing on.
    Still,
    /// A quarry: it takes lubricant for its drill and hands nothing on.
    Lubed,
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
            Kind::Intake
                | Kind::Mill
                | Kind::Firebox
                | Kind::Steam
                | Kind::Settler
                | Kind::Still
                | Kind::Lubed
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
            Kind::Settler => liquid == crate::muddy_water::key(),
            Kind::Still => liquid == super::key_named("cogwork:oil"),
            Kind::Lubed => liquid == crate::refinery::products::Product::Lubricant.key(),
            Kind::Tube | Kind::Pump | Kind::Driven => true,
        }
    }
}

impl Vessel {
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
    /// grinding mill, a washer, a boiler, a flotation cell or a condenser only water, a firebox
    /// only lava, a steam machine or a heater only steam, a clarifier only muddy water, a
    /// refinery's controller only crude, a quarry only lubricant, a drain or a tap nothing.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steam_caught_up_into_an_empty_tube_stands_by_its_pressure_not_its_height() {
        let high = Pos { x: 0, y: 40, z: 0 };
        let tube = Vessel::One {
            id: 1,
            pos: high,
            capacity: 1000,
            kind: Kind::Tube,
        };
        let empty = tube.level(super::super::NO_LIQUID, 0, 0).head;
        let mut held = Held {
            head: empty,
            ..Held::default()
        };
        held.add(crate::steam::key(), 1000);
        let full = tube.level(held.liquid, held.amount, held.head).head;
        assert_eq!(full, UNIT, "full of steam, a block's pressure");
    }
}
