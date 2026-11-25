//! Every refinery's tick, at its controller: check the column behind it, hand its sulfur on by
//! its front, then run it tick by tick for the ticks the call stands for (`distil`): each tick
//! is heated, cooled and fed only by what its heaters, condensers and controller hold then, so
//! missed time is paid for or cools it, never run for free. Then show it: the shell formed or
//! not, the controller lit while it distils, vapour and a hiss while it vents, and its rumble.
//!
//! A built column shows its shell formed every tick; one not built shows its whole box plain
//! once: when it stops being built, and on the first tick of a new controller.

mod distil;

use weft_sdk::{Output, SystemBatch, Vec3, World};

use super::column::{self, Layout, SHAPE};
use super::heat;
use super::parts::{part_at, show_formed};
use super::products::Product;
use super::{REFINERY, Refinery, Status, VAPOUR};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::sounds;

/// The most ticks a call runs one by one; the time before them the column is left alone.
const CATCH_UP_MOST: u32 = 1_200;
/// Ticks between rumbles while it works, and between puffs of vapour and hisses while it vents.
const RUMBLE_EVERY: u64 = 60;
const VENT_EVERY: u64 = 5;
const HISS_EVERY: u64 = 30;

pub fn tick(batch: &mut SystemBatch<Refinery>) {
    let world = batch.world();
    for (id, refinery, ticks) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&REFINERY) {
            continue;
        }
        crate::machine::start_meter(&world, id);
        let front = crate::machine::front(&world, pos);
        let shown_built = refinery.status() != Status::Unbuilt || refinery.checked == 0;
        refinery.checked = 1;
        let is_air = |at| world.block_typed(at) == BlockState::AIR;
        match column::check(pos, front, |at| part_at(&world, at), is_air) {
            Ok(layout) => {
                crate::store::eject_toward(&world, pos, &[front]);
                let run = ticks.min(CATCH_UP_MOST);
                refinery.warmth = heat::left_alone(refinery.warmth, ticks - run);
                for _ in 0..run {
                    distil::tick(&world, id, refinery, &layout);
                }
                form(&world, &layout.shell, true);
                show(&world, &layout, refinery);
            }
            Err(problem) => {
                refinery.warmth = heat::left_alone(refinery.warmth, ticks);
                refinery.set_unbuilt(problem);
                if shown_built {
                    unform(&world, pos, front);
                }
            }
        }
        let working = matches!(
            refinery.status(),
            Status::Distilling | Status::TooCold | Status::TooHot
        );
        crate::machine::show_working(&world, pos, working);
        if working && world.game_time().is_multiple_of(RUMBLE_EVERY) {
            world.play_sound(&sounds::REFINERY_RUMBLE, pos.add_y(2).center(), 1.0);
        }
    }
}

/// Shows a column venting its gas, too hot or with no gas tap: vapour out of the tops of its
/// condensers, and a hiss.
fn show(world: &World, layout: &Layout, refinery: &Refinery) {
    let time = world.game_time();
    let no_gas_tap = layout.taps_of(Product::Gas).is_empty();
    let venting = match refinery.status() {
        Status::TooHot => true,
        Status::Distilling => no_gas_tap,
        _ => false,
    };
    if !venting || !time.is_multiple_of(VENT_EVERY) {
        return;
    }
    for &condenser in &layout.condensers {
        let top = condenser.floor_middle() + Vec3::new(0.0, 1.05, 0.0);
        let burst = world.spawn_particles(&VAPOUR, top).count(3);
        drop(
            burst
                .spread(0.3)
                .velocity(Vec3::new(0.0, 0.35, 0.0))
                .speed(0.1),
        );
    }
    if time.is_multiple_of(HISS_EVERY)
        && let Some(&condenser) = layout.condensers.first()
    {
        world.play_sound(&sounds::REFINERY_VENT, condenser.center(), 1.0);
    }
}

/// Shows each part at `shell` formed, or not.
fn form(world: &World, shell: &[Pos], formed: bool) {
    for &part in shell {
        show_formed(world, part, formed);
    }
}

/// Shows every block of the box behind the controller at `pos`, facing `front`, as it is when
/// the column is not built.
fn unform(world: &World, pos: Pos, front: weft_sdk::Dir) {
    let middle = pos.toward(front.opposite());
    let side = front.clockwise();
    let mut blocks = Vec::new();
    for layer in 0..SHAPE.height {
        for depth in -1..=1 {
            for across in -1..=1 {
                let row = middle.add_y(layer).toward_by(front, depth);
                blocks.push(row.toward_by(side, across));
            }
        }
    }
    form(world, &blocks, false);
}

/// The controller at `pos` was broken, its block `state` before: its column's casing shows it
/// is not built any more.
pub fn broken(world: &World, pos: Pos, state: BlockState) {
    unform(world, pos, crate::machine::front_of(state));
}
