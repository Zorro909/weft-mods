//! Every blast furnace's tick, at its controller: check the box behind it, hand on its blooms
//! and slag by its front, smelt for the ticks the call stands for while a driven bellows blows
//! into its bottom layer (`smelt`), then show it: the tap hole glowing, sparks out of its top and
//! a roar while it smelts.

use weft_sdk::{Output, StackData, SystemBatch, Vec3, World};

use super::{BLAST_FURNACE, Blast, SHAPE, Status, part_at};
use crate::brickwork::{self, Layout};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::steel::{SLAG, SPARK, STEEL_BLOOM};

/// Ticks between roars while it smelts: the length of the sound.
const ROAR_EVERY: u64 = 60;
/// Ticks between bursts of sparks, and the sparks a burst.
const SPARKS_EVERY: u64 = 4;
const SPARKS: u32 = 5;
/// Blocks above the controller its open top is.
const TOP: f64 = 4.0;

pub fn tick(batch: &mut SystemBatch<Blast>) {
    let world = batch.world();
    for (id, blast, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&BLAST_FURNACE) {
            continue;
        }
        let front = crate::machine::front(&world, pos);
        let is_air = |at| world.block_typed(at) == BlockState::AIR;
        match brickwork::check(pos, front, &SHAPE, |at| part_at(&world, at), is_air) {
            Ok(layout) => {
                crate::store::eject_toward(&world, pos, &[front]);
                let status = smelt(&world, id, blast, ticks, blown(&world, &layout));
                blast.set_status(status);
            }
            Err(problem) => blast.set_unbuilt(problem),
        }
        show(&world, pos, blast.status() == Status::Smelting);
    }
}

/// Whether a driven bellows blows into a block of the bottom layer of `layout`.
fn blown(world: &World, layout: &Layout) -> bool {
    layout
        .bottom()
        .any(|pos| crate::bellows::driven_into(world, pos))
}

/// Smelts the iron and coke of the blast furnace `id` for `ticks`.
fn smelt(world: &World, id: u64, blast: &mut Blast, ticks: u32, blown: bool) -> Status {
    let inventory = world.inventory(id);
    let (input, fuel) = (inventory.group("input"), inventory.group("fuel"));
    let charges = input.get(0).count.min(fuel.get(0).count);
    blast.smelt(ticks, blown, charges, || {
        let made = [STEEL_BLOOM.stack(1), SLAG.stack(1)];
        if !crate::store::insert_all(inventory.group("output"), &made) {
            return false;
        }
        input.set(0, input.get(0).shrink(1));
        fuel.set(0, fuel.get(0).shrink(1));
        true
    })
}

/// Shows the furnace at `pos` smelting or not: its tap hole glowing, sparks out of its top, and
/// a roar.
fn show(world: &World, pos: Pos, smelting: bool) {
    crate::machine::show_working(world, pos, smelting);
    if !smelting {
        return;
    }
    let time = world.game_time();
    let front = crate::machine::front(world, pos);
    let middle = pos.toward(front.opposite());
    if time.is_multiple_of(ROAR_EVERY) {
        world.play_sound(&crate::sounds::BLAST_FURNACE, middle.center(), 1.0);
    }
    if time.is_multiple_of(SPARKS_EVERY) {
        let top = middle.floor_middle() + Vec3::new(0.0, TOP, 0.0);
        let burst = world.spawn_particles(&SPARK, top).count(SPARKS);
        drop(
            burst
                .spread(0.3)
                .velocity(Vec3::new(0.0, 0.25, 0.0))
                .speed(0.15),
        );
    }
}
