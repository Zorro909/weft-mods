//! Lines of holders run turn after turn: how high a pump lifts, what each kind of tube takes and
//! carries, and that a mixed line makes and loses nothing.

use super::*;
use crate::liquid::{LiquidKey, MOST_A_TURN, NO_HEAD, key_named, rises};

/// A line of holders, each joined to the next: what each is, what it keeps and the head it told.
struct Line {
    vessels: Vec<Vessel>,
    held: Vec<Held>,
    told: Vec<i64>,
}

impl Line {
    fn new(vessels: Vec<Vessel>, held: Vec<Held>) -> Line {
        let told = vec![NO_HEAD; vessels.len()];
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
                (rises(held.liquid), seen.head)
            });
            let pushed = push_on(rises(self.held[at].liquid), heads);
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
    fn total(&self, liquid: LiquidKey) -> u32 {
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

fn holding(liquid: LiquidKey, amount: u32) -> Held {
    Held {
        liquid,
        amount,
        head: NO_HEAD,
        ..Held::default()
    }
}

/// The highest of a column of `kind` over a pump that a pump keeps full of water fills.
fn pumped_up(kind: Kind) -> usize {
    let water = key_named("base:water");
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
    let water = key_named("base:water");
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
fn a_pressure_tube_takes_only_steam_and_acid() {
    let water = key_named("base:water");
    let (tube, pressure) = (holder(Kind::Tube, 1), holder(Kind::Pressure, 0));
    let falls_in = |liquid| {
        share(
            Party::of(&tube, holding(liquid, 1000), i64::MIN),
            Party::of(&pressure, Held::default(), i64::MIN),
        )
    };
    assert_eq!(falls_in(water), 0, "no water falls into it");
    assert_eq!(
        falls_in(crate::acid::key()),
        2 * MOST_A_TURN,
        "acid does, at the pressure tube's pace"
    );
    assert!(pressure.admits(crate::steam::key()), "steam goes in");
}

#[test]
fn a_pressure_tube_carries_twice_as_much_acid_a_turn_and_by_height() {
    let acid = crate::acid::key();
    let down = |kind| {
        let (from, to) = (holder(kind, 1), holder(kind, 0));
        share(
            Party::of(&from, holding(acid, 1000), i64::MIN),
            Party::of(&to, Held::default(), i64::MIN),
        )
    };
    assert_eq!(down(Kind::Tube), MOST_A_TURN, "a tube's turn");
    assert_eq!(down(Kind::Pressure), 2 * MOST_A_TURN, "twice that");
    let (low, high) = (holder(Kind::Pressure, 0), holder(Kind::Pressure, 1));
    let up = share(
        Party::of(&low, holding(acid, 500), i64::MIN),
        Party::of(&high, Held::default(), i64::MIN),
    );
    assert_eq!(up, 0, "acid stands as water does, it never rises");
}

#[test]
fn a_pressure_line_keeps_steam_and_acid_apart_and_loses_neither() {
    let (steam, acid) = (crate::steam::key(), crate::acid::key());
    let kinds = [
        (Kind::Pressure, acid, 1000),
        (Kind::Pressure, steam, 800),
        (Kind::Pressure, NO_LIQUID, 0),
        (Kind::Pressure, acid, 5),
        (Kind::Tube, acid, 600),
        (Kind::Pressure, steam, 3),
        (Kind::Pressure, NO_LIQUID, 0),
    ];
    let vessels = (0..).zip(kinds).map(|(y, (kind, ..))| holder(kind, y % 2));
    let held = kinds.map(|(_, liquid, amount)| holding(liquid, amount));
    let mut line = Line::new(vessels.collect(), held.to_vec());
    let (steam_before, acid_before) = (line.total(steam), line.total(acid));
    for _ in 0..500 {
        let was = line.held.clone();
        line.turn();
        for (before, after) in was.iter().zip(&line.held) {
            let rising = |held: &Held| rises(held.liquid);
            let replaced = before.amount > 0 && after.amount > 0 && rising(before) != rising(after);
            assert!(!replaced, "{before:?} became {after:?}");
        }
        assert_eq!(line.total(steam), steam_before, "no steam made or lost");
        assert_eq!(line.total(acid), acid_before, "no acid made or lost");
    }
}

#[test]
fn a_mixed_line_makes_and_loses_nothing_and_keeps_steam_and_water_apart() {
    let (steam, water) = (crate::steam::key(), key_named("base:water"));
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
    assert_eq!(held.head, NO_HEAD, "the head told of the old is gone");
}
