//! An excavator's tick: plan the area, build the frame, then mine, and pass on what it mined.

use weft_sdk::*;

use super::mine::{Mined, mine_block, opens_into};
use super::{Dig, EXCAVATOR, GANTRY_FRAME, area, rig, stage, status};

/// Energy to put up one block of frame.
const FRAME_COST: i64 = 100;
/// The most energy it puts into its work per tick, however many engines drive it: stone takes
/// at least 8 ticks.
const DRAW_RATE: i64 = 120;
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
        eject(&w, id, pos);
        match d.stage {
            stage::FRAME => build_frame(&w, id, d),
            stage::MINE => mine_next(&w, id, d),
            _ => {}
        }
        rig::update(&w, id, d);
        let working = matches!(d.status, status::FRAME | status::MINING);
        if w.prop(pos, "working") != Some(if working { "true" } else { "false" }) {
            w.set_prop(pos, "working", if working { "true" } else { "false" });
        }
    }
}

fn start(w: &World, pos: [i32; 3], d: &mut Dig) {
    let front = w
        .prop(pos, "facing")
        .and_then(Dir::from_name)
        .unwrap_or(Dir::North);
    (d.lo, d.hi) = area::plan(w, pos, front);
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
        if GANTRY_FRAME.is(state)
            || HARDNESS.on_block(state).is_some_and(|h| h < 0.0)
            || EXCAVATOR.is(state)
        {
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

/// Puts stored energy into a job of `cost` until `d.spent` covers it; true once it does. The
/// caller starts `spent` over when the job is done.
pub fn pay(w: &World, id: u64, d: &mut Dig, cost: i64) -> bool {
    let drawn = w
        .energy(id)
        .map_or(0, |e| e.stored)
        .min((cost - d.spent).clamp(0, DRAW_RATE));
    if let Some(e) = w.energy_mut(id) {
        e.stored -= drawn;
    }
    d.spent += drawn;
    if d.spent >= cost {
        return true;
    }
    if drawn == 0 {
        d.status = status::NO_POWER;
    }
    false
}

/// Mines toward the next block that is there to mine.
fn mine_next(w: &World, id: u64, d: &mut Dig) {
    let Some(target) = find_target(w, d) else {
        return;
    };
    let state = w.get(target);
    if !super::mine::drill_to(d, target) {
        d.status = status::MINING;
        return;
    }
    match mine_block(w, id, d, target, state) {
        Mined::Done | Mined::Skipped => d.cursor = area::next(d.lo, d.hi, target),
        Mined::Waiting(s) => d.status = s,
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

/// Hands one stack of what it mined to a tube or machine beside it.
fn eject(w: &World, id: u64, pos: [i32; 3]) {
    let out = w.inventory(id).group("output");
    let Some(slot) = (0..out.len()).find(|&i| !out.get(i).is_empty()) else {
        return;
    };
    let mut stack = out.get(slot);
    for face in 0..6 {
        let other = w.block_entity(crate::tube::step(pos, face));
        if other == 0 || other == id {
            continue;
        }
        stack = if w.component(other, &crate::tube::TUBE).is_some() {
            crate::tube::inject(w, other, stack)
        } else {
            crate::store::insert(w, other, stack, false)
        };
        if stack.is_empty() {
            break;
        }
    }
    out.set(slot, stack);
}
