//! Tests of how holders pass liquid and stand.

use std::cell::Cell;

use super::*;
use crate::liquid::MOST_A_TURN;

fn at(x: i32) -> Pos {
    Pos { x, y: 0, z: 0 }
}

fn tube(id: u64, pos: Pos) -> Vessel {
    Vessel::One {
        id,
        pos,
        capacity: 1000,
        kind: Kind::Tube,
    }
}

#[test]
fn a_driven_timber_tube_pushes_as_the_pump_does() {
    let pos = Pos { x: 0, y: 10, z: 0 };
    let driven = Vessel::One {
        id: 1,
        pos,
        capacity: 1000,
        kind: Kind::Driven,
    };
    let lift = (10 + super::super::flow::LIFT) * super::super::flow::UNIT;
    let oil = super::super::key_named("cogwork:oil");
    assert_eq!(driven.level(oil, 500, 0).head, lift, "half full, it pushes");
    assert!(
        tube(1, pos).level(oil, 500, 0).head < lift,
        "undriven, it doesn't"
    );
    assert!(
        Kind::Driven.pushes() && !Kind::Tube.pushes(),
        "nor takes when it does"
    );
}

#[test]
fn an_intake_takes_only_what_an_oil_engine_burns() {
    let intake = Vessel::One {
        id: 1,
        pos: at(0),
        capacity: 1000,
        kind: Kind::Intake,
    };
    let named = super::super::key_named;
    assert!(intake.admits(named("cogwork:oil")), "oil");
    assert!(intake.admits(named("base:water")), "water for vapor");
    assert!(!intake.admits(named("base:lava")), "nothing else");
    assert!(
        tube(1, at(0)).admits(named("base:lava")),
        "a tube takes any liquid"
    );
}

#[test]
fn a_grinding_mill_or_washer_takes_only_water_and_keeps_it() {
    let mill = Vessel::One {
        id: 1,
        pos: at(0),
        capacity: 1000,
        kind: Kind::Mill,
    };
    let named = super::super::key_named;
    assert!(mill.admits(named("base:water")), "water for its grinds");
    assert!(!mill.admits(named("cogwork:oil")), "not oil");
    assert!(
        Kind::Mill.keeps() && Kind::Intake.keeps(),
        "hands nothing on"
    );
    assert!(!Kind::Tube.keeps(), "a tube does");
}

#[test]
fn a_coke_oven_drain_hands_on_and_takes_nothing() {
    let drain = Vessel::One {
        id: 1,
        pos: at(0),
        capacity: 1000,
        kind: Kind::Drain,
    };
    let named = super::super::key_named;
    assert!(
        !drain.admits(named("cogwork:creosote")),
        "not even creosote"
    );
    assert!(!drain.admits(named("base:water")), "nor water");
    assert!(
        !Kind::Drain.keeps() && !Kind::Drain.pushes(),
        "it hands on by height, as a tube"
    );
}

#[test]
fn water_oil_and_creosote_stand_as_they_did_before_steam() {
    let named = super::super::key_named;
    let liquids = ["base:water", "cogwork:oil", "cogwork:creosote", "base:lava"];
    let high = Pos { x: 0, y: 37, z: 0 };
    for liquid in liquids.map(named) {
        for amount in [0, 1, 250, 999, 1000] {
            let level = tube(1, high).level(liquid, amount, 0);
            let surface = flow::surface(high.y, amount, 1000);
            let before = if amount + MOST_A_TURN >= 1000 {
                surface.max(0)
            } else {
                surface
            };
            assert_eq!(level.head, before, "{amount} mB by its height");
            assert_eq!(level.per_block, 1000, "a tube's block");
        }
        let pushed = tube(1, high).level(liquid, 1000, 99 * flow::UNIT).head;
        assert_eq!(pushed, 99 * flow::UNIT, "a full tube passes on the push");
    }
}

#[test]
fn steam_stands_by_how_full_its_holder_is_wherever_that_is() {
    let steam = crate::steam::key();
    let low = tube(1, at(0)).level(steam, 500, 0);
    let high = tube(2, Pos { x: 0, y: 40, z: 0 }).level(steam, 500, 0);
    assert_eq!(low, high, "half full is half full");
    assert_eq!(low.head, flow::UNIT / 2, "half a block's head");
}

#[test]
fn steam_and_other_liquids_push_only_their_own() {
    let water_line = [(false, 40 * flow::UNIT), (true, flow::UNIT)];
    assert_eq!(
        push_on(false, water_line.into_iter()),
        40 * flow::UNIT - LOSS,
        "water, by the water beside it"
    );
    assert_eq!(
        push_on(true, water_line.into_iter()),
        flow::UNIT - LOSS,
        "steam, by the steam beside it"
    );
    let only_steam = [(true, 5 * flow::UNIT)];
    assert_eq!(
        push_on(false, only_steam.into_iter()),
        i64::MIN,
        "steam never pushes water"
    );
}

#[test]
fn a_firebox_takes_only_lava_and_a_steam_machine_only_steam() {
    let named = super::super::key_named;
    let holder = |kind| Vessel::One {
        id: 1,
        pos: at(0),
        capacity: 1000,
        kind,
    };
    assert!(
        holder(Kind::Firebox).admits(named("base:lava")),
        "lava to burn"
    );
    assert!(
        !holder(Kind::Firebox).admits(named("base:water")),
        "not water"
    );
    assert!(holder(Kind::Steam).admits(crate::steam::key()), "steam");
    assert!(
        !holder(Kind::Steam).admits(named("base:water")),
        "not water"
    );
    assert!(
        Kind::Firebox.keeps() && Kind::Steam.keeps(),
        "both hand nothing on"
    );
}

#[test]
fn a_holder_is_looked_up_once_a_tick() {
    let looked = Cell::new(0);
    let find = |pos| {
        looked.set(looked.get() + 1);
        Some(tube(7, pos))
    };
    let mut found = Found::default();
    let first = found.at(at(1), find).expect("a tube is there");
    let again = found.at(at(1), find).expect("and still is");
    assert_eq!(looked.get(), 1, "the second time it is known");
    assert!(Rc::ptr_eq(&first.vessel, &again.vessel), "the same holder");
    assert_eq!(again.id, 7, "met at its block");
}

#[test]
fn a_block_that_holds_nothing_is_looked_up_once_too() {
    let looked = Cell::new(0);
    let find = |_| {
        looked.set(looked.get() + 1);
        None
    };
    let mut found = Found::default();
    assert!(found.at(at(1), find).is_none(), "nothing there");
    assert!(found.at(at(1), find).is_none(), "nor the second time");
    assert_eq!(looked.get(), 1, "known to hold nothing");
    assert!(found.at(at(2), find).is_none(), "another block");
    assert_eq!(looked.get(), 2, "is another question");
}

/// A line of holders, each joined to the next: what each is, what it keeps and the head it told.
struct Line {
    vessels: Vec<Vessel>,
    held: Vec<Held>,
    told: Vec<i64>,
}

impl Line {
    fn new(vessels: Vec<Vessel>, held: Vec<Held>) -> Line {
        let told = vec![super::super::NO_HEAD; vessels.len()];
        Line {
            vessels,
            held,
            told,
        }
    }

    /// Every holder's turn in order, as `turn` takes it: the push into it from its neighbours,
    /// what it hands each, then the head it tells.
    fn turn(&mut self) {
        for at in 0..self.vessels.len() {
            let me = &self.vessels[at];
            let beside: Vec<usize> = [at.checked_sub(1), Some(at + 1)]
                .into_iter()
                .flatten()
                .filter(|&other| other < self.vessels.len())
                .collect();
            let heads = beside.iter().map(|&other| {
                let held = self.held[other];
                let seen = self.vessels[other].level_into(
                    held.liquid,
                    held.amount,
                    self.told[other],
                    me.kind(),
                );
                (super::super::rises(held.liquid), seen.head)
            });
            let pushed = push_on(super::super::rises(self.held[at].liquid), heads);
            for &other in &beside {
                let from = Party::of(me, self.held[at], pushed);
                let to = Party::of(&self.vessels[other], self.held[other], self.told[other]);
                let liquid = self.held[at].liquid;
                let amount = share(from, to);
                if amount > 0 {
                    self.held[at].remove(amount);
                    self.held[other].add(liquid, amount);
                }
            }
            let held = self.held[at];
            self.told[at] = me.level(held.liquid, held.amount, pushed).head;
        }
    }

    /// Millibuckets of `liquid` in the whole line.
    fn total(&self, liquid: super::super::LiquidKey) -> u32 {
        let of = |held: &&Held| held.liquid == liquid;
        self.held.iter().filter(of).map(|held| held.amount).sum()
    }
}

fn holder(kind: Kind, y: i32) -> Vessel {
    Vessel::One {
        id: 1,
        pos: Pos { x: 0, y, z: 0 },
        capacity: 1000,
        kind,
    }
}

fn holding(liquid: super::super::LiquidKey, amount: u32) -> Held {
    Held {
        liquid,
        amount,
        head: super::super::NO_HEAD,
        ..Held::default()
    }
}

/// The highest of a column of `kind` over a pump that a pump keeps full of water fills.
fn pumped_up(kind: Kind) -> usize {
    let water = super::super::key_named("base:water");
    let mut vessels = vec![holder(Kind::Pump, 0)];
    vessels.extend((1..=40).map(|y| holder(kind, y)));
    let mut held = vec![Held::default(); vessels.len()];
    held[0] = holding(water, 1000);
    let mut line = Line::new(vessels, held);
    for _ in 0..4_000 {
        line.held[0] = holding(water, 1000);
        line.turn();
    }
    (1..line.vessels.len())
        .rfind(|&at| line.held[at].amount > 0)
        .unwrap_or(0)
}

#[test]
fn a_sealed_column_lifts_a_pump_s_water_twice_as_high() {
    let plain = pumped_up(Kind::Tube);
    let sealed = pumped_up(Kind::Sealed);
    assert!((14..=16).contains(&plain), "a liquid tube: {plain} up");
    assert!((29..=32).contains(&sealed), "a sealed tube: {sealed} up");
}

#[test]
fn a_pump_lifts_into_a_sealed_tube_only_its_own_lift() {
    let pump = holder(Kind::Pump, 10);
    let water = super::super::key_named("base:water");
    let into = |kind| pump.level_into(water, 500, 0, kind).head / flow::UNIT;
    assert_eq!(into(Kind::Tube), 10 + flow::LIFT, "16 into a tube");
    assert_eq!(
        into(Kind::Sealed),
        10 + flow::SEALED_LIFT,
        "32 into a sealed one"
    );
    assert_eq!(
        into(Kind::Pressure),
        10 + flow::LIFT,
        "16 into a pressure tube"
    );
}

#[test]
fn a_pressure_tube_carries_twice_as_much_steam_a_turn() {
    let steam = crate::steam::key();
    let between = |kind| {
        let (from, to) = (holder(kind, 0), holder(kind, 0));
        share(
            Party::of(&from, holding(steam, 1000), i64::MIN),
            Party::of(&to, Held::default(), i64::MIN),
        )
    };
    assert_eq!(between(Kind::Tube), MOST_A_TURN, "a tube's turn");
    assert_eq!(between(Kind::Pressure), 2 * MOST_A_TURN, "twice that");
}

#[test]
fn a_pressure_tube_takes_only_steam() {
    let water = super::super::key_named("base:water");
    let (tube, pressure) = (holder(Kind::Tube, 1), holder(Kind::Pressure, 0));
    let water_in = share(
        Party::of(&tube, holding(water, 1000), i64::MIN),
        Party::of(&pressure, Held::default(), i64::MIN),
    );
    assert_eq!(water_in, 0, "no water falls into it");
    assert!(pressure.admits(crate::steam::key()), "steam goes in");
}

#[test]
fn a_mixed_line_makes_and_loses_nothing_and_keeps_steam_and_water_apart() {
    let (steam, water) = (crate::steam::key(), super::super::key_named("base:water"));
    let kinds = [
        (Kind::Tube, water, 1000),
        (Kind::Sealed, water, 600),
        (Kind::Pressure, steam, 1000),
        (Kind::Pressure, NO_LIQUID, 0),
        (Kind::Tube, steam, 300),
        (Kind::Pressure, steam, 900),
        (Kind::Tube, NO_LIQUID, 0),
        (Kind::Sealed, water, 9),
        (Kind::Pressure, NO_LIQUID, 0),
    ];
    let vessels = (0..).zip(kinds).map(|(y, (kind, ..))| holder(kind, y % 3));
    let held = kinds.map(|(_, liquid, amount)| holding(liquid, amount));
    let mut line = Line::new(vessels.collect(), held.to_vec());
    let (steam_before, water_before) = (line.total(steam), line.total(water));
    for _ in 0..500 {
        let was = line.held.clone();
        line.turn();
        for (before, after) in was.iter().zip(&line.held) {
            let replaced = before.amount > 0 && after.amount > 0 && before.liquid != after.liquid;
            assert!(!replaced, "{before:?} became {after:?}");
        }
        assert_eq!(line.total(steam), steam_before, "no steam made or lost");
        assert_eq!(line.total(water), water_before, "no water made or lost");
    }
    let pressure_water = (0..kinds.len())
        .filter(|&at| line.vessels[at].kind() == Kind::Pressure)
        .any(|at| line.held[at].liquid == water && line.held[at].amount > 0);
    assert!(!pressure_water, "water never got into a pressure tube");
}

#[test]
fn a_new_liquid_in_a_pressure_tube_starts_with_no_head() {
    let steam = crate::steam::key();
    let mut held = Held {
        liquid: NO_LIQUID,
        head: 40 * flow::UNIT,
        ..Held::default()
    };
    held.add(steam, 200);
    assert_eq!(
        held.head,
        super::super::NO_HEAD,
        "the head told of the old is gone"
    );
}
