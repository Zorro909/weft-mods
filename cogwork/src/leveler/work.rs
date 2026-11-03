//! A leveler's tick: plan the area, then clear it from the top layer down to its height, then
//! fill it from the deepest layer up to just below.

use weft_sdk::*;

use super::{LEVELER, Level, Status, stage};
use crate::harvest::{Harvest, breakable, cost, harvest, opens_into};

/// Layers cleared from the leveler's height up, and filled below it.
const CLEAR_HEIGHT: i32 = 16;
const FILL_DEPTH: i32 = 4;
/// The area when no markers say otherwise: 9 by 9 behind it.
const DEFAULT_SIZE: i32 = 9;
/// Energy to clear a block (a base and as much again per point of hardness), and to fill one.
const CLEAR_BASE: f32 = 100.0;
const CLEAR_PER_HARDNESS: f32 = 200.0;
const FILL_COST: i64 = 60;
/// The most energy it puts into its work per tick.
const DRAW_RATE: i64 = 100;
/// The most blocks looked at per tick for the next one to work on.
const SCAN: u32 = 512;

pub fn tick(b: &mut SystemBatch<Level>) {
    let w = b.world();
    for (id, l, _) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        let pos = st.block_pos();
        if !LEVELER.is(w.get(pos)) {
            continue;
        }
        if l.stage == stage::NEW {
            start(&w, pos, l);
        }
        crate::store::eject(&w, id, pos);
        match l.stage {
            stage::CLEAR => clear_next(&w, id, l),
            stage::FILL => fill_next(&w, id, l),
            _ => {}
        }
        let working = matches!(Status::from_raw(l.status), Status::Clearing | Status::Filling);
        crate::machine::show_working(&w, pos, working);
    }
}

fn start(w: &World, pos: [i32; 3], l: &mut Level) {
    let front = crate::machine::front(w, pos);
    (l.lo, l.hi) = crate::markers::take_area(w, pos)
        .unwrap_or_else(|| crate::markers::square_behind(pos, front, DEFAULT_SIZE, 0));
    l.stage = stage::CLEAR;
    l.cursor = [l.lo[0], l.lo[1] + CLEAR_HEIGHT - 1, l.lo[2]];
}

/// The block looked at after `c`: along x, then z, then the next layer `dy` away.
fn next(lo: [i32; 3], hi: [i32; 3], c: [i32; 3], dy: i32) -> [i32; 3] {
    if c[0] < hi[0] {
        [c[0] + 1, c[1], c[2]]
    } else if c[2] < hi[2] {
        [lo[0], c[1], c[2] + 1]
    } else {
        [lo[0], c[1] + dy, lo[2]]
    }
}

/// Breaks the next block standing at or above the leveler's height.
fn clear_next(w: &World, id: u64, l: &mut Level) {
    let Some((c, state)) = find(w, l, |s| breakable(s) && !LEVELER.is(s)) else {
        return;
    };
    l.status = Status::Clearing as u32;
    if !pay(w, id, l, cost(state, CLEAR_BASE, CLEAR_PER_HARDNESS)) {
        return;
    }
    match harvest(w, id, c, state) {
        // Paid for already: it stays paid until there is room.
        Harvest::Full => l.status = Status::Full as u32,
        Harvest::Done | Harvest::Refused => {
            l.spent = 0;
            l.cursor = next(l.lo, l.hi, c, -1);
        }
    }
}

/// Fills the next hole below the leveler's height with a block from its stock.
fn fill_next(w: &World, id: u64, l: &mut Level) {
    let Some((c, _)) = find(w, l, opens_into) else {
        return;
    };
    let stock = w.inventory(id).group("input");
    let Some((slot, block)) =
        (0..stock.len()).find_map(|i| fill_block(stock.get(i)).map(|b| (i, b)))
    else {
        l.status = Status::NoBlocks as u32;
        return;
    };
    l.status = Status::Filling as u32;
    if !pay(w, id, l, FILL_COST) {
        return;
    }
    w.set(c, block);
    stock.set(slot, stock.get(slot).shrink(1));
    l.spent = 0;
    l.cursor = next(l.lo, l.hi, c, 1);
}

/// The block a stack places, if it is a whole solid one fit to fill with.
fn fill_block(stack: ItemStack) -> Option<u32> {
    let block = item_block(stack.item).filter(|_| !stack.is_empty())?;
    let whole = abi::block_flags::SOLID | abi::block_flags::OPAQUE;
    (state_flags(block) & whole == whole).then_some(block)
}

/// From the cursor on, the next block that `wanted` picks and its state; moves on to the next
/// stage at the end of this one. `None` while waiting or when there is nothing this tick.
fn find(w: &World, l: &mut Level, wanted: impl Fn(u32) -> bool) -> Option<([i32; 3], u32)> {
    let level = l.lo[1];
    for _ in 0..SCAN {
        let c = l.cursor;
        if l.stage == stage::CLEAR && c[1] < level {
            l.stage = stage::FILL;
            l.cursor = [l.lo[0], level - FILL_DEPTH, l.lo[2]];
            return None;
        }
        if l.stage == stage::FILL && c[1] >= level {
            l.stage = stage::DONE;
            l.status = Status::Done as u32;
            return None;
        }
        let Some(state) = w.get_loaded(c) else {
            l.status = Status::Blocked as u32;
            return None;
        };
        if wanted(state) {
            return Some((c, state));
        }
        l.cursor = next(l.lo, l.hi, c, if l.stage == stage::CLEAR { -1 } else { 1 });
    }
    None
}

/// Puts stored energy into a job of `cost`; true once it is paid for.
fn pay(w: &World, id: u64, l: &mut Level, cost: i64) -> bool {
    let drawn = crate::machine::draw(w, id, &mut l.spent, cost, DRAW_RATE);
    if l.spent >= cost {
        return true;
    }
    if drawn == 0 {
        l.status = Status::NoPower as u32;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_then_layers_in_either_direction() {
        let (lo, hi) = ([0, 5, 0], [1, 5, 1]);
        let mut c = [0, 5, 0];
        let mut seen = vec![c];
        for _ in 0..4 {
            c = next(lo, hi, c, -1);
            seen.push(c);
        }
        assert_eq!(
            seen,
            vec![[0, 5, 0], [1, 5, 0], [0, 5, 1], [1, 5, 1], [0, 4, 0]],
            "along x, then z, then down"
        );
        assert_eq!(next(lo, hi, [1, 5, 1], 1), [0, 6, 0], "or up");
    }
}
