//! The engines' tick: make energy, push it out of the trunk, warm up or cool down, and move the
//! piston while running.

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

pub fn tick_engines(b: &mut SystemBatch<Engine>) {
    let w = b.world();
    for (id, e, ticks) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        let pos = Pos::from(st.block_pos());
        let running = match e.kind() {
            Kind::Spring => w.signal(pos) > 0,
            Kind::Ember => burn(&w, id, pos, e, ticks),
        };
        let made = if running { makes(e.kind()) } else { 0 };
        let full = store(&w, id, made * ticks as i64);
        push(&w, id, pos);
        let was_overheated = e.is_overheated();
        warm(e, running, full, ticks);
        e.stroke = piston::advance(e.stroke, running, w.game_time());
        sound(&w, pos, e, running, was_overheated);
        show_heat(&w, pos, e);
        piston::show(&w, id, pos, e);
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
fn burn(w: &World, id: u64, pos: Pos, e: &mut Engine, ticks: u32) -> bool {
    if e.is_overheated() {
        return false;
    }
    if e.burn == 0 {
        let slot = w.inventory(id).group("fuel");
        let fuel = slot.get(0);
        let Some(t) = fuel_ticks(fuel) else {
            return false;
        };
        e.burn = t;
        e.burn_max = t;
        slot.set(0, consume_one(fuel));
        w.play_sound(&sounds::EMBER_IGNITE, pos.center(), 1.0);
    }
    e.burn = e.burn.saturating_sub(ticks);
    true
}

/// Ticks of burning one item of `stack` gives, if it is fuel.
fn fuel_ticks(stack: ItemStack) -> Option<u32> {
    FUEL.on_item(stack.item).filter(|_| !stack.is_empty())
}

/// Adds what the engine made to its store; true when the store is full.
fn store(w: &World, id: u64, made: i64) -> bool {
    let Some(energy) = w.component_mut(id, &ENERGY) else {
        return false;
    };
    energy.stored = (energy.stored + made).min(energy.capacity);
    energy.stored == energy.capacity
}

/// Pushes stored energy into whatever block entity the trunk points at.
fn push(w: &World, id: u64, pos: Pos) {
    let Some(out) = model::output(w, pos) else {
        return;
    };
    let stored = w.component(id, &ENERGY).map_or(0, |e| e.stored);
    if stored == 0 {
        return;
    }
    let moved = w.energy_move(pos.toward(out), stored.min(PUSH), false);
    if let Some(energy) = w.component_mut(id, &ENERGY) {
        energy.stored -= moved;
    }
}

/// Heat climbs toward half while running and to the top when the energy can't get out; an ember
/// engine at the top stops until it has cooled. A spring engine never gets past warm.
fn warm(e: &mut Engine, running: bool, full: bool, ticks: u32) {
    let target = match (running, e.kind(), full) {
        (false, ..) => 0,
        (true, Kind::Spring, _) => SPRING_HEAT,
        (true, Kind::Ember, false) => EMBER_HEAT,
        (true, Kind::Ember, true) => OVERHEAT,
    };
    e.heat = if e.heat < target {
        (e.heat + ticks).min(target)
    } else {
        e.heat.saturating_sub(2 * ticks).max(target)
    };
    if e.heat >= OVERHEAT {
        e.overheated = 1;
    } else if e.heat <= COOLED {
        e.overheated = 0;
    }
}

/// The piston's knock at the top of each stroke, and the ember engine cutting out as it
/// overheats and starting again once cooled.
fn sound(w: &World, pos: Pos, e: &Engine, running: bool, was_overheated: bool) {
    let at = pos.center();
    if running && piston::at_top(e.stroke) {
        let stroke = match e.kind() {
            Kind::Spring => &sounds::SPRING_STROKE,
            Kind::Ember => &sounds::EMBER_STROKE,
        };
        w.play_sound(stroke, at, 1.0);
    }
    match (was_overheated, e.is_overheated()) {
        (false, true) => drop(w.play_sound(&sounds::OVERHEAT, at, 1.0)),
        (true, false) => drop(w.play_sound(&sounds::COOLED, at, 1.0)),
        _ => {}
    }
}

/// Shows the heat on the block's trunk.
fn show_heat(w: &World, pos: Pos, e: &Engine) {
    let level = heat_level(e).to_string();
    if w.prop(pos, "heat") != Some(level.as_str()) {
        w.prop_set(pos, "heat", &level);
    }
}

/// The block's `heat` for the engine's: 0 cool, 1 warm, 2 hot, and 3 while it cools down.
fn heat_level(e: &Engine) -> u32 {
    match e.heat {
        _ if e.is_overheated() => 3,
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
        let mut e = ember();
        warm(&mut e, true, true, OVERHEAT);
        assert!(e.is_overheated(), "full store, running: overheated");
        assert_eq!(heat_level(&e), 3, "and the trunk shows it");
        warm(&mut e, false, true, (OVERHEAT - COOLED) / 2 - 1);
        assert!(e.is_overheated(), "still too hot to start");
        warm(&mut e, false, true, 1);
        assert!(!e.is_overheated(), "cooled down");
        assert_eq!(heat_level(&e), 1, "warm, by its heat again");
    }

    #[test]
    fn running_engines_stay_below_overheating_while_delivering() {
        let mut e = ember();
        warm(&mut e, true, false, 5000);
        assert_eq!((e.heat, e.overheated), (600, 0), "ember settles at 600");
        let mut s = Engine::of(Kind::Spring);
        warm(&mut s, true, true, 5000);
        assert_eq!((s.heat, s.overheated), (450, 0), "spring never passes warm");
    }
}
