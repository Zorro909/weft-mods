//! What the machines that raise a grade share (the washer, the flotation cell): each works the
//! first Ore Rubble waiting in its input, and passes on Ore Rubble its step doesn't take.

use weft_sdk::SlotGroup;

use super::{Grade, Ore, ore_of};

/// The first Ore Rubble waiting in `input`: its slot, its metal and its grade.
pub fn waiting(input: &SlotGroup) -> Option<(u32, Ore, Grade)> {
    (0..input.len()).find_map(|slot| {
        let (ore, grade) = ore_of(&input.get(slot))?;
        Some((slot, ore, grade))
    })
}

/// Moves the stack in `slot` of `input` into `output` as far as it fits; whether any went.
pub fn pass_on(input: &SlotGroup, slot: u32, output: SlotGroup<'static>) -> bool {
    let stack = input.get(slot);
    let left = crate::store::insert_into(output, stack, false);
    input.set(slot, left);
    left != stack
}
