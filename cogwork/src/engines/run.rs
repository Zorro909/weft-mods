//! The engines' tick: make energy, push it out of the trunk, warm up or cool down, and start
//! the piston's strokes while running.

use base_api::FUEL;
use weft_sdk::*;

use super::{Engine, Kind, model, piston};
use crate::lib::Pos;
use crate::sounds;

/// Energy per tick: a spring engine with a signal, an ember engine burning.
const SPRING_MAKES: i64 = 10;
const EMBER_MAKES: i64 = 40;
/// The most an engine pushes into the block at its trunk per tick.
const PUSH: i64 = 200;
/// Heat a running engine settles at: a spring engine, and an ember engine whose energy gets out.
const SPRING_HEAT: u32 = 450;
const EMBER_HEAT: u32 = 600;
/// Heat at which an ember engine stops, and below which it starts again.
const OVERHEAT: u32 = 1000;
const COOLED: u32 = 300;

pub fn tick_engines(batch: &mut SystemBatch<Engine>) {
    let world = batch.world();
    for (id, engine, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        let running = match engine.kind() {
            Kind::Spring => world.signal(pos) > 0,
            Kind::Ember => burn(&world, id, pos, engine, ticks),
        };
        let made = if running { makes(engine.kind()) } else { 0 };
        let full = store(&world, id, made * ticks as i64);
        push(&world, id, pos);
        let was_overheated = engine.is_overheated();
        warm(engine, running, full, ticks);
        let time = world.game_time();
        engine.stroke = piston::stroking(engine.is_stroking(), running, time) as u32;
        sound(&world, pos, engine, time, was_overheated);
        show_heat(&world, pos, engine);
        piston::show(&world, id, pos, engine, time);
    }
}

/// Energy per tick an engine of `kind` makes while it runs.
fn makes(kind: Kind) -> i64 {
    match kind {
        Kind::Spring => SPRING_MAKES,
        Kind::Ember => EMBER_MAKES,
    }
}

/// An ember engine's fire: lights the next fuel when out, unless it is cooling down. True while
/// burning.
fn burn(world: &World, id: u64, pos: Pos, engine: &mut Engine, ticks: u32) -> bool {
    if engine.is_overheated() {
        return false;
    }
    if engine.burn == 0 {
        let slot = world.inventory(id).group("fuel");
        let fuel = slot.get(0);
        let Some(burn_ticks) = fuel_ticks(fuel) else {
            return false;
        };
        engine.burn = burn_ticks;
        engine.burn_max = burn_ticks;
        slot.set(0, consume_one(fuel));
        world.play_sound(&sounds::EMBER_IGNITE, pos.center(), 1.0);
    }
    engine.burn = engine.burn.saturating_sub(ticks);
    true
}

/// Ticks of burning one item of `stack` gives, if it is fuel.
fn fuel_ticks(stack: ItemStack) -> Option<u32> {
    FUEL.on_item(stack.item).filter(|_| !stack.is_empty())
}

/// Adds what the engine made to its store; true when the store is full.
fn store(world: &World, id: u64, made: i64) -> bool {
    let Some(energy) = world.component_mut(id, &ENERGY) else {
        return false;
    };
    energy.stored = (energy.stored + made).min(energy.capacity);
    energy.stored == energy.capacity
}

/// Pushes stored energy into whatever block entity the trunk points at.
fn push(world: &World, id: u64, pos: Pos) {
    let Some(out) = model::output(world, pos) else {
        return;
    };
    let stored = world
        .component(id, &ENERGY)
        .map_or(0, |energy| energy.stored);
    if stored == 0 {
        return;
    }
    let moved = world.energy_move(pos.toward(out), stored.min(PUSH), false);
    if let Some(energy) = world.component_mut(id, &ENERGY) {
        energy.stored -= moved;
    }
}

/// Heat climbs toward half while running and to the top when the energy can't get out; an ember
/// engine at the top stops until it has cooled. A spring engine never gets past warm.
fn warm(engine: &mut Engine, running: bool, full: bool, ticks: u32) {
    let target = match (running, engine.kind(), full) {
        (false, ..) => 0,
        (true, Kind::Spring, _) => SPRING_HEAT,
        (true, Kind::Ember, false) => EMBER_HEAT,
        (true, Kind::Ember, true) => OVERHEAT,
    };
    engine.heat = if engine.heat < target {
        (engine.heat + ticks).min(target)
    } else {
        engine.heat.saturating_sub(2 * ticks).max(target)
    };
    if engine.heat >= OVERHEAT {
        engine.overheated = 1;
    } else if engine.heat <= COOLED {
        engine.overheated = 0;
    }
}

/// The piston's knock at the top of each stroke, and the ember engine cutting out as it
/// overheats and starting again once cooled.
fn sound(world: &World, pos: Pos, engine: &Engine, time: u64, was_overheated: bool) {
    let at = pos.center();
    if engine.is_stroking() && piston::at_top(time) {
        let stroke = match engine.kind() {
            Kind::Spring => &sounds::SPRING_STROKE,
            Kind::Ember => &sounds::EMBER_STROKE,
        };
        world.play_sound(stroke, at, 1.0);
    }
    match (was_overheated, engine.is_overheated()) {
        (false, true) => drop(world.play_sound(&sounds::OVERHEAT, at, 1.0)),
        (true, false) => drop(world.play_sound(&sounds::COOLED, at, 1.0)),
        _ => {}
    }
}

/// Shows the heat on the block's trunk.
fn show_heat(world: &World, pos: Pos, engine: &Engine) {
    let level = heat_level(engine).to_string();
    if world.prop(pos, "heat") != Some(level.as_str()) {
        world.prop_set(pos, "heat", &level);
    }
}

/// The block's `heat` for the engine's: 0 cool, 1 warm, 2 hot, and 3 while it cools down.
fn heat_level(engine: &Engine) -> u32 {
    match engine.heat {
        _ if engine.is_overheated() => 3,
        0..250 => 0,
        250..500 => 1,
        _ => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ember() -> Engine {
        Engine::of(Kind::Ember)
    }

    #[test]
    fn ember_engine_overheats_when_its_energy_cannot_leave() {
        let mut engine = ember();
        warm(&mut engine, true, true, OVERHEAT);
        assert!(engine.is_overheated(), "full store, running: overheated");
        assert_eq!(heat_level(&engine), 3, "and the trunk shows it");
        warm(&mut engine, false, true, (OVERHEAT - COOLED) / 2 - 1);
        assert!(engine.is_overheated(), "still too hot to start");
        warm(&mut engine, false, true, 1);
        assert!(!engine.is_overheated(), "cooled down");
        assert_eq!(heat_level(&engine), 1, "warm, by its heat again");
    }

    #[test]
    fn running_engines_stay_below_overheating_while_delivering() {
        let mut engine = ember();
        warm(&mut engine, true, false, 5000);
        assert_eq!(
            (engine.heat, engine.overheated),
            (600, 0),
            "ember settles at 600"
        );
        let mut spring = Engine::of(Kind::Spring);
        warm(&mut spring, true, true, 5000);
        assert_eq!(
            (spring.heat, spring.overheated),
            (450, 0),
            "spring never passes warm"
        );
    }
}
