//! What a quarry builds its frame of: each frame block takes its kind's `frame_cost` out of the
//! `materials` group, filled by hand or by tube. A kind with no cost builds with energy alone.

use weft_sdk::{ItemStack, StackData, World};

use super::dig::Quarry;
use super::kind::Cost;

/// The slot group a quarry keeps its frame materials in.
pub const GROUP: &str = "materials";

/// Takes one frame block's materials out of the `materials` group of `quarry`; false, and
/// nothing taken, when they aren't all there. With `simulate` it only says whether they are.
pub fn take(world: &World, quarry: Quarry, simulate: bool) -> bool {
    let cost = quarry.kind.frame_cost;
    if cost.is_empty() {
        return true;
    }
    let Some(group) = world.inventory(quarry.id).find_group(GROUP) else {
        return false;
    };
    let slots = crate::store::stacks(&group);
    let Some(left) = taken(&slots, cost, |cost, item| cost.material.is(item)) else {
        return false;
    };
    if !simulate {
        for (i, (&before, &after)) in (0..).zip(slots.iter().zip(&left)) {
            if before != after {
                group.set(i, after);
            }
        }
    }
    true
}

/// `slots` after taking each of `cost` from the stacks that `is` says are its material; none
/// when one isn't there in full.
fn taken(
    slots: &[ItemStack],
    cost: &[Cost],
    is: impl Fn(&Cost, u32) -> bool,
) -> Option<Vec<ItemStack>> {
    let mut left = slots.to_vec();
    for cost in cost {
        let mut owed = cost.count;
        for slot in &mut left {
            if owed == 0 {
                break;
            }
            if slot.is_empty() || !is(cost, slot.item) {
                continue;
            }
            let take = owed.min(slot.count);
            *slot = slot.shrink(take);
            owed -= take;
        }
        if owed > 0 {
            return None;
        }
    }
    Some(left)
}

#[cfg(test)]
mod tests {
    use weft_sdk::{Item, Tag};

    use super::super::kind::Material;
    use super::*;

    static LOGS: Tag = Tag::named("logs");
    static STRING: Item = Item::named("base:string");
    const OAK: u32 = 3;
    const BIRCH: u32 = 4;
    const STRING_ID: u32 = 9;

    /// A log and a string, as the Timber Quarry takes.
    fn lashed() -> [Cost; 2] {
        [
            Cost {
                material: Material::Tagged(&LOGS),
                count: 1,
            },
            Cost {
                material: Material::Item(&STRING),
                count: 1,
            },
        ]
    }

    /// Whether `item` is the material of `cost`, by the ids above.
    fn is(cost: &Cost, item: u32) -> bool {
        match cost.material {
            Material::Tagged(_) => item == OAK || item == BIRCH,
            Material::Item(_) => item == STRING_ID,
        }
    }

    #[test]
    fn a_frame_block_takes_a_log_and_a_string() {
        let slots = [
            ItemStack::new(STRING_ID, 3),
            ItemStack::EMPTY,
            ItemStack::new(BIRCH, 1),
            ItemStack::new(OAK, 5),
        ];
        let left = taken(&slots, &lashed(), is).expect("both are there");
        assert_eq!(left[0].count, 2, "a string");
        assert!(left[2].is_empty(), "the first log found");
        assert_eq!(left[3].count, 5, "and no other");
    }

    #[test]
    fn nothing_is_taken_short_of_a_whole_block() {
        let logs_only = [ItemStack::new(OAK, 5)];
        assert!(taken(&logs_only, &lashed(), is).is_none(), "no string");
        assert!(taken(&[], &lashed(), is).is_none(), "empty");
        assert_eq!(taken(&[], &[], is), Some(vec![]), "a frame of energy alone");
    }

    #[test]
    fn a_cost_of_several_is_gathered_across_stacks() {
        let iron = [Cost {
            material: Material::Item(&STRING),
            count: 3,
        }];
        let slots = [ItemStack::new(STRING_ID, 2), ItemStack::new(STRING_ID, 2)];
        let left = taken(&slots, &iron, is).expect("4 is enough");
        assert_eq!(left.iter().map(|stack| stack.count).sum::<u32>(), 1);
    }
}
