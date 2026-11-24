//! Each player's progress, on the server: the components it is kept in, counting toward a
//! quest's tasks, and finishing the quest when all of them are met.

use std::sync::Once;

use almanac_api::{
    COUNTING, Counting, DONE, Done, FINISHED, Finished, Quest, SHOWN, Shown, State, book,
};
use weft_sdk::*;

use crate::targets::TaskAt;
use crate::toast;

pub fn register(r: &mut Registry) {
    // Kept through death: what a player did stays done.
    let kept = component_flags::SYNC_OWNER | component_flags::KEEP_ON_RESPAWN;
    r.entity_component(&FINISHED, Finished::NONE, kept | component_flags::SAVE);
    r.entity_component(&COUNTING, Counting::NONE, kept | component_flags::SAVE);
    r.entity_component(&SHOWN, Shown::FIRST, kept);
    r.edit(&PLAYER)
        .attach(&FINISHED)
        .attach(&COUNTING)
        .attach(&SHOWN);
}

/// How a task's count changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Change {
    /// So many more.
    By(u32),
    /// To at least this: for what is counted anew each time, as the items a player holds.
    To(u32),
}

impl Change {
    /// The count after it, from `before`, no further than `goal`.
    fn from(self, before: u32, goal: u32) -> u32 {
        let count = match self {
            Change::By(more) => before.saturating_add(more),
            Change::To(least) => before.max(least),
        };
        count.min(goal)
    }
}

/// Changes the count of the task at `at` for `player`, and finishes its quest once every task
/// is at its goal. Only an open quest of a player in survival counts.
pub fn count(world: &World, player: u64, at: TaskAt, change: Change) {
    if player == 0 || world.outside_survival(player) {
        return;
    }
    let TaskAt { quest, task } = at;
    let book = book();
    let held = &book.quests[quest];
    let Some(goal) = held.tasks.get(task).map(|task| task.count) else {
        return;
    };
    let (Some(finished), Some(counting)) = (
        world.component_mut(player, &FINISHED),
        world.component_mut(player, &COUNTING),
    ) else {
        return;
    };
    if book.state(quest, finished) != State::Open {
        return;
    }
    let before = counting.of(held.key)[task];
    let count = change.from(before, goal);
    if count == before {
        return;
    }
    if !raised(counting, held, task, count) {
        static SAID: Once = Once::new();
        return SAID.call_once(|| {
            warn!(
                "almanac: a player counts too many quests at once, {} is left out",
                held.name
            )
        });
    }
    if all_met(held, counting) {
        finish(world, player, quest);
    }
}

/// Raises task `task` of `quest` to `count` in `counting`, making room first when it is full.
/// False when the count is lost.
fn raised(counting: &mut Counting, quest: &Quest, task: usize, count: u32) -> bool {
    if counting.raise(quest.key, task, count) {
        return true;
    }
    let book = book();
    counting.keep(|key| book.quests.iter().any(|quest| quest.key == key));
    counting.raise(quest.key, task, count)
}

/// Holds the quest at `quest` in the book as finished by `player`, whatever its counts, and
/// tells everyone. False when it already was.
pub fn finish(world: &World, player: u64, quest: usize) -> bool {
    let book = book();
    let held = &book.quests[quest];
    let (Some(finished), Some(counting)) = (
        world.component_mut(player, &FINISHED),
        world.component_mut(player, &COUNTING),
    ) else {
        return false;
    };
    if finished.has(held.key) {
        return false;
    }
    if !finished.finish(held.key) {
        warn!(
            "almanac: a player's save holds no more finished quests, {} is left out",
            held.name
        );
        return false;
    }
    counting.forget(held.key);
    let chapter = book.chapters[held.chapter].id;
    world.emit(
        &DONE,
        Done {
            player,
            quest: held.id,
            chapter,
        },
    );
    toast::send(world, player, held.id);
    // A quest that hands over nothing is claimed in answer to `DONE` (`claim::claim_nothing`).
    if !held.rewards.is_empty() {
        world.hint("almanac.rewards_wait", &[]).to(player);
    }
    true
}

/// Forgets everything `player` did.
pub fn reset(world: &World, player: u64) {
    world.set_component(player, &FINISHED, Finished::NONE);
    world.set_component(player, &COUNTING, Counting::NONE);
    world.set_component(player, &SHOWN, Shown::FIRST);
}

/// Whether every task of `quest` is at its goal in `counting`.
fn all_met(quest: &Quest, counting: &Counting) -> bool {
    let counts = counting.of(quest.key);
    quest
        .tasks
        .iter()
        .zip(counts)
        .all(|(task, count)| count >= task.count)
}

#[cfg(test)]
mod tests {
    use almanac_api::{Kind, Task, key_of};

    use super::*;

    #[test]
    fn counts_add_up_to_the_goal_and_stop() {
        assert_eq!(Change::By(3).from(2, 8), 5, "more");
        assert_eq!(Change::By(30).from(2, 8), 8, "no further than the goal");
        assert_eq!(Change::By(u32::MAX).from(2, 8), 8, "however much");
    }

    #[test]
    fn what_is_held_only_raises_the_count() {
        assert_eq!(Change::To(3).from(1, 8), 3, "more than before");
        assert_eq!(Change::To(1).from(3, 8), 3, "fewer: the count stays");
        assert_eq!(Change::To(64).from(3, 8), 8, "no further than the goal");
    }

    fn quest(goals: &[u32]) -> Quest {
        Quest {
            id: 0,
            name: "a:quest".to_string(),
            key: key_of("a:quest"),
            chapter: 0,
            icon: 0,
            after: Vec::new(),
            tasks: goals
                .iter()
                .map(|&goal| Task::new(Kind::Have, "base:coal", goal))
                .collect(),
            rewards: Vec::new(),
        }
    }

    #[test]
    fn a_quest_is_met_when_every_task_is() {
        let quest = quest(&[2, 5]);
        let mut counting = Counting::NONE;
        assert!(!all_met(&quest, &counting), "nothing counted");
        counting.raise(quest.key, 0, 2);
        assert!(!all_met(&quest, &counting), "one of two");
        counting.raise(quest.key, 1, 5);
        assert!(all_met(&quest, &counting), "both");
    }
}
