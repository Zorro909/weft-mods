//! The pattern's cells: a ghost item each says what goes there, and each holds at most one real
//! item, put in one at a time. A craft takes them all at once.

use weft_sdk::*;

/// The cell an `item` goes into next: one whose ghost it is and that is still empty.
pub fn free_cell(pattern: &[u32], grid: &[ItemStack], item: u32) -> Option<usize> {
    (0..pattern.len()).find(|&i| item != 0 && pattern[i] == item && grid[i].is_empty())
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
    fn items_fill_their_cells_one_each() {
        let pattern = [PLANK, 0, PLANK];
        let mut grid = [NONE; 3];
        assert_eq!(
            free_cell(&pattern, &grid, STICK),
            None,
            "no cell asks for a stick"
        );
        assert_eq!(
            free_cell(&pattern, &grid, PLANK),
            Some(0),
            "the first plank cell"
        );
        grid[0] = ItemStack::new(PLANK, 1);
        assert!(!complete(&pattern, &grid), "one plank short");
        assert_eq!(
            free_cell(&pattern, &grid, PLANK),
            Some(2),
            "then the next one"
        );
        grid[2] = ItemStack::new(PLANK, 1);
        assert!(complete(&pattern, &grid), "every cell filled");
        assert_eq!(
            free_cell(&pattern, &grid, PLANK),
            None,
            "and no room for more"
        );
    }

    #[test]
    fn a_changed_pattern_leaves_items_out_of_place() {
        let grid = [ItemStack::new(PLANK, 1), NONE, ItemStack::new(STICK, 1)];
        assert_eq!(misplaced(&[PLANK, 0, PLANK], &grid), vec![2], "the stick");
        assert!(!complete(&[PLANK, 0, PLANK], &grid), "so it can't craft");
    }
}
