//! What a player has done, kept on the player in three components the server writes and the
//! player's client reads. `Finished` and `Counting` are saved, so their layouts are a file
//! format: quests are held by `Key`, never by an id, and the sizes below are part of it.

use weft_sdk::{Component, value};

use crate::names::{Key, NO_KEY};

/// The quests a player's save holds as finished, at most.
pub const FINISHED_MOST: usize = 256;
/// The quests counted for a player at once, at most: a quest takes a place with its first
/// count and gives it back when it is finished.
pub const COUNTING_MOST: usize = 32;
/// The tasks of a quest that are counted; a quest's further tasks are left out of the book.
pub const TASKS_MOST: usize = 4;

value! {
    /// The quests a player finished (`NO_KEY` for a free place), with a bit each in `claimed`
    /// once its rewards are taken.
    pub struct Finished { pub quests: [Key; FINISHED_MOST], pub claimed: [u64; FINISHED_MOST / 64] }
}

value! {
    /// How far one quest's tasks are, in the order the quest lists them.
    pub struct Count { pub quest: Key, pub tasks: [u32; TASKS_MOST] }
}

value! {
    /// The counts of the quests a player is working on (`NO_KEY` for a free place).
    pub struct Counting { pub quests: [Count; COUNTING_MOST] }
}

value! {
    /// The chapter and quest a player's book is open at, as record ids; `NOTHING` for the first
    /// of each. Not saved: ids are only good while the game runs.
    pub struct Shown { pub chapter: u32, pub quest: u32 }
}

pub static FINISHED: Component<Finished> = Component::named("almanac:finished");
pub static COUNTING: Component<Counting> = Component::named("almanac:counting");
pub static SHOWN: Component<Shown> = Component::named("almanac:shown");

/// No chapter or quest in a `Shown`.
pub const NOTHING: u32 = u32::MAX;

impl Shown {
    pub const FIRST: Shown = Shown {
        chapter: NOTHING,
        quest: NOTHING,
    };
}

impl Finished {
    pub const NONE: Finished = Finished {
        quests: [NO_KEY; FINISHED_MOST],
        claimed: [0; FINISHED_MOST / 64],
    };

    fn place(&self, quest: Key) -> Option<usize> {
        self.quests.iter().position(|&held| held == quest)
    }

    pub fn has(&self, quest: Key) -> bool {
        quest != NO_KEY && self.place(quest).is_some()
    }

    /// Whether the rewards of `quest` are taken.
    pub fn claimed(&self, quest: Key) -> bool {
        quest != NO_KEY
            && self
                .place(quest)
                .is_some_and(|place| self.claimed[place / 64] >> (place % 64) & 1 != 0)
    }

    /// How many quests are finished.
    pub fn len(&self) -> usize {
        self.quests.iter().filter(|&&held| held != NO_KEY).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Holds `quest` as finished. False when it already was, or when no place is free.
    pub fn finish(&mut self, quest: Key) -> bool {
        if quest == NO_KEY || self.has(quest) {
            return false;
        }
        let Some(free) = self.place(NO_KEY) else {
            return false;
        };
        self.quests[free] = quest;
        self.claimed[free / 64] &= !(1 << (free % 64));
        true
    }

    /// Marks the rewards of `quest` taken. False unless it is finished and they weren't.
    pub fn claim(&mut self, quest: Key) -> bool {
        if quest == NO_KEY || self.claimed(quest) {
            return false;
        }
        let Some(place) = self.place(quest) else {
            return false;
        };
        self.claimed[place / 64] |= 1 << (place % 64);
        true
    }
}

impl Counting {
    const FREE: Count = Count {
        quest: NO_KEY,
        tasks: [0; TASKS_MOST],
    };
    pub const NONE: Counting = Counting {
        quests: [Counting::FREE; COUNTING_MOST],
    };

    fn place(&self, quest: Key) -> Option<usize> {
        self.quests.iter().position(|count| count.quest == quest)
    }

    /// The counts of `quest`, all 0 for one not counted yet.
    pub fn of(&self, quest: Key) -> [u32; TASKS_MOST] {
        match self.place(quest) {
            Some(place) if quest != NO_KEY => self.quests[place].tasks,
            _ => [0; TASKS_MOST],
        }
    }

    /// Raises task `task` of `quest` to `count`; a lower `count` changes nothing. False when the
    /// count is lost: no place is free for another quest.
    pub fn raise(&mut self, quest: Key, task: usize, count: u32) -> bool {
        if quest == NO_KEY || task >= TASKS_MOST {
            return false;
        }
        let Some(place) = self.place(quest).or_else(|| self.place(NO_KEY)) else {
            return false;
        };
        let held = &mut self.quests[place];
        held.quest = quest;
        held.tasks[task] = held.tasks[task].max(count);
        true
    }

    /// Gives the place of `quest` back.
    pub fn forget(&mut self, quest: Key) {
        if let Some(place) = self.place(quest).filter(|_| quest != NO_KEY) {
            self.quests[place] = Counting::FREE;
        }
    }

    /// Gives back the places of quests `known` doesn't hold: those of a mod that left.
    pub fn keep(&mut self, known: impl Fn(Key) -> bool) {
        for count in &mut self.quests {
            if !known(count.quest) {
                *count = Counting::FREE;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use super::*;

    #[test]
    fn the_saved_layouts_are_pinned() {
        // Saved with players: a changed layout takes new component names.
        assert_eq!(offset_of!(Finished, quests), 0);
        assert_eq!(offset_of!(Finished, claimed), 8 * FINISHED_MOST);
        assert_eq!(size_of::<Finished>(), 8 * FINISHED_MOST + FINISHED_MOST / 8);
        assert_eq!(offset_of!(Count, tasks), 8);
        assert_eq!(size_of::<Count>(), 8 + 4 * TASKS_MOST);
        assert_eq!(size_of::<Counting>(), COUNTING_MOST * size_of::<Count>());
    }

    #[test]
    fn a_quest_is_finished_once_and_claimed_once() {
        let mut finished = Finished::NONE;
        assert!(!finished.has(7), "nothing yet");
        assert!(finished.finish(7), "the first time");
        assert!(!finished.finish(7), "not twice");
        assert!(
            finished.has(7) && !finished.claimed(7),
            "finished, rewards waiting"
        );
        assert!(finished.claim(7), "claimed");
        assert!(!finished.claim(7), "not twice");
        assert_eq!(finished.len(), 1);
    }

    #[test]
    fn an_unfinished_quest_gives_nothing() {
        let mut finished = Finished::NONE;
        assert!(!finished.claim(7), "no rewards before it is done");
        assert!(!finished.claimed(7));
    }

    #[test]
    fn no_key_is_never_finished() {
        let mut finished = Finished::NONE;
        assert!(!finished.has(NO_KEY), "free places are no quest");
        assert!(!finished.finish(NO_KEY) && !finished.claim(NO_KEY));
        assert!(finished.is_empty());
    }

    #[test]
    fn a_full_list_takes_no_more() {
        let mut finished = Finished::NONE;
        for quest in 1..=FINISHED_MOST as Key {
            assert!(finished.finish(quest), "quest {quest} fits");
        }
        assert!(!finished.finish(9999), "one more does not");
        assert!(
            finished.claim(FINISHED_MOST as Key),
            "the last place has its own bit"
        );
        assert!(!finished.claimed(FINISHED_MOST as Key - 1), "and only it");
    }

    #[test]
    fn counts_only_go_up() {
        let mut counting = Counting::NONE;
        assert!(counting.raise(7, 1, 3));
        assert!(
            counting.raise(7, 1, 2),
            "a lower count is kept out, not lost"
        );
        assert_eq!(counting.of(7), [0, 3, 0, 0]);
        assert_eq!(counting.of(8), [0; TASKS_MOST], "another quest has none");
    }

    #[test]
    fn a_forgotten_quest_frees_its_place() {
        let mut counting = Counting::NONE;
        for quest in 1..=COUNTING_MOST as Key {
            assert!(counting.raise(quest, 0, 1), "quest {quest} fits");
        }
        assert!(!counting.raise(999, 0, 1), "no place for one more");
        counting.forget(3);
        assert_eq!(counting.of(3), [0; TASKS_MOST], "gone");
        assert!(counting.raise(999, 0, 1), "its place is free again");
    }

    #[test]
    fn quests_that_left_the_book_give_their_places_back() {
        let mut counting = Counting::NONE;
        for quest in 1..=COUNTING_MOST as Key {
            counting.raise(quest, 0, 1);
        }
        counting.keep(|quest| quest != 3 && quest != 4);
        assert_eq!(counting.of(3), [0; TASKS_MOST], "gone");
        assert_eq!(counting.of(5)[0], 1, "a known quest keeps its count");
        assert!(counting.raise(998, 0, 1) && counting.raise(999, 0, 1));
        assert!(!counting.raise(1000, 0, 1), "two places came free, no more");
    }

    #[test]
    fn a_task_past_the_last_is_not_counted() {
        let mut counting = Counting::NONE;
        assert!(!counting.raise(7, TASKS_MOST, 1));
        assert!(!counting.raise(NO_KEY, 0, 1), "nor is no quest");
    }
}
