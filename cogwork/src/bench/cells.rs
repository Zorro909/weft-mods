//! The pattern's cells: a ghost item each says what goes there, and each holds a stack of it.
//! A craft takes one item from every cell.

use weft_sdk::*;

/// The cell the next item of `stack` goes into: of those whose ghost it is, that it stacks with
/// and that have room (`cap` a cell), the emptiest, so the cells of one item fill evenly and
/// crafting goes on.
pub fn free_cell(pattern: &[u32], grid: &[ItemStack], stack: ItemStack, cap: u32) -> Option<usize> {
    let fits = |cell: &ItemStack| cell.is_empty() || cell.stacks_with(&stack);
    (0..pattern.len())
        .filter(|&i| stack.item != 0 && pattern[i] == stack.item)
        .filter(|&i| fits(&grid[i]) && grid[i].count < cap)
        .min_by_key(|&i| grid[i].count)
}

/// Whether every cell holds what its ghost asks for, and only that.
pub fn complete(pattern: &[u32], grid: &[ItemStack]) -> bool {
    pattern.iter().zip(grid).all(|(&ghost, s)| match ghost {
        0 => s.is_empty(),
        _ => !s.is_empty() && s.item == ghost,
    })
}

/// The cells holding something their ghost doesn't ask for (the pattern changed).
pub fn misplaced(pattern: &[u32], grid: &[ItemStack]) -> Vec<usize> {
    (0..pattern.len())
        .filter(|&i| !grid[i].is_empty() && grid[i].item != pattern[i])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLANK: u32 = 7;
    const STICK: u32 = 9;
    const NONE: ItemStack = ItemStack::EMPTY;

    #[test]
    fn items_fill_their_cells_evenly_up_to_a_stack() {
        let pattern = [PLANK, 0, PLANK];
        let mut grid = [NONE; 3];
        let next =
            |grid: &[ItemStack], item| free_cell(&pattern, grid, ItemStack::new(item, 1), 64);
        assert_eq!(next(&grid, STICK), None, "no cell asks for a stick");
        assert_eq!(next(&grid, PLANK), Some(0), "the first plank cell");
        grid[0] = ItemStack::new(PLANK, 1);
        assert!(!complete(&pattern, &grid), "one plank short");
        assert_eq!(next(&grid, PLANK), Some(2), "then the emptier one");
        grid[2] = ItemStack::new(PLANK, 1);
        assert!(complete(&pattern, &grid), "every cell filled");
        assert_eq!(next(&grid, PLANK), Some(0), "and on, a stack each");
        grid = [ItemStack::new(PLANK, 64), NONE, ItemStack::new(PLANK, 64)];
        assert_eq!(next(&grid, PLANK), None, "till both are full");
    }

    #[test]
    fn an_item_with_other_data_does_not_join_a_cell() {
        let named = ItemStack {
            data: 3,
            ..ItemStack::new(PLANK, 1)
        };
        let pattern = [PLANK, PLANK];
        let grid = [named, NONE];
        let plain = ItemStack::new(PLANK, 1);
        assert_eq!(
            free_cell(&pattern, &grid, plain, 64),
            Some(1),
            "the empty cell"
        );
        let grid = [named, named];
        assert_eq!(
            free_cell(&pattern, &grid, plain, 64),
            None,
            "not onto the named ones"
        );
        assert_eq!(
            free_cell(&pattern, &grid, named, 64),
            Some(0),
            "named onto named"
        );
    }

    #[test]
    fn a_changed_pattern_leaves_items_out_of_place() {
        let grid = [ItemStack::new(PLANK, 1), NONE, ItemStack::new(STICK, 1)];
        assert_eq!(misplaced(&[PLANK, 0, PLANK], &grid), vec![2], "the stick");
        assert!(!complete(&[PLANK, 0, PLANK], &grid), "so it can't craft");
    }
}
