//! Tests of how holders pass liquid and stand; `line` runs whole lines of them.

mod line;

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
