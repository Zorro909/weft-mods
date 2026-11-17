//! The tick that passes liquid between the holders. A holder that is awake takes a turn: it
//! works out the push coming into it from its neighbours, hands liquid toward each side it
//! reaches, where a holder reaches back and stands lower (`flow`), and tells its neighbours its
//! head. When nothing changed it sleeps until a neighbour wakes it, by changing or handing it
//! liquid; so a settled line costs nearly nothing, and pressure spreads a block a tick with no
//! walk over the whole line.
//!
//! Every block of a tank takes a turn, and every turn asks for the holders around it. Which
//! blocks make a holder and what it touches stay as they are during a tick, so `Found` works
//! each out once a tick, however many turns ask.

use std::collections::HashMap;
use std::rc::Rc;

use weft_sdk::{SystemBatch, World};

use super::flow::LOSS;
use super::vessel::{Holders, Kind, Vessel};
use super::{Held, MOST_A_TURN, NO_LIQUID, flow};
use crate::lib::{Pos, TypedWorld};
use crate::liquid_tube::LIQUID_TUBE;

/// A holder as met at one of its blocks: `id`, that block's entity, keeps the head it told
/// there. A tank tells all its blocks the same head, but blocks it joined since may keep another.
#[derive(Clone)]
struct Met {
    vessel: Rc<Vessel>,
    id: u64,
}

/// A side of a holder that touches another: its block there, and the holder beside it, none
/// while that has no block entity yet.
struct Side {
    block: Pos,
    beside: Option<Met>,
}

/// The holders looked up this tick, by each of their blocks, and the sides of each.
#[derive(Default)]
struct Found {
    vessels: HashMap<Pos, Option<Met>>,
    sides: HashMap<Pos, Rc<[Side]>>,
}

impl Found {
    /// The holder whose block is at `pos`, if one is: `find` looks it up the first time one of
    /// its blocks is asked for.
    fn at(&mut self, pos: Pos, find: impl FnOnce(Pos) -> Option<Vessel>) -> Option<Met> {
        if let Some(known) = self.vessels.get(&pos) {
            return known.clone();
        }
        self.vessels.insert(pos, None);
        let vessel = Rc::new(find(pos)?);
        for (id, block) in vessel.blocks() {
            let vessel = Rc::clone(&vessel);
            self.vessels.insert(block, Some(Met { vessel, id }));
        }
        self.vessels.get(&pos).cloned().flatten()
    }

    /// Every side of `vessel`, the holder with a block at `pos`, that touches another holder.
    fn sides(&mut self, world: &World, pos: Pos, vessel: &Vessel) -> Rc<[Side]> {
        if let Some(known) = self.sides.get(&pos) {
            return Rc::clone(known);
        }
        let mut sides = Vec::new();
        for (block, dir) in vessel.sides(world) {
            let beside = self.at(block.toward(dir), |pos| Vessel::at(world, pos));
            sides.push(Side { block, beside });
        }
        let sides: Rc<[Side]> = sides.into();
        for (_, block) in vessel.blocks() {
            self.sides.insert(block, Rc::clone(&sides));
        }
        sides
    }
}

/// Every awake holder's turn, then a liquid tube shows its fill.
pub fn tick(batch: &mut SystemBatch<Held>) {
    let world = batch.world();
    if batch.values().iter().all(|held| held.asleep != 0) {
        return;
    }
    let ids = batch.ids().to_vec();
    let mut holders = Holders::of_batch(world, &ids, batch.values());
    let mut found = Found::default();
    for &id in &ids {
        if holders.get(id).asleep != 0 {
            continue;
        }
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        turn(&mut holders, &mut found, id, pos);
        // A timber liquid tube shows its own, after its pump (`timber_liquid_tube`).
        if world.block_typed(pos).is(&LIQUID_TUBE) {
            let mut held = holders.get(id);
            crate::liquid_tube::show(&world, pos, &mut held, NO_LIQUID);
            holders.set(id, held);
        }
    }
}

/// The turn of the holder `id` at `pos`: liquid toward the sides of its block, then its head
/// told to its neighbours. With nothing moved or changed, it sleeps, as does a block of a big
/// tank that touches nothing.
fn turn(holders: &mut Holders, found: &mut Found, id: u64, pos: Pos) {
    let world = *holders.world();
    let Some(me) = found.at(pos, |pos| Vessel::at(&world, pos)) else {
        sleep(holders, id);
        return;
    };
    let me = me.vessel;
    let sides = found.sides(&world, pos, &me);
    let mut own = sides.iter().filter(|side| side.block == pos).peekable();
    if own.peek().is_none() {
        sleep(holders, id);
        return;
    }
    let neighbours = || sides.iter().filter_map(|side| side.beside.as_ref());
    let rises = |vessel: &Vessel| super::rises(vessel.held(holders).liquid);
    let mine = rises(&me);
    let heads = neighbours().map(|met| (rises(&met.vessel), met.vessel.head_now(holders, met.id)));
    let pushed = push_on(mine, heads);
    let mut moved = false;
    for to in own.filter_map(|side| side.beside.as_ref()) {
        moved |= pass(holders, &me, pushed, to);
    }
    let held = me.held(holders);
    let head = me.level(held.liquid, held.amount, pushed).head;
    let changed = head != holders.get(id).head;
    if changed {
        me.tell(holders, head);
    }
    if moved || changed {
        for met in neighbours() {
            met.vessel.wake(holders);
        }
    } else {
        sleep(holders, id);
    }
}

/// The push coming into a holder from beside: the highest head of its `neighbours`, each with
/// whether it holds steam, less `LOSS`. Steam's heads have no height to them (`flow::pressure`),
/// so only neighbours on the same side of that, steam or not as the holder's own liquid, push it.
fn push_on(rises: bool, neighbours: impl Iterator<Item = (bool, i64)>) -> i64 {
    neighbours
        .filter(|&(theirs, _)| theirs == rises)
        .map(|(_, head)| head - LOSS)
        .max()
        .unwrap_or(i64::MIN)
}

fn sleep(holders: &mut Holders, id: u64) {
    let mut held = holders.get(id);
    held.asleep = 1;
    holders.set(id, held);
}

/// Hands from `from`, pushed by `pushed`, to `to` what evens out their heads; whether any went.
/// A holder that pushes never takes: liquid run back into it would only be pushed out again. A
/// driven timber liquid tube hands nothing to a tank: it pulls from there. An intake or a
/// grinding mill hands nothing on, and takes only what its machine uses.
fn pass(holders: &mut Holders, from: &Vessel, pushed: i64, to: &Met) -> bool {
    let (to, told) = (&*to.vessel, holders.get(to.id).head);
    let into_source = from.kind() == Kind::Driven && matches!(to, Vessel::Tank(_));
    if to.kind().pushes() || into_source || from.kind().keeps() {
        return false;
    }
    let (giving, taking) = (from.held(holders), to.held(holders));
    let admitted = to.admits(giving.liquid) && taking.takes(giving.liquid, to.capacity());
    if giving.is_empty() || !admitted {
        return false;
    }
    // A dribble of another liquid counts for nothing: it goes when this comes in.
    let had = if taking.liquid == giving.liquid {
        taking.amount
    } else {
        0
    };
    let higher = from.level(giving.liquid, giving.amount, pushed);
    let lower = to.level(giving.liquid, had, told);
    let room = to.capacity().saturating_sub(had);
    let amount = flow(higher, lower, MOST_A_TURN)
        .min(room)
        .min(giving.amount);
    if amount == 0 {
        return false;
    }
    from.take(holders, amount);
    to.put(holders, giving.liquid, amount);
    true
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

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
}
