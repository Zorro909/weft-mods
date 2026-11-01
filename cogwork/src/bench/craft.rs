//! A bench's tick: what its pattern makes, paid for with energy once every cell holds its item,
//! one item of each cell used up into the result.

use base_api::CRAFTING;
use weft_sdk::*;

use super::{Bench, CRAFT_COST, cells, status};
use crate::store::insert_into;

/// The most energy a bench puts into a craft per tick: one takes at least 8 ticks.
const DRAW_RATE: i64 = 30;

pub fn tick(b: &mut SystemBatch<Bench>) {
    let w = b.world();
    for (id, bench, _) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        let pos = st.block_pos();
        evict_misplaced(&w, id, pos, &bench.pattern);
        bench.status = work(&w, id, pos, bench);
        bench.filled = filled(&grid(&w, id));
        let working = if bench.status == status::WORKING {
            "true"
        } else {
            "false"
        };
        if w.prop(pos, "working").is_some_and(|p| p != working) {
            w.set_prop(pos, "working", working);
        }
    }
}

fn grid(w: &World, id: u64) -> Vec<ItemStack> {
    let g = w.inventory(id).group("grid");
    (0..g.len()).map(|i| g.get(i)).collect()
}

/// Which cells hold items, a bit each.
fn filled(cells: &[ItemStack]) -> u32 {
    (0..cells.len())
        .filter(|&i| !cells[i].is_empty())
        .fold(0, |bits, i| bits | 1 << i)
}

/// Moves what the pattern no longer asks for out of its cells, into the output.
fn evict_misplaced(w: &World, id: u64, pos: [i32; 3], pattern: &[u32; 9]) {
    for i in cells::misplaced(pattern, &grid(w, id)) {
        let g = w.inventory(id).group("grid");
        put_out(w, id, pos, g.get(i as u32));
        g.set(i as u32, ItemStack::EMPTY);
    }
}

/// Works on the next craft; returns the bench's status.
fn work(w: &World, id: u64, pos: [i32; 3], bench: &mut Bench) -> u32 {
    let pattern: Vec<ItemStack> = bench
        .pattern
        .iter()
        .map(|&item| {
            if item == 0 {
                ItemStack::EMPTY
            } else {
                ItemStack::new(item, 1)
            }
        })
        .collect();
    let result = recipe_match(&CRAFTING, &pattern, 3, 3);
    if result.is_empty() {
        bench.spent = 0;
        return status::NO_RECIPE;
    }
    let cells = grid(w, id);
    if !cells::complete(&bench.pattern, &cells) {
        return status::MISSING;
    }
    if !insert_into(w.inventory(id).group("output"), result, true).is_empty() {
        return status::FULL;
    }
    let drawn = crate::draw(w, id, &mut bench.spent, CRAFT_COST, DRAW_RATE);
    if bench.spent < CRAFT_COST {
        return if drawn > 0 {
            status::WORKING
        } else {
            status::NO_POWER
        };
    }
    let g = w.inventory(id).group("grid");
    for (i, used) in cells.iter().enumerate().filter(|(_, s)| !s.is_empty()) {
        g.set(i as u32, used.shrink(1));
        // What a used-up ingredient leaves behind (a bucket of a water bucket).
        if let Some(left) = REMAINDER.on_item(used.item) {
            put_out(w, id, pos, ItemStack::new(left.0, 1));
        }
    }
    put_out(w, id, pos, result);
    bench.spent = 0;
    status::WORKING
}

/// Puts `stack` into the output or, with no room there, onto the bench.
fn put_out(w: &World, id: u64, pos: [i32; 3], stack: ItemStack) {
    let rest = insert_into(w.inventory(id).group("output"), stack, false);
    if !rest.is_empty() {
        let top = [
            pos[0] as f64 + 0.5,
            pos[1] as f64 + 1.1,
            pos[2] as f64 + 0.5,
        ];
        w.spawn_item(top, rest);
    }
}
