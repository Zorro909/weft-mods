//! Handing out a quest's rewards, and what the book's buttons send to the server.

use almanac_api::{CLAIM, CLAIMED, DONE, Done, FINISHED, NOTHING, Reward, SHOW, SHOWN, book};
use weft_sdk::*;

pub fn register(r: &mut Registry) {
    r.packet(&CLAIM, |packet, asked| {
        let (world, player) = (packet.world(), packet.player());
        // Asked by a client: `claim` gives only what the sender finished and hasn't taken yet.
        let claimed = book()
            .quest(asked.quest)
            .is_some_and(|quest| claim(&world, player, quest));
        // The book turns to what is next in the chapter.
        if let Some(shown) = world.component_mut(player, &SHOWN).filter(|_| claimed) {
            shown.quest = NOTHING;
        }
    });
    r.packet(&SHOW, |packet, shown| {
        packet.world().set_component(packet.player(), &SHOWN, shown);
    });
    r.on_event(&DONE, |batch| {
        let world = batch.world();
        for done in batch.iter() {
            claim_nothing(&world, done);
        }
    });
}

/// Claims a finished quest that hands over no items: there is nothing to wait for. As an answer
/// to `DONE`, so that everyone hears of the quest being done before they hear of the claim.
fn claim_nothing(world: &World, done: &Done) {
    let book = book();
    let Some(quest) = book.quest(done.quest) else {
        return;
    };
    if book.quests[quest].rewards.is_empty() {
        claim(world, done.player, quest);
    }
}

/// Hands `player` the rewards of the quest at `quest` in the book and tells everyone. False
/// unless the player finished it and hasn't taken them yet.
pub fn claim(world: &World, player: u64, quest: usize) -> bool {
    let book = book();
    let held = &book.quests[quest];
    let Some(finished) = world.component_mut(player, &FINISHED) else {
        return false;
    };
    if !finished.claim(held.key) {
        return false;
    }
    for reward in &held.rewards {
        give(world, player, reward);
    }
    let chapter = book.chapters[held.chapter].id;
    world.emit(
        &CLAIMED,
        Done {
            player,
            quest: held.id,
            chapter,
        },
    );
    true
}

/// Puts `reward` into the inventory of `player`, a stack at a time; what doesn't fit drops at
/// their feet.
fn give(world: &World, player: u64, reward: &Reward) {
    let Some(item) = Item::lookup(&reward.item) else {
        return warn!("almanac: a reward names {}, which is no item", reward.item);
    };
    let Some(at) = world.entity(player).map(|player| player.pos) else {
        return;
    };
    for count in stacks(reward.count, max_stack(item)) {
        let left = world.inventory(player).insert(ItemStack::new(item, count));
        if !left.is_empty() {
            world.spawn_item(at, left);
        }
    }
}

/// `count` items as stacks of at most `most`.
fn stacks(count: u32, most: u32) -> Vec<u32> {
    let most = most.max(1);
    let (full, rest) = (count / most, count % most);
    std::iter::repeat_n(most, full as usize)
        .chain((rest > 0).then_some(rest))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewards_come_a_stack_at_a_time() {
        assert_eq!(stacks(8, 64), [8], "less than a stack");
        assert_eq!(stacks(130, 64), [64, 64, 2], "full stacks, then the rest");
        assert_eq!(stacks(128, 64), [64, 64], "no empty stack after full ones");
        assert_eq!(stacks(3, 1), [1, 1, 1], "tools don't stack");
    }

    #[test]
    fn an_item_that_stacks_to_nothing_still_comes() {
        assert_eq!(stacks(2, 0), [1, 1]);
    }
}
