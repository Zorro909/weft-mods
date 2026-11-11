//! Which things each task counts, found once the game is loaded: a task names them as text
//! (`cogwork:spring_engine`, `#ingots/iron`), and what happens in the world comes as ids.

use std::collections::HashMap;
use std::sync::OnceLock;

use almanac_api::{Book, Kind, Task, book};
use weft_sdk::abi::entry_kind;
use weft_sdk::*;

/// Where a task is: the place of its quest in the book, and its own among the quest's tasks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TaskAt {
    pub quest: usize,
    pub task: usize,
}

/// A `Kind::Have` task and the items it counts.
pub struct Held {
    pub at: TaskAt,
    pub items: Vec<u32>,
}

#[derive(Default)]
pub struct Targets {
    /// The tasks that count a thing of a kind: an item, a block or an entity kind, by its id.
    counting: HashMap<(Kind, u32), Vec<TaskAt>>,
    pub held: Vec<Held>,
}

impl Targets {
    /// The tasks that count `thing` when `kind` happens to it.
    pub fn counting(&self, kind: Kind, thing: u32) -> &[TaskAt] {
        self.counting
            .get(&(kind, thing))
            .map_or(&[], |tasks| tasks.as_slice())
    }

    fn of(book: &Book, things: impl Fn(Kind, &str) -> Vec<u32>) -> Targets {
        let mut targets = Targets::default();
        let tasks = book.quests.iter().enumerate().flat_map(|(quest, held)| {
            held.tasks
                .iter()
                .enumerate()
                .map(move |(task, asked)| (TaskAt { quest, task }, asked))
        });
        for (at, task) in tasks {
            targets.add(at, task, &things);
        }
        targets
    }

    fn add(&mut self, at: TaskAt, task: &Task, things: impl Fn(Kind, &str) -> Vec<u32>) {
        match task.kind() {
            Some(Kind::Count) | None => {}
            Some(Kind::Have) => self.held.push(Held {
                at,
                items: things(Kind::Have, &task.what),
            }),
            Some(kind) => {
                for thing in things(kind, &task.what) {
                    self.counting.entry((kind, thing)).or_default().push(at);
                }
            }
        }
    }
}

/// Every task's things, found the first time it is asked for: the book and the registry don't
/// change while the game runs.
pub fn targets() -> &'static Targets {
    static TARGETS: OnceLock<Targets> = OnceLock::new();
    TARGETS.get_or_init(|| Targets::of(book(), things))
}

/// The ids `what` names for a task of `kind`: items, blocks (each once, whatever its state) or
/// entity kinds; none for a name nothing has.
fn things(kind: Kind, what: &str) -> Vec<u32> {
    let tagged = what.strip_prefix('#');
    let mut things = match kind {
        Kind::Have | Kind::Use => match tagged {
            Some(tag) => entries_with(entry_kind::ITEM, tag),
            None => Item::lookup(what).into_iter().collect(),
        },
        Kind::Place | Kind::Dig => {
            let states = match tagged {
                Some(tag) => entries_with(entry_kind::BLOCK, tag),
                None => Block::lookup(what).into_iter().collect(),
            };
            states.into_iter().map(block_of).collect()
        }
        Kind::Kill => match tagged {
            Some(tag) => entries_with(entry_kind::ENTITY, tag),
            None => EntityKind::lookup(what).into_iter().collect(),
        },
        Kind::Count => Vec::new(),
    };
    things.sort_unstable();
    things.dedup();
    if things.is_empty() && kind != Kind::Count {
        warn!("almanac: a task counts {what}, which names nothing");
    }
    things
}

#[cfg(test)]
mod tests {
    use almanac_api::{ChapterRecord, QuestRecord};

    use super::*;

    /// A book of one quest with `tasks`.
    fn book_of(tasks: Vec<Task>) -> Book {
        let chapter = ChapterRecord {
            name: "a:chapter".to_string(),
            ..ChapterRecord::default()
        };
        let quest = QuestRecord {
            name: "a:quest".to_string(),
            chapter: "a:chapter".to_string(),
            tasks,
            ..QuestRecord::default()
        };
        Book::of(vec![chapter], vec![quest]).0
    }

    /// The task at place `task` of the book's one quest.
    fn task(task: usize) -> TaskAt {
        TaskAt { quest: 0, task }
    }

    /// Every name stands for the ids 1 and 2.
    fn two_things(_: Kind, _: &str) -> Vec<u32> {
        vec![1, 2]
    }

    #[test]
    fn a_task_is_found_by_each_thing_it_counts() {
        let book = book_of(vec![
            Task::new(Kind::Place, "#engines", 1),
            Task::new(Kind::Dig, "a:ore", 1),
        ]);
        let targets = Targets::of(&book, two_things);
        assert_eq!(
            targets.counting(Kind::Place, 1),
            [task(0)],
            "the first thing"
        );
        assert_eq!(
            targets.counting(Kind::Place, 2),
            [task(0)],
            "and the second"
        );
        assert_eq!(targets.counting(Kind::Dig, 1), [task(1)], "by its own kind");
        assert!(targets.counting(Kind::Kill, 1).is_empty(), "no task kills");
        assert!(
            targets.counting(Kind::Place, 3).is_empty(),
            "nor counts another thing"
        );
    }

    #[test]
    fn held_items_are_looked_at_and_not_awaited() {
        let book = book_of(vec![Task::new(Kind::Have, "a:cog", 4)]);
        let targets = Targets::of(&book, two_things);
        assert_eq!(targets.held.len(), 1);
        assert_eq!(
            (targets.held[0].at, targets.held[0].items.as_slice()),
            (task(0), [1, 2].as_slice())
        );
        assert!(
            targets.counting(Kind::Have, 1).is_empty(),
            "nothing happens to a held item"
        );
    }

    #[test]
    fn a_mods_own_count_waits_for_no_thing() {
        let book = book_of(vec![Task::counted("burned", 200, "a.text")]);
        let targets = Targets::of(&book, two_things);
        assert!(targets.held.is_empty() && targets.counting.is_empty());
    }
}
