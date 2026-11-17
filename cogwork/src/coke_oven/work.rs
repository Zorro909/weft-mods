//! Every coke oven's tick, at its door: check the box behind it, hand on its coke and sulfur by
//! its front, cook its coal for the ticks the call stands for (`batch`), and let its creosote
//! down into its drains; then show it: the door glowing and smoke over the top while it cooks.

use weft_sdk::{Item, ItemStack, Output, StackData, SystemBatch, Vec3, World};

use super::batch::Done;
use super::{COKE, COKE_OVEN, Coking, SHAPE, SMOKE, SULFUR, Status, part_at};
use crate::brickwork::{self, Layout};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid::{Holders, LiquidKey, Vessel, key_named};

/// Ticks between puffs of smoke while it cooks, and the puffs each time.
const SMOKE_EVERY: u64 = 10;
const PUFFS: u32 = 2;
/// Blocks above the door its smoke comes out at: just over the top of the box.
const SMOKE_HEIGHT: f64 = 2.05;

pub fn tick(batch: &mut SystemBatch<Coking>) {
    let world = batch.world();
    for (id, coking, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&COKE_OVEN) {
            continue;
        }
        let front = crate::machine::front(&world, pos);
        let is_air = |at| world.block_typed(at) == BlockState::AIR;
        match brickwork::check(pos, front, &SHAPE, |at| part_at(&world, at), is_air) {
            Ok(layout) => {
                crate::store::eject_toward(&world, pos, &[front]);
                let status = cook(&world, id, pos, coking, ticks);
                coking.set_status(status);
                drain(&world, pos, &layout);
            }
            Err(problem) => coking.set_unbuilt(problem),
        }
        show(&world, pos, coking.status() == Status::Coking);
    }
}

/// The key of creosote, as holders keep liquids.
fn creosote() -> LiquidKey {
    key_named("cogwork:creosote")
}

/// Cooks the coal in the input of the coke oven `id` at `pos` for `ticks`.
fn cook(world: &World, id: u64, pos: Pos, coking: &mut Coking, ticks: u32) -> Status {
    let input = world.inventory(id).group("input");
    let coal = input.get(0).count;
    coking.cook(ticks, coal, |done| finish(world, id, pos, done))
}

/// Puts away a batch the oven at `pos` has cooked: its coke and sulfur into the output and its
/// creosote into the door, then takes its coal out of the input; or says what has no room.
fn finish(world: &World, id: u64, pos: Pos, done: Done) -> Result<(), Status> {
    let Some(door) = Vessel::at(world, pos) else {
        return Err(Status::NoDrain);
    };
    let mut holders = Holders::outside(*world);
    let held = door.held(&holders);
    let room = door.capacity().saturating_sub(held.amount);
    if !held.takes(creosote(), door.capacity()) || room < done.creosote {
        return Err(Status::NoDrain);
    }
    let made = [stack_of(&COKE, done.coal), stack_of(&SULFUR, done.sulfur)];
    let output = world.inventory(id).group("output");
    if !crate::store::insert_all(output, &made) {
        return Err(Status::Full);
    }
    door.put(&mut holders, creosote(), done.creosote);
    let input = world.inventory(id).group("input");
    input.set(0, input.get(0).shrink(done.coal));
    Ok(())
}

/// `count` of `item`, nothing for none.
fn stack_of(item: &Item, count: u32) -> ItemStack {
    if count == 0 {
        ItemStack::default()
    } else {
        item.stack(count)
    }
}

/// Lets the creosote of the door at `pos` down into the drains of `layout`, as far as they
/// have room.
fn drain(world: &World, pos: Pos, layout: &Layout) {
    let Some(door) = Vessel::at(world, pos) else {
        return;
    };
    let mut holders = Holders::outside(*world);
    for &port in &layout.ports {
        let held = door.held(&holders);
        let Some(drain) = Vessel::at(world, port).filter(|_| !held.is_empty()) else {
            continue;
        };
        let has = drain.held(&holders);
        if !has.takes(held.liquid, drain.capacity()) {
            continue;
        }
        let moved = held.amount.min(drain.capacity().saturating_sub(has.amount));
        if moved > 0 {
            door.take(&mut holders, moved);
            drain.put(&mut holders, held.liquid, moved);
        }
    }
}

/// Shows the oven at `pos` cooking or not: its door glowing, and smoke over its top.
fn show(world: &World, pos: Pos, cooking: bool) {
    crate::machine::show_working(world, pos, cooking);
    if !cooking || !world.game_time().is_multiple_of(SMOKE_EVERY) {
        return;
    }
    let front = crate::machine::front(world, pos);
    let top = pos.toward(front.opposite()).floor_middle() + Vec3::new(0.0, SMOKE_HEIGHT, 0.0);
    let burst = world.spawn_particles(&SMOKE, top).count(PUFFS);
    drop(
        burst
            .spread(0.3)
            .velocity(Vec3::new(0.0, 0.3, 0.0))
            .speed(0.1),
    );
}
