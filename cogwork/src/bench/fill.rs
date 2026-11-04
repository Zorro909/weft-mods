//! Filling the pattern: items given to the bench are spread over the cells whose ghosts ask for
//! them.

use weft_sdk::*;

use super::{BENCH, cells, pattern};

/// Puts what of `stack` the pattern of the bench `at` has room for into its cells, one item at a
/// time into the emptiest; returns the rest. `None` when `at` is no bench.
pub fn insert(w: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    let pattern = pattern(&w.component(at, &BENCH)?);
    let grid = w.inventory(at).group("grid");
    let mut cells: Vec<ItemStack> = (0..grid.len()).map(|i| grid.get(i)).collect();
    let mut left = stack;
    let cap = max_stack(stack.item);
    while let Some(i) = cells::free_cell(&pattern, &cells, left, cap).filter(|_| !left.is_empty()) {
        cells[i] = ItemStack {
            count: cells[i].count + 1,
            ..left
        };
        if !simulate {
            grid.set(i as u32, cells[i]);
        }
        left = left.shrink(1);
    }
    Some(left)
}
