//! An excavator's tick: plan the area, build the frame, then mine, and pass on what it mined.

use weft_sdk::*;

use super::mine::{Mined, block_cost, drill_to, mine_block, progress_point, sweep};
use super::power::{hold, pay};
use super::{Dig, EXCAVATOR, GANTRY_FRAME, area, rig, stage, status};
use crate::harvest::opens_into;

/// Energy to put up one block of frame.
const FRAME_COST: i64 = 100;
/// The most blocks looked at per tick for the next one to mine (air is skipped).
const SCAN: u32 = 512;
/// Ticks a layer may stay unloaded before the excavator takes it for the floor of the world.
const FLOOR_WAIT: u32 = 200;
/// The deepest layer it mines, like the floor of vanilla's worlds.
const FLOOR_Y: i32 = -64;

pub fn tick(b: &mut SystemBatch<Dig>) {
    let w = b.world();
    for (id, d, _ticks) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        let pos = st.block_pos();
        if !EXCAVATOR.is(w.get(pos)) {
            continue;
        }
        if d.stage == stage::NEW {
            start(&w, pos, d);
        }
        crate::store::eject(&w, id, pos);
        let current = d.stage;
        match current {
            stage::FRAME | stage::MINE if !hold(&w, id, d) => {}
            stage::FRAME => build_frame(&w, id, d),
            stage::MINE => mine_next(&w, id, d),
            _ => {}
        }
        rig::update(&w, id, d);
        let working = matches!(d.status, status::FRAME | status::MINING);
        crate::machine::show_working(&w, pos, working);
    }
}

fn start(w: &World, pos: [i32; 3], d: &mut Dig) {
    (d.lo, d.hi) = area::plan(w, pos, crate::machine::front(w, pos));
    d.stage = stage::FRAME;
    d.step = 0;
}

/// One block of frame a tick: mined first if something is in the way.
fn build_frame(w: &World, id: u64, d: &mut Dig) {
    let cells = area::frame_cells(d.lo, d.hi);
    while let Some(&c) = cells.get(d.step as usize) {
        let Some(state) = w.get_loaded(c) else {
            d.status = status::BLOCKED;
            return;
        };
        if frame_passes_over(state) {
            d.step += 1;
            continue;
        }
        if !opens_into(state) {
            if let Mined::Waiting(s) = mine_block(w, id, d, c, state) {
                d.status = s;
            }
            return;
        }
        d.status = status::FRAME;
        if pay(w, id, d, FRAME_COST) {
            w.set(c, GANTRY_FRAME.id());
            d.spent = 0;
            d.step += 1;
        }
        return;
    }
    d.stage = stage::MINE;
    d.cursor = area::first(d.lo, d.hi);
    d.head = [
        d.cursor[0] as f32 + 0.5,
        d.hi[1] as f32,
        d.cursor[2] as f32 + 0.5,
    ];
}

/// Whether the frame leaves a block as it is: frame already, the excavator itself, or what
/// can't be broken.
fn frame_passes_over(state: u32) -> bool {
    GANTRY_FRAME.is(state)
        || EXCAVATOR.is(state)
        || HARDNESS.on_block(state).is_some_and(|h| h < 0.0)
}

/// Mines toward the next block that is there to mine.
fn mine_next(w: &World, id: u64, d: &mut Dig) {
    let Some(target) = find_target(w, d) else {
        return;
    };
    let state = w.get(target);
    let cost = block_cost(state);
    // Where the energy paid so far has the drill: into a block it has begun, it carries on from
    // there instead of going back to its edge.
    if !drill_to(w, id, d, progress_point(d.lo, target, d.spent, cost)) {
        return;
    }
    match mine_block(w, id, d, target, state) {
        Mined::Done => {
            d.head = sweep(d.lo, target, 1.0);
            d.cursor = area::next(d.lo, d.hi, target);
        }
        Mined::Skipped => d.cursor = area::next(d.lo, d.hi, target),
        Mined::Waiting(s) => {
            d.head = progress_point(d.lo, target, d.spent, cost);
            d.status = s;
        }
    }
}

/// The next block worth mining from the cursor on, skipping air, liquids and what can't be
/// broken; `None` while waiting or once finished.
fn find_target(w: &World, d: &mut Dig) -> Option<[i32; 3]> {
    for _ in 0..SCAN {
        let c = d.cursor;
        if c[1] < FLOOR_Y {
            finish(d);
            return None;
        }
        let Some(state) = w.get_loaded(c) else {
            d.status = status::BLOCKED;
            d.waited += 1;
            if d.waited > FLOOR_WAIT {
                finish(d);
            }
            return None;
        };
        d.waited = 0;
        if super::mine::worth_mining(state) {
            return Some(c);
        }
        d.cursor = area::next(d.lo, d.hi, c);
    }
    d.status = status::MINING;
    None
}

fn finish(d: &mut Dig) {
    d.stage = stage::DONE;
    d.status = status::DONE;
}
