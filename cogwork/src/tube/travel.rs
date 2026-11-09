//! Moving items along: every tube of a region in one system call. Items reaching the far face
//! of a tube are handed on after all tubes have moved, into the next tube's slots in the same
//! batch, into a machine, or out into the world where nothing is.

use std::collections::HashMap;

use weft_sdk::*;

use super::motion::{BASE_SPEED, GILDED_SPEED};
use super::route::{Route, route};
use super::{Kind, Motion, SLOTS, Tube, clear_slot, free_slot, visual};
use crate::lib::Pos;

/// An item at the far face of a tube, to go on through it.
struct Handoff {
    tube: usize,
    slot: usize,
}

pub fn tick_tubes(b: &mut SystemBatch<Tube>) {
    let w = b.world();
    let ids = b.ids().to_vec();
    let tubes = b.values();
    let index: HashMap<u64, usize> = ids.iter().copied().zip(0..).collect();
    let mut handoffs = Vec::new();
    for (i, tube) in tubes.iter_mut().enumerate() {
        let Some(st) = w.entity(ids[i]) else { continue };
        let pos = Pos::from(st.block_pos());
        if tube.kind() == Kind::Timber {
            super::extract::pull(&w, ids[i], pos, tube);
        }
        for slot in 0..SLOTS {
            if advance(&w, ids[i], pos, tube, slot) {
                handoffs.push(Handoff { tube: i, slot });
            }
        }
    }
    let mut passed_on = vec![false; tubes.len()];
    for h in handoffs {
        let tube = h.tube;
        passed_on[tube] |= hand_on(&w, &ids, &index, tubes, h);
    }
    for (i, tube) in tubes.iter_mut().enumerate() {
        if let Some(st) = w.entity(ids[i]) {
            super::overflow::check(&w, ids[i], st.block_pos().into(), tube, passed_on[i]);
        }
    }
}

/// Moves one item a tick along and shows it; true when it waits at the far face.
fn advance(w: &World, id: u64, pos: Pos, tube: &mut Tube, slot: usize) -> bool {
    let stack = w.inventory(id).get(slot as u32);
    let kind = tube.kind();
    let m = &mut tube.moving[slot];
    if stack.is_empty() {
        visual::remove(w, m.visual);
        *m = Motion::default();
        return false;
    }
    if m.item != stack.item || m.speed == 0.0 {
        // Put in by someone else (another mod, the excavator): it starts at the hub.
        visual::remove(w, m.visual);
        *m = Motion::at_hub(stack.item);
    }
    m.speed = if kind == Kind::Gilded {
        GILDED_SPEED
    } else {
        (m.speed * 0.98).max(BASE_SPEED)
    };
    m.step();
    if m.reached_hub() && m.leaves_by().is_none() {
        decide(w, id, pos, tube, slot, stack);
    }
    let m = &mut tube.moving[slot];
    if m.is_none() {
        return false;
    }
    visual::show(w, pos, m, stack);
    m.at_far_face()
}

/// At the hub: where the item goes next.
fn decide(w: &World, id: u64, pos: Pos, tube: &mut Tube, slot: usize, stack: ItemStack) {
    let from = tube.moving[slot].came_by();
    match route(w, id, pos, tube.kind(), from, stack) {
        Route::Out(d) => tube.moving[slot].leave_by(d),
        Route::Wait => {}
        Route::Destroy => {
            w.play_sound(&crate::sounds::TUBE_VOID, pos.center(), 1.0);
            clear_slot(w, id, tube, slot);
        }
        Route::Spill => {
            spill(w, pos, None, stack);
            clear_slot(w, id, tube, slot);
        }
    }
}

/// Drops `stack` into the world out of the tube's `side` (its hub for `None`).
fn spill(w: &World, pos: Pos, side: Option<Dir>, stack: ItemStack) {
    let at = side.map_or(pos, |d| pos.toward(d));
    w.spawn_item(at.center(), stack);
}

/// Sends an item waiting at its tube's far face on: into the next tube, into a machine, or out.
/// True when it left the tube.
fn hand_on(
    w: &World,
    ids: &[u64],
    index: &HashMap<u64, usize>,
    tubes: &mut [Tube],
    h: Handoff,
) -> bool {
    let id = ids[h.tube];
    let Some(pos) = w.entity(id).map(|st| Pos::from(st.block_pos())) else {
        return false;
    };
    let m = tubes[h.tube].moving[h.slot];
    let Some(out) = m.leaves_by() else {
        return false;
    };
    let stack = w.inventory(id).get(h.slot as u32);
    let next = pos.toward(out);
    let other = w.block_entity(next);
    if let Some(&j) = index.get(&other) {
        if !into_tube(w, next, other, &mut tubes[j], m, stack) {
            return false;
        }
        // Its picture went along: only the slot is emptied.
        w.inventory(id).set(h.slot as u32, ItemStack::EMPTY);
        tubes[h.tube].moving[h.slot] = Motion::default();
        return true;
    }
    let tube = &mut tubes[h.tube];
    if other == 0 {
        spill(w, pos, Some(out), stack);
    } else if super::is_tube(w, next) {
        // A tube outside this batch (another region): the item waits.
        return false;
    } else if !into_machine(w, id, tube, h.slot, other, stack) {
        return false;
    }
    clear_slot(w, id, tube, h.slot);
    true
}

/// Puts `stack`, in `slot` of the tube `id`, into the machine `machine`. What doesn't fit stays
/// in the slot and turns back the way it came, to be sent somewhere else; false then.
fn into_machine(
    w: &World,
    id: u64,
    tube: &mut Tube,
    slot: usize,
    machine: u64,
    stack: ItemStack,
) -> bool {
    let left = crate::store::insert(w, machine, stack, false);
    if left.is_empty() {
        return true;
    }
    w.inventory(id).set(slot as u32, left);
    tube.moving[slot].turn_back();
    false
}

/// Moves the item `m` into a free slot of the tube `id` at `pos`, entering by the side it came
/// toward; false when that tube is full or doesn't connect back.
fn into_tube(w: &World, pos: Pos, id: u64, tube: &mut Tube, m: Motion, stack: ItemStack) -> bool {
    let Some(side) = m.leaves_by().map(Dir::opposite) else {
        return false;
    };
    if !super::connects_to_tube(w, pos, side) {
        return false;
    }
    let Some(free) = free_slot(w, id, tube) else {
        return false;
    };
    w.inventory(id).set(free as u32, stack);
    tube.moving[free] = m.handed_on(side);
    true
}
