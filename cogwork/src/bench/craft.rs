//! A bench's tick: what its pattern makes, paid for with energy once every cell holds its item,
//! then made, one item of each cell used up, as soon as the output has room for all of it.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use base_api::CRAFTING;
use weft_sdk::*;

use super::{Bench, CRAFT_COST, Status, cells, pattern};
use crate::lib::Pos;
use crate::machine::{self, Payment};
use crate::store::{insert_all, insert_into, stacks};

/// The most energy a bench puts into a craft per tick: one takes at least 8 ticks.
const DRAW_RATE: i64 = 30;

/// What each pattern of ghosts makes, worked out the first time a bench has it: recipes don't
/// change while the game runs.
static MADE_BY: LazyLock<Mutex<HashMap<[u32; 9], ItemStack>>> = LazyLock::new(Default::default);

pub fn tick(b: &mut SystemBatch<Bench>) {
    let w = b.world();
    for (id, bench, _) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        let pos = Pos::from(st.block_pos());
        let pattern = pattern(bench);
        let status = if evict_misplaced(&w, id, &pattern) {
            work(&w, id, pos, bench, &pattern)
        } else {
            Status::Full
        };
        bench.status = status as u32;
        bench.filled = filled(&grid(&w, id));
        crate::machine::show_working(&w, pos, status == Status::Working);
    }
}

fn grid(w: &World, id: u64) -> Vec<ItemStack> {
    stacks(&w.inventory(id).group("grid"))
}

/// Which cells hold items, a bit each.
fn filled(cells: &[ItemStack]) -> u32 {
    (0..cells.len())
        .filter(|&i| !cells[i].is_empty())
        .fold(0, |bits, i| bits | 1 << i)
}

/// Moves what the pattern no longer asks for out of its cells, into the output; false while
/// some of it has no room there and stays in its cell.
fn evict_misplaced(w: &World, id: u64, pattern: &[u32; 9]) -> bool {
    let g = w.inventory(id).group("grid");
    let mut all_out = true;
    for i in cells::misplaced(pattern, &grid(w, id)) {
        let left = insert_into(w.inventory(id).group("output"), g.get(i as u32), false);
        g.set(i as u32, left);
        all_out &= left.is_empty();
    }
    all_out
}

/// Works on the next craft of `pattern` at the bench `id` at `pos`: pays for it, then makes it
/// once the output has room.
fn work(w: &World, id: u64, pos: Pos, bench: &mut Bench, pattern: &[u32; 9]) -> Status {
    let result = made_by(pattern);
    if result.is_empty() {
        bench.spent = 0;
        return Status::NoRecipe;
    }
    let cells = grid(w, id);
    if !cells::complete(pattern, &cells) {
        return Status::Missing;
    }
    match machine::pay(w, id, &mut bench.spent, CRAFT_COST, DRAW_RATE) {
        Payment::Paying => return Status::Working,
        Payment::NoPower => return Status::NoPower,
        // It waits with its arrow full until the output has room.
        Payment::Paid => {}
    }
    let mut made = vec![result];
    made.extend(cells.iter().filter_map(|&used| left_behind(used)));
    if !insert_all(w.inventory(id).group("output"), &made) {
        return Status::Full;
    }
    use_up(w, id, &cells);
    w.play_sound(&crate::sounds::CRAFT, pos.center(), 1.0);
    bench.spent = 0;
    Status::Working
}

/// What the pattern of ghosts `pattern` makes; empty for no recipe.
fn made_by(pattern: &[u32; 9]) -> ItemStack {
    let mut known = MADE_BY.lock().unwrap_or_else(|e| e.into_inner());
    *known.entry(*pattern).or_insert_with(|| {
        let grid = pattern.map(|item| match item {
            0 => ItemStack::EMPTY,
            _ => ItemStack::new(item, 1),
        });
        recipe_grid_output(&CRAFTING, &grid, 3, 3)
    })
}

/// What an ingredient leaves behind when used up: a bucket of a water bucket.
fn left_behind(used: ItemStack) -> Option<ItemStack> {
    let left = REMAINDER.on_item(used.item).filter(|_| !used.is_empty())?;
    Some(ItemStack::new(left.0, 1))
}

/// Takes one item out of every cell.
fn use_up(w: &World, id: u64, cells: &[ItemStack]) {
    let g = w.inventory(id).group("grid");
    for (i, used) in (0..).zip(cells) {
        if !used.is_empty() {
            g.set(i, used.shrink(1));
        }
    }
}
