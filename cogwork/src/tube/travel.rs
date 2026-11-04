//! Moving items along: every tube of a region in one system call. Items reaching the far face
//! of a tube are handed on after all tubes have moved, into the next tube's slots in the same
//! batch, into a machine, or out into the world where nothing is.

use std::collections::HashMap;

use weft_sdk::*;

use super::route::{Route, route};
use super::{BASE_SPEED, GILDED_SPEED, Motion, NONE, SLOTS, Tube, free_slot, kind, visual};
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
    let index: HashMap<u64, usize> = ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();
    let mut handoffs = Vec::new();
    for (i, tube) in tubes.iter_mut().enumerate() {
        let Some(st) = w.entity(ids[i]) else { continue };
        let pos = Pos::from(st.block_pos());
        if tube.kind == kind::TIMBER {
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
    let m = &mut tube.moving[slot];
    if stack.is_empty() {
        visual::remove(w, m.visual);
        *m = Motion::default();
        return false;
    }
    if m.item != stack.item || m.speed == 0.0 {
        // Put in by someone else (another mod, the excavator): it starts at the hub.
        visual::remove(w, m.visual);
        *m = Motion {
            item: stack.item,
            visual: 0,
            pos: 0.5,
            speed: BASE_SPEED,
            from: NONE,
            to: NONE,
        };
    }
    m.speed = if tube.kind == kind::GILDED {
        GILDED_SPEED
    } else {
        (m.speed * 0.98).max(BASE_SPEED)
    };
    if m.pos < 0.5 || m.to != NONE {
        m.pos = (m.pos + m.speed).min(if m.to == NONE { 0.5 } else { 1.0 });
    }
    if m.pos >= 0.5 && m.to == NONE {
        decide(w, id, pos, tube, slot, stack);
    }
    let m = &mut tube.moving[slot];
    if m.item == 0 {
        return false;
    }
    visual::show(w, pos, m, stack);
    m.pos >= 1.0
}

/// At the hub: where the item goes next.
fn decide(w: &World, id: u64, pos: Pos, tube: &mut Tube, slot: usize, stack: ItemStack) {
    let m = &mut tube.moving[slot];
    match route(w, id, pos, tube.kind, m.from, stack) {
        Route::Out(face) => m.to = face,
        Route::Wait => {}
        Route::Destroy => {
            w.play_sound(&crate::sounds::TUBE_VOID, pos.center(), 1.0);
            w.inventory(id).set(slot as u32, ItemStack::EMPTY);
            visual::remove(w, m.visual);
            *m = Motion::default();
        }
        Route::Spill => {
            spill(w, pos, NONE, stack);
            w.inventory(id).set(slot as u32, ItemStack::EMPTY);
            visual::remove(w, m.visual);
            *m = Motion::default();
        }
    }
}

/// Drops `stack` into the world out of the tube's `face` (its hub for `NONE`).
fn spill(w: &World, pos: Pos, face: u32, stack: ItemStack) {
    let at = if face == NONE {
        pos
    } else {
        pos.neighbor(face)
    };
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
    let stack = w.inventory(id).get(h.slot as u32);
    let next = pos.neighbor(m.to);
    let other = w.block_entity(next);
    if let Some(&j) = index.get(&other) {
        let moved = into_tube(w, next, other, &mut tubes[j], m, stack);
        if moved {
            w.inventory(id).set(h.slot as u32, ItemStack::EMPTY);
            tubes[h.tube].moving[h.slot] = Motion::default();
        }
        return moved;
    }
    if other != 0 && super::is_tube(w, next) {
        // A tube outside this batch (another region): the item waits.
        return false;
    }
    let left = if other != 0 {
        crate::store::insert(w, other, stack, false)
    } else {
        stack
    };
    let m = &mut tubes[h.tube].moving[h.slot];
    if other == 0 {
        spill(w, pos, m.to, left);
    } else if !left.is_empty() {
        // Full: back the way it came, to be sent somewhere else.
        w.inventory(id).set(h.slot as u32, left);
        (m.from, m.to, m.pos) = (m.to, NONE, 0.0);
        return false;
    }
    w.inventory(id).set(h.slot as u32, ItemStack::EMPTY);
    visual::remove(w, m.visual);
    *m = Motion::default();
    true
}

/// Moves the item `m` into a free slot of the tube `id` at `pos`, entering by the face it came
/// toward; false when that tube is full or doesn't connect back.
fn into_tube(w: &World, pos: Pos, id: u64, tube: &mut Tube, m: Motion, stack: ItemStack) -> bool {
    let face = m.to ^ 1;
    if !super::connects_to_tube(w, pos, face) {
        return false;
    }
    let Some(free) = free_slot(w, id, tube) else {
        return false;
    };
    w.inventory(id).set(free as u32, stack);
    tube.moving[free] = Motion {
        pos: (m.pos - 1.0).max(0.0),
        from: face,
        to: NONE,
        ..m
    };
    true
}
