//! Items in a row of slots, counted and moved by kind: a stall's stock and till, a player's
//! inventory. Plain values in and out, so a trade can be worked out before anything moves.

use weft_sdk::ItemStack;

/// Whether `stack` holds items of `kind`'s kind: the same item with the same data.
pub fn same_kind(stack: ItemStack, kind: ItemStack) -> bool {
    !stack.is_empty() && stack.item == kind.item && stack.data == kind.data
}

/// How many items of `kind`'s kind `slots` hold.
pub fn count(slots: &[ItemStack], kind: ItemStack) -> u32 {
    slots
        .iter()
        .filter(|stack| same_kind(**stack, kind))
        .map(|stack| stack.count)
        .sum()
}

/// How many more items of `kind`'s kind fit into `slots`, `max` to a stack.
pub fn room(slots: &[ItemStack], kind: ItemStack, max: u32) -> u32 {
    slots
        .iter()
        .map(|stack| match stack {
            stack if stack.is_empty() => max,
            stack if same_kind(*stack, kind) => max.saturating_sub(stack.count),
            _ => 0,
        })
        .sum()
}

/// Takes `n` items of `kind`'s kind out of `slots`, from the last slot back; how many it took.
pub fn take(slots: &mut [ItemStack], kind: ItemStack, n: u32) -> u32 {
    let mut left = n;
    for stack in slots.iter_mut().rev() {
        if left == 0 {
            break;
        }
        if !same_kind(*stack, kind) {
            continue;
        }
        let taken = left.min(stack.count);
        stack.count -= taken;
        left -= taken;
        if stack.count == 0 {
            *stack = ItemStack::EMPTY;
        }
    }
    n - left
}

/// Puts `n` items of `kind`'s kind into `slots`, `max` to a stack: onto stacks of the kind first,
/// then into empty slots; how many it put.
pub fn put(slots: &mut [ItemStack], kind: ItemStack, n: u32, max: u32) -> u32 {
    let mut left = n;
    for stack in slots.iter_mut().filter(|stack| same_kind(**stack, kind)) {
        let added = left.min(max.saturating_sub(stack.count));
        stack.count += added;
        left -= added;
    }
    for stack in slots.iter_mut().filter(|stack| stack.is_empty()) {
        if left == 0 {
            break;
        }
        let added = left.min(max);
        *stack = ItemStack {
            count: added,
            ..kind
        };
        left -= added;
    }
    n - left
}

#[cfg(test)]
mod tests {
    use super::*;

    const BREAD: ItemStack = ItemStack {
        item: 7,
        count: 1,
        data: 0,
    };
    const IRON: ItemStack = ItemStack {
        item: 9,
        count: 1,
        data: 0,
    };

    fn of(kind: ItemStack, count: u32) -> ItemStack {
        ItemStack { count, ..kind }
    }

    #[test]
    fn counting_goes_by_kind() {
        let slots = [of(BREAD, 10), of(IRON, 3), ItemStack::EMPTY, of(BREAD, 5)];
        assert_eq!(count(&slots, BREAD), 15, "both stacks of bread");
        let marked = ItemStack { data: 1, ..BREAD };
        assert_eq!(count(&slots, marked), 0, "other data is another kind");
    }

    #[test]
    fn room_is_empty_slots_and_the_rest_of_stacks() {
        let slots = [of(BREAD, 60), of(IRON, 3), ItemStack::EMPTY];
        assert_eq!(
            room(&slots, BREAD, 64),
            4 + 64,
            "the stack's rest and an empty slot"
        );
    }

    #[test]
    fn taking_takes_from_the_back_and_empties_slots() {
        let mut slots = [of(BREAD, 10), of(IRON, 3), of(BREAD, 5)];
        assert_eq!(take(&mut slots, BREAD, 7), 7, "all of it");
        assert_eq!(
            slots,
            [of(BREAD, 8), of(IRON, 3), ItemStack::EMPTY],
            "the last stack first"
        );
        assert_eq!(take(&mut slots, BREAD, 20), 8, "only what there is");
    }

    #[test]
    fn putting_fills_stacks_then_empty_slots() {
        let mut slots = [ItemStack::EMPTY, of(BREAD, 60), of(IRON, 3)];
        assert_eq!(put(&mut slots, BREAD, 10, 64), 10, "all of it");
        assert_eq!(
            slots,
            [of(BREAD, 6), of(BREAD, 64), of(IRON, 3)],
            "the stack first"
        );
        assert_eq!(put(&mut slots, BREAD, 100, 64), 58, "only what fits");
    }
}
