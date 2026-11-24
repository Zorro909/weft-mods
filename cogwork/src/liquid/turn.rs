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
use super::{Held, NO_LIQUID, flow};
use crate::lib::{Pos, TypedWorld};

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
        if crate::liquid_tube::shows_own_fill(world.block_typed(pos)) {
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
    let heads = neighbours().map(|met| {
        let head = met.vessel.head_now(holders, met.id, me.kind());
        (rises(&met.vessel), head)
    });
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

/// Hands from `from`, pushed by `pushed`, to `to` what evens out their heads (`share`); whether
/// any went.
fn pass(holders: &mut Holders, from: &Vessel, pushed: i64, to: &Met) -> bool {
    let (to, told) = (&*to.vessel, holders.get(to.id).head);
    let (giving, taking) = (from.held(holders), to.held(holders));
    let amount = share(Party::of(from, giving, pushed), Party::of(to, taking, told));
    if amount == 0 {
        return false;
    }
    from.take(holders, amount);
    to.put(holders, giving.liquid, amount);
    true
}

/// A holder on either end of a turn's share: what it is, what it keeps, and the push into it.
struct Party<'v> {
    vessel: &'v Vessel,
    held: Held,
    pushed: i64,
}

impl<'v> Party<'v> {
    fn of(vessel: &'v Vessel, held: Held, pushed: i64) -> Party<'v> {
        Party {
            vessel,
            held,
            pushed,
        }
    }
}

/// Millibuckets that go from `from` to `to` in a turn: what evens out their heads, at most what
/// `from` has, what `to` has room for, and a turn's worth of the faster of the two (a pressure
/// tube's is twice a tube's). A holder that pushes never takes: liquid run back into it would only
/// be pushed out again. A driven timber liquid tube hands nothing to a tank: it pulls from there.
/// A machine's holder hands nothing on, and takes only what its machine uses. Steam never takes
/// the place of another liquid, nor another liquid that of steam (`Held::takes`).
fn share(from: Party, to: Party) -> u32 {
    let (giving, taking) = (from.held, to.held);
    let (into, kind) = (to.vessel, to.vessel.kind());
    let into_source = from.vessel.kind() == Kind::Driven && matches!(into, Vessel::Tank(_));
    if kind.pushes() || into_source || from.vessel.kind().keeps() {
        return 0;
    }
    let admitted = into.admits(giving.liquid) && taking.takes(giving.liquid, into.capacity());
    if giving.is_empty() || !admitted {
        return 0;
    }
    // A dribble of another liquid counts for nothing: it goes when this comes in.
    let had = if taking.liquid == giving.liquid {
        taking.amount
    } else {
        0
    };
    let higher = from
        .vessel
        .level_into(giving.liquid, giving.amount, from.pushed, kind);
    let lower = into.level(giving.liquid, had, to.pushed);
    let room = into.capacity().saturating_sub(had);
    let most = from.vessel.kind().most_a_turn().max(kind.most_a_turn());
    flow(higher, lower, most).min(room).min(giving.amount)
}

#[cfg(test)]
mod tests;
