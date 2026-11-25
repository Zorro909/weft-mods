//! Finding the holders of liquid in the world: which block is one, with what capacity and kind,
//! and which of its sides reach another.

use weft_sdk::{Dir, World};

use super::{Kind, Vessel};
use crate::acid_tank::ACID_TANK;
use crate::clarifier::{self, CLARIFIER};
use crate::coke_oven;
use crate::excavator::kind::Kind as Quarry;
use crate::firebox::{self, BOILER, FIREBOX};
use crate::flotation::{self, FLOTATION_CELL};
use crate::grinding_mill::{self, GRINDING_MILL};
use crate::leaching_vat::LEACHING_VAT;
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid_tube::{PRESSURE_TUBE, SEALED_LIQUID_TUBE, is_liquid_tube};
use crate::oil_engine::OIL_INTAKE;
use crate::polymerizer::POLYMERIZER;
use crate::pump::PUMP;
use crate::refinery::{self, REFINERY, REFINERY_CONDENSER, REFINERY_HEATER, REFINERY_TAP};
use crate::steam_engine::{self, STEAM_ENGINE};
use crate::tank::{LIQUID_TANK, Tank};
use crate::timber_liquid_tube::{self, TIMBER_LIQUID_TUBE};
use crate::trip_hammer::{self, TRIP_HAMMER};
use crate::washer::{self, WASHER};

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
            (crate::liquid_tube::CAPACITY, tube_kind(block))
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
        } else if block.is(&FLOTATION_CELL) {
            (flotation::WATER_CAPACITY, Kind::Mill)
        } else if block.is(&CLARIFIER) {
            (clarifier::MUDDY_CAPACITY, Kind::Settler)
        } else if block.is(&POLYMERIZER) {
            (crate::polymerizer::GAS_CAPACITY, Kind::Polymer)
        } else if let Some(found) = refinery_part(block) {
            found
        } else if Quarry::of(block).is_some() {
            (crate::excavator::LUBRICANT_CAPACITY, Kind::Lubed)
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

    /// Every side of it that touches another holder, by the block it is on.
    pub(in crate::liquid) fn sides(&self, world: &World) -> Vec<(Pos, Dir)> {
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

/// How the liquid tube `block`, not a timber one, passes liquid on.
fn tube_kind(block: BlockState) -> Kind {
    if block.is(&SEALED_LIQUID_TUBE) {
        Kind::Sealed
    } else if block.is(&PRESSURE_TUBE) {
        Kind::Pressure
    } else {
        Kind::Tube
    }
}

/// The capacity and kind of the part of a refinery `block` is, if it holds liquid.
fn refinery_part(block: BlockState) -> Option<(u32, Kind)> {
    let parts = [
        (&REFINERY, refinery::CRUDE_CAPACITY, Kind::Still),
        (&REFINERY_HEATER, refinery::STEAM_CAPACITY, Kind::Steam),
        (&REFINERY_CONDENSER, refinery::WATER_CAPACITY, Kind::Mill),
        (&REFINERY_TAP, refinery::TAP_CAPACITY, Kind::Drain),
    ];
    let (_, capacity, kind) = parts.into_iter().find(|(part, ..)| block.is(part))?;
    Some((capacity, kind))
}

/// Whether the block at `pos` holds liquid.
pub fn is_vessel(world: &World, pos: Pos) -> bool {
    let block = world.block_typed(pos);
    is_liquid_tube(block) || block.is(&PUMP) || is_machine(block) || block.is(&LIQUID_TANK)
}

/// Whether `block` is a machine that holds liquid for its work and reaches it on every side: an
/// intake, a grinding mill, a washer, a coke oven's door or drain, a firebox, a boiler, a steam
/// engine, a trip hammer, a flotation cell, a clarifier, a refinery's part that holds liquid, a
/// quarry or a polymerizer; or one that draws its liquid in and lets it out itself, keeping it in
/// its own state (`liquid::draw`, `liquid::pour`) and so no holder: an acid tank or a
/// leaching vat.
fn is_machine(block: BlockState) -> bool {
    let machines: [&weft_sdk::Block; 12] = [
        &OIL_INTAKE,
        &GRINDING_MILL,
        &WASHER,
        &FIREBOX,
        &BOILER,
        &STEAM_ENGINE,
        &TRIP_HAMMER,
        &FLOTATION_CELL,
        &CLARIFIER,
        &POLYMERIZER,
        &ACID_TANK,
        &LEACHING_VAT,
    ];
    machines.iter().any(|machine| block.is(machine))
        || coke_oven::creosote_capacity(block).is_some()
        || refinery_part(block).is_some()
        || Quarry::of(block).is_some()
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
