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

pub fn tick_tubes(batch: &mut SystemBatch<Tube>) {
    let world = batch.world();
    let ids = batch.ids().to_vec();
    let tubes = batch.values();
    let index: HashMap<u64, usize> = ids.iter().copied().zip(0..).collect();
    let mut handoffs = Vec::new();
    for (i, tube) in tubes.iter_mut().enumerate() {
        let Some(entity) = world.entity(ids[i]) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if tube.kind() == Kind::Timber {
            super::extract::pull(&world, ids[i], pos, tube);
        }
        for slot in 0..SLOTS {
            if advance(&world, ids[i], pos, tube, slot) {
                handoffs.push(Handoff { tube: i, slot });
            }
        }
    }
    let mut passed_on = vec![false; tubes.len()];
    for handoff in handoffs {
        let tube = handoff.tube;
        passed_on[tube] |= hand_on(&world, &ids, &index, tubes, handoff);
    }
    for (i, tube) in tubes.iter_mut().enumerate() {
        if let Some(entity) = world.entity(ids[i]) {
            super::overflow::check(
                &world,
                ids[i],
                entity.block_pos().into(),
                tube,
                passed_on[i],
            );
        }
    }
}

/// Moves one item a tick along and shows it; true when it waits at the far face.
fn advance(world: &World, id: u64, pos: Pos, tube: &mut Tube, slot: usize) -> bool {
    let stack = world.inventory(id).get(slot as u32);
    let kind = tube.kind();
    let motion = &mut tube.moving[slot];
    if stack.is_empty() {
        visual::remove(world, motion.visual);
        *motion = Motion::default();
        return false;
    }
    if motion.item != stack.item || motion.speed == 0.0 {
        // Put in by someone else (another mod, the excavator): it starts at the hub.
        visual::remove(world, motion.visual);
        *motion = Motion::at_hub(stack.item);
    }
    motion.speed = if kind == Kind::Gilded {
        GILDED_SPEED
    } else {
        (motion.speed * 0.98).max(BASE_SPEED)
    };
    motion.step();
    if motion.reached_hub() && motion.leaves_by().is_none() {
        decide(world, id, pos, tube, slot, stack);
    }
    let motion = &mut tube.moving[slot];
    if motion.is_none() {
        return false;
    }
    visual::show(world, pos, motion, stack);
    motion.at_far_face()
}

/// At the hub: where the item goes next.
fn decide(world: &World, id: u64, pos: Pos, tube: &mut Tube, slot: usize, stack: ItemStack) {
    let from = tube.moving[slot].came_by();
    match route(world, id, pos, tube.kind(), from, stack) {
        Route::Out(dir) => tube.moving[slot].leave_by(dir),
        Route::Wait => {}
        Route::Destroy => {
            world.play_sound(&crate::sounds::TUBE_VOID, pos.center(), 1.0);
            clear_slot(world, id, tube, slot);
        }
        Route::Spill => {
            spill(world, pos, None, stack);
            clear_slot(world, id, tube, slot);
        }
    }
}

/// Drops `stack` into the world out of the tube's `side` (its hub for `None`).
fn spill(world: &World, pos: Pos, side: Option<Dir>, stack: ItemStack) {
    let at = side.map_or(pos, |dir| pos.toward(dir));
    world.spawn_item(at.center(), stack);
}

/// Sends an item waiting at its tube's far face on: into the next tube, into a machine, or out.
/// True when it left the tube.
fn hand_on(
    world: &World,
    ids: &[u64],
    index: &HashMap<u64, usize>,
    tubes: &mut [Tube],
    handoff: Handoff,
) -> bool {
    let Handoff { tube: from, slot } = handoff;
    let id = ids[from];
    let Some(pos) = world.entity(id).map(|entity| Pos::from(entity.block_pos())) else {
        return false;
    };
    let motion = tubes[from].moving[slot];
    let Some(out) = motion.leaves_by() else {
        return false;
    };
    let stack = world.inventory(id).get(slot as u32);
    let next = pos.toward(out);
    let other = world.block_entity(next);
    if let Some(&neighbor) = index.get(&other) {
        if !into_tube(world, next, other, &mut tubes[neighbor], motion, stack) {
            return false;
        }
        // Its picture went along: only the slot is emptied.
        world.inventory(id).set(slot as u32, ItemStack::EMPTY);
        tubes[from].moving[slot] = Motion::default();
        return true;
    }
    let tube = &mut tubes[from];
    if other == 0 {
        spill(world, pos, Some(out), stack);
    } else if super::is_tube(world, next) {
        // A tube outside this batch (another region): the item waits.
        return false;
    } else if !into_machine(world, id, tube, slot, other, stack) {
        return false;
    }
    clear_slot(world, id, tube, slot);
    true
}

/// Puts `stack`, in `slot` of the tube `id`, into the machine `machine`. What doesn't fit stays
/// in the slot and turns back the way it came, to be sent somewhere else; false then.
fn into_machine(
    world: &World,
    id: u64,
    tube: &mut Tube,
    slot: usize,
    machine: u64,
    stack: ItemStack,
) -> bool {
    let left = crate::store::insert(world, machine, stack, false);
    if left.is_empty() {
        return true;
    }
    world.inventory(id).set(slot as u32, left);
    tube.moving[slot].turn_back();
    false
}

/// Moves the item `motion` into a free slot of the tube `id` at `pos`, entering by the side it came
/// toward; false when that tube is full or doesn't connect back.
fn into_tube(
    world: &World,
    pos: Pos,
    id: u64,
    tube: &mut Tube,
    motion: Motion,
    stack: ItemStack,
) -> bool {
    let Some(side) = motion.leaves_by().map(Dir::opposite) else {
        return false;
    };
    if !super::connects_to_tube(world, pos, side) {
        return false;
    }
    let Some(free) = free_slot(world, id, tube) else {
        return false;
    };
    world.inventory(id).set(free as u32, stack);
    tube.moving[free] = motion.handed_on(side);
    true
}
