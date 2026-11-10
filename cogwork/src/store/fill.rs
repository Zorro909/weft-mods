//! Putting stacks into slots: onto the stacks they join first, then into empty slots.

use weft_sdk::*;

/// Puts `stack` into `slots`, onto stacks it joins first and then into empty ones, at most
/// `cap(item)` to a slot; returns what didn't fit.
pub fn fill_slots(
    slots: &mut [ItemStack],
    stack: ItemStack,
    cap: impl Fn(u32) -> u32,
) -> ItemStack {
    let mut left = stack;
    for into_empty in [false, true] {
        for slot in slots.iter_mut() {
            let joins = !slot.is_empty() && slot.stacks_with(&left);
            let takes = if into_empty { slot.is_empty() } else { joins };
            if takes && !left.is_empty() {
                left = top_up(slot, left, cap(left.item));
            }
        }
    }
    left
}

/// Puts as much of `stack` into `slot`, empty or holding the same, as brings it to `cap`;
/// returns the rest.
fn top_up(slot: &mut ItemStack, stack: ItemStack, cap: u32) -> ItemStack {
    let held = if slot.is_empty() { 0 } else { slot.count };
    let moved = cap.saturating_sub(held).min(stack.count);
    if moved == 0 {
        return stack;
    }
    *slot = ItemStack {
        count: held + moved,
        ..stack
    };
    stack.shrink(moved)
}

/// Puts `stacks` into `slots` one after another; false when they don't all fit.
pub fn fill_all(slots: &mut [ItemStack], stacks: &[ItemStack], cap: impl Fn(u32) -> u32) -> bool {
    stacks
        .iter()
        .all(|&stack| fill_slots(slots, stack, &cap).is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLANK: u32 = 7;
    const BUCKET: u32 = 9;

    /// Planks stack to 64, buckets not at all.
    fn cap(item: u32) -> u32 {
        if item == BUCKET { 1 } else { 64 }
    }

    #[test]
    fn items_join_their_stacks_before_taking_empty_slots() {
        let mut slots = [ItemStack::EMPTY, ItemStack::new(PLANK, 60)];
        let left = fill_slots(&mut slots, ItemStack::new(PLANK, 10), cap);
        assert_eq!(slots[1].count, 64, "the stack filled up first");
        assert_eq!(
            slots[0],
            ItemStack::new(PLANK, 6),
            "the rest in the empty slot"
        );
        assert!(left.is_empty(), "all of it went in");
    }

    #[test]
    fn what_has_no_room_is_handed_back() {
        let mut slots = [ItemStack::new(BUCKET, 1), ItemStack::EMPTY];
        let left = fill_slots(&mut slots, ItemStack::new(BUCKET, 3), cap);
        assert_eq!(
            slots[1],
            ItemStack::new(BUCKET, 1),
            "one into the empty slot"
        );
        assert_eq!(left, ItemStack::new(BUCKET, 2), "two left over");
    }
    #[test]
    fn stacks_that_each_fit_alone_may_not_fit_together() {
        let output = [ItemStack::new(PLANK, 1), ItemStack::EMPTY];
        let made = [ItemStack::new(BUCKET, 1), ItemStack::new(BUCKET, 1)];
        assert!(
            fill_all(&mut output.clone(), &made[..1], cap),
            "one bucket fits"
        );
        assert!(!fill_all(&mut output.clone(), &made, cap), "two don't");
    }
}
