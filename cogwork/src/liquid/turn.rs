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
use super::{Held, Level, MOST_A_TURN, flow};
use crate::lib::{Pos, TypedWorld};
use crate::liquid_tube::is_liquid_tube;

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
        if is_liquid_tube(world.block_typed(pos)) {
            let mut held = holders.get(id);
            crate::liquid_tube::show(&world, pos, &mut held);
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
    let pushed = neighbours()
        .map(|met| met.vessel.head_now(holders, met.id) - LOSS)
        .max()
        .unwrap_or(i64::MIN);
    let mut moved = false;
    for to in own.filter_map(|side| side.beside.as_ref()) {
        moved |= pass(holders, &me, pushed, to);
    }
    let head = me.head(me.held(holders).amount, pushed);
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

fn sleep(holders: &mut Holders, id: u64) {
    let mut held = holders.get(id);
    held.asleep = 1;
    holders.set(id, held);
}

/// Hands from `from`, pushed by `pushed`, to `to` what evens out their heads; whether any went.
/// A holder that pushes never takes: liquid run back into it would only be pushed out again. A
/// driven timber liquid tube hands nothing to a tank: it pulls from there.
fn pass(holders: &mut Holders, from: &Vessel, pushed: i64, to: &Met) -> bool {
    let (to, told) = (&*to.vessel, holders.get(to.id).head);
    let into_source = from.kind() == Kind::Driven && matches!(to, Vessel::Tank(_));
    if to.kind().pushes() || into_source {
        return false;
    }
    let (giving, taking) = (from.held(holders), to.held(holders));
    if giving.is_empty() || !taking.takes(giving.liquid, to.capacity()) {
        return false;
    }
    // A dribble of another liquid counts for nothing: it goes when this comes in.
    let had = if taking.liquid == giving.liquid {
        taking.amount
    } else {
        0
    };
    let higher = Level {
        head: from.head(giving.amount, pushed),
        per_block: from.per_block(),
    };
    let lower = Level {
        head: to.head(had, told),
        per_block: to.per_block(),
    };
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
        assert_eq!(driven.head(500, 0), lift, "half full, it pushes");
        assert!(tube(1, pos).head(500, 0) < lift, "undriven, it doesn't");
        assert!(
            Kind::Driven.pushes() && !Kind::Tube.pushes(),
            "nor takes when it does"
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
