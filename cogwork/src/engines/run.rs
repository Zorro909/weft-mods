//! The engines' tick: make energy, push it out of the trunk, warm up or cool down, and move the
//! piston while running.

use base_api::FUEL;
use weft_sdk::*;

use super::piston;
use super::{Engine, kind, model};
use crate::lib::Pos;
use crate::sounds;

/// Energy per tick: a spring engine with a signal, an ember engine burning.
const SPRING_MAKES: i64 = 10;
const EMBER_MAKES: i64 = 40;
/// The most an engine pushes into the block at its trunk per tick.
const PUSH: i64 = 200;
/// Heat at which an ember engine stops, and below which it starts again.
const OVERHEAT: u32 = 1000;
const COOLED: u32 = 300;

pub fn tick_engines(b: &mut SystemBatch<Engine>) {
    let w = b.world();
    for (id, e, ticks) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        let pos = Pos::from(st.block_pos());
        let running = if e.kind == kind::SPRING {
            w.signal(pos) > 0
        } else {
            burn(&w, id, pos, e, ticks)
        };
        let made = if !running {
            0
        } else if e.kind == kind::SPRING {
            SPRING_MAKES
        } else {
            EMBER_MAKES
        };
        let full = store(&w, id, made * ticks as i64);
        push(&w, id, pos);
        let was_overheated = e.overheated;
        warm(e, running, full, ticks);
        e.stroke = piston::advance(e.stroke, running, w.game_time());
        sound(&w, pos, e, running, was_overheated);
        show_heat(&w, pos, e);
        piston::show(&w, id, pos, e);
    }
}

/// An ember engine's fire: lights the next fuel when out, unless it is cooling down. True while
/// burning.
fn burn(w: &World, id: u64, pos: Pos, e: &mut Engine, ticks: u32) -> bool {
    if e.overheated != 0 {
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
    let target = match (running, full, e.kind) {
        (false, _, _) => 0,
        (true, _, kind::SPRING) => 450,
        (true, false, _) => 600,
        (true, true, _) => OVERHEAT,
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
fn sound(w: &World, pos: Pos, e: &Engine, running: bool, was_overheated: u32) {
    let at = pos.center();
    if running && piston::at_top(e.stroke) {
        let stroke = if e.kind == kind::SPRING {
            &sounds::SPRING_STROKE
        } else {
            &sounds::EMBER_STROKE
        };
        w.play_sound(stroke, at, 1.0);
    }
    match (was_overheated, e.overheated) {
        (0, 1) => drop(w.play_sound(&sounds::OVERHEAT, at, 1.0)),
        (1, 0) => drop(w.play_sound(&sounds::COOLED, at, 1.0)),
        _ => {}
    }
}

/// Shows the heat on the block's trunk.
fn show_heat(w: &World, pos: Pos, e: &Engine) {
    let heat = match e.heat {
        _ if e.overheated != 0 => 3,
        0..250 => 0,
        250..500 => 1,
        _ => 2,
    };
    let heat = heat.to_string();
    if w.prop(pos, "heat") != Some(heat.as_str()) {
        w.prop_set(pos, "heat", &heat);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ember() -> Engine {
        Engine {
            kind: kind::EMBER,
            ..Engine::default()
        }
    }

    #[test]
    fn ember_engine_overheats_when_its_energy_cannot_leave() {
        let mut e = ember();
        warm(&mut e, true, true, OVERHEAT);
        assert_eq!(e.overheated, 1, "full store, running: overheated");
        warm(&mut e, false, true, (OVERHEAT - COOLED) / 2 - 1);
        assert_eq!(e.overheated, 1, "still too hot to start");
        warm(&mut e, false, true, 1);
        assert_eq!(e.overheated, 0, "cooled down");
    }

    #[test]
    fn running_engines_stay_below_overheating_while_delivering() {
        let mut e = ember();
        warm(&mut e, true, false, 5000);
        assert_eq!((e.heat, e.overheated), (600, 0), "ember settles at 600");
        let mut s = Engine {
            kind: kind::SPRING,
            ..Engine::default()
        };
        warm(&mut s, true, true, 5000);
        assert_eq!((s.heat, s.overheated), (450, 0), "spring never passes warm");
    }
}
