//! What a quarry builds its frame of: a batch of frame blocks (`Kind::frame_batch`) takes its
//! kind's `frame_cost` out of the `materials` group, filled by hand or by tube, as the first of
//! them goes up, and the quarry counts the blocks it has paid for ahead. The count follows the
//! blocks put up, never their place in the frame, so a block still standing neither costs nor
//! saves anything. A kind with no cost builds with energy alone.

use weft_sdk::{ItemStack, StackData, World};

use super::dig::Quarry;
use super::kind::Cost;

/// The slot group a quarry keeps its frame materials in.
pub const GROUP: &str = "materials";

/// Whether the next frame block takes a batch's materials, with `paid_ahead` blocks paid for.
pub fn takes_batch(paid_ahead: u32) -> bool {
    paid_ahead == 0
}

/// The blocks paid for ahead once one more frame block is up, from `paid_ahead`, in batches of
/// `batch`: a block that took a batch leaves the rest of it.
pub fn after_putting_up(paid_ahead: u32, batch: u32) -> u32 {
    paid_ahead.checked_sub(1).unwrap_or(batch.max(1) - 1)
}

/// Takes a batch's materials out of the `materials` group of `quarry`; false, and nothing
/// taken, when they aren't all there. With `simulate` it only says whether they are.
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
    /// Frame blocks a steel ingot pays for, and the blocks of a 9 by 9 frame.
    const STEEL_BATCH: u32 = 4;
    const FRAME_BLOCKS: usize = 92;

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

    /// A frame being built and broken: where a block stands, whether it took a batch, which it
    /// gives back when broken (`frames`); and the batches taken and given back so far.
    #[derive(Default)]
    struct Model {
        blocks: Vec<Option<bool>>,
        paid_ahead: u32,
        taken: u32,
        given_back: u32,
    }

    impl Model {
        fn new() -> Model {
            Model {
                blocks: vec![None; FRAME_BLOCKS],
                ..Model::default()
            }
        }

        /// Puts up every missing block, as `build_frame` passes over those standing.
        fn build(&mut self) {
            for block in self.blocks.iter_mut().filter(|block| block.is_none()) {
                let takes = takes_batch(self.paid_ahead);
                self.taken += u32::from(takes);
                *block = Some(takes);
                self.paid_ahead = after_putting_up(self.paid_ahead, STEEL_BATCH);
            }
        }

        /// Breaks the blocks at the places `broken` picks.
        fn break_where(&mut self, broken: impl Fn(usize) -> bool) {
            for (place, block) in self.blocks.iter_mut().enumerate() {
                if broken(place) && block.take() == Some(true) {
                    self.given_back += 1;
                }
            }
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

    #[test]
    fn a_batch_is_taken_for_the_first_of_its_blocks() {
        let mut paid_ahead = 0;
        let takes: Vec<bool> = (0..9)
            .map(|_| {
                let takes = takes_batch(paid_ahead);
                paid_ahead = after_putting_up(paid_ahead, STEEL_BATCH);
                takes
            })
            .collect();
        let at: Vec<usize> = (0..9).filter(|&block| takes[block]).collect();
        assert_eq!(at, [0, 4, 8], "the first of every four");
        assert!(takes_batch(after_putting_up(0, 1)), "a batch of one: each");
        assert!(takes_batch(after_putting_up(0, 0)), "and of none");
    }

    #[test]
    fn rebuilding_a_steel_frame_pays_for_every_block_put_up() {
        let mut frame = Model::new();
        frame.build();
        assert_eq!(frame.taken, 23, "92 blocks take 23 ingots");
        frame.break_where(|place| place % 4 != 0);
        assert_eq!(frame.given_back, 0, "the blocks that took them still stand");
        frame.build();
        assert_eq!(
            frame.taken,
            23 + 18,
            "69 blocks put up again take 18 ingots"
        );
    }

    #[test]
    fn no_breaking_and_rebuilding_gives_back_more_than_was_paid() {
        let mut seed: u64 = 42;
        let mut random = move || {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            seed >> 33
        };
        let mut frame = Model::new();
        for _ in 0..200 {
            frame.build();
            if random() % 3 == 0 {
                // The quarry broken and placed again: a new one has nothing paid ahead.
                frame.paid_ahead = 0;
            }
            let (every, from) = (random() % 7 + 1, random() % 7);
            frame.break_where(|place| place as u64 % every == from % every);
            assert!(frame.given_back <= frame.taken, "never more than was paid");
        }
        frame.break_where(|_| true);
        assert_eq!(
            frame.given_back, frame.taken,
            "all of it, once all is broken"
        );
    }
}
