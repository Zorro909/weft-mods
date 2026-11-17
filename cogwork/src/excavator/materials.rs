//! What a quarry builds its frame of: each frame block takes its kind's `frame_cost` out of the
//! `materials` group, filled by hand or by tube; a cost that lasts several frame blocks is taken
//! for the first of them. A kind with no cost builds with energy alone.

use weft_sdk::{ItemStack, StackData, World};

use super::dig::Quarry;
use super::kind::Cost;

/// The slot group a quarry keeps its frame materials in.
pub const GROUP: &str = "materials";

/// Takes the materials of the frame block at `step` of the frame out of the `materials` group of
/// `quarry`; false, and nothing taken, when they aren't all there. With `simulate` it only says
/// whether they are.
pub fn take(world: &World, quarry: Quarry, step: u32, simulate: bool) -> bool {
    let cost: Vec<&Cost> = due(quarry.kind.frame_cost, step).collect();
    if cost.is_empty() {
        return true;
    }
    let Some(group) = world.inventory(quarry.id).find_group(GROUP) else {
        return false;
    };
    let slots = crate::store::stacks(&group);
    let Some(left) = taken(&slots, &cost, |cost, item| cost.material.is(item)) else {
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

/// What of `cost` the frame block at `step` takes.
fn due(cost: &[Cost], step: u32) -> impl Iterator<Item = &Cost> {
    cost.iter().filter(move |cost| cost.is_due(step))
}

/// `slots` after taking each of `cost` from the stacks that `is` says are its material; none
/// when one isn't there in full.
fn taken(
    slots: &[ItemStack],
    cost: &[&Cost],
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
                blocks: 1,
            },
            Cost {
                material: Material::Item(&STRING),
                count: 1,
                blocks: 1,
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
        let lashed = lashed();
        let left = taken(&slots, &due(&lashed, 0).collect::<Vec<_>>(), is).expect("both are there");
        assert_eq!(left[0].count, 2, "a string");
        assert!(left[2].is_empty(), "the first log found");
        assert_eq!(left[3].count, 5, "and no other");
    }

    #[test]
    fn nothing_is_taken_short_of_a_whole_block() {
        let logs_only = [ItemStack::new(OAK, 5)];
        let lashed = lashed();
        let all: Vec<&Cost> = lashed.iter().collect();
        assert!(taken(&logs_only, &all, is).is_none(), "no string");
        assert!(taken(&[], &all, is).is_none(), "empty");
        assert_eq!(taken(&[], &[], is), Some(vec![]), "a frame of energy alone");
    }

    #[test]
    fn a_cost_of_several_is_gathered_across_stacks() {
        let iron = Cost {
            material: Material::Item(&STRING),
            count: 3,
            blocks: 1,
        };
        let slots = [ItemStack::new(STRING_ID, 2), ItemStack::new(STRING_ID, 2)];
        let left = taken(&slots, &[&iron], is).expect("4 is enough");
        assert_eq!(left.iter().map(|stack| stack.count).sum::<u32>(), 1);
    }

    #[test]
    fn a_cost_for_several_blocks_is_taken_for_the_first_of_them() {
        let steel = [Cost {
            material: Material::Item(&STRING),
            count: 1,
            blocks: 4,
        }];
        let taken_at = |step| due(&steel, step).count();
        assert_eq!(taken_at(0), 1, "the first block takes it");
        assert_eq!(
            (1..4).map(taken_at).sum::<usize>(),
            0,
            "the next three nothing"
        );
        assert_eq!(taken_at(4), 1, "the fifth again");
    }
}
