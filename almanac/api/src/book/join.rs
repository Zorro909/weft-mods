//! Joining the records into a book: sorting them, finding each quest's chapter and the quests it
//! comes after, and leaving out what can't be joined, with a line for the log each time.

use super::*;
use crate::names::key_of;
use crate::progress::TASKS_MOST;

impl Book {
    /// The book of these records, and what was left out of it, each as a line for the log.
    pub fn of(mut chapters: Vec<ChapterRecord>, quests: Vec<QuestRecord>) -> (Book, Vec<String>) {
        let mut problems = Vec::new();
        chapters.sort_by(|one, other| (one.order, &one.name).cmp(&(other.order, &other.name)));
        let records = in_page_order(quests, &chapters, &mut problems);
        let names: Vec<String> = records.iter().map(|quest| quest.name.clone()).collect();
        let quests: Vec<Quest> = records
            .into_iter()
            .map(|quest| joined(quest, &chapters, &names, &mut problems))
            .collect();
        let chapters = chapters
            .into_iter()
            .enumerate()
            .map(|(place, chapter)| with_quests(chapter, place, &quests))
            .collect();
        (Book { chapters, quests }, problems)
    }
}

/// The place of chapter `name` among `chapters`.
fn place_of(chapters: &[ChapterRecord], name: &str) -> Option<usize> {
    chapters.iter().position(|chapter| chapter.name == name)
}

/// The quests of the chapters there are, chapter by chapter, each chapter's by order and then
/// by name.
fn in_page_order(
    mut quests: Vec<QuestRecord>,
    chapters: &[ChapterRecord],
    problems: &mut Vec<String>,
) -> Vec<QuestRecord> {
    quests.retain(|quest| {
        let known = place_of(chapters, &quest.chapter).is_some();
        if !known {
            problems.push(format!(
                "quest {} is in chapter {}, which nobody registered",
                quest.name, quest.chapter
            ));
        }
        known
    });
    quests.sort_by_cached_key(|quest| {
        let chapter = place_of(chapters, &quest.chapter);
        (chapter, quest.order, quest.name.clone())
    });
    quests
}

/// `quest` with its chapter and the quests it comes after found, and its tasks checked. `names`
/// are the book's quests in page order.
fn joined(
    quest: QuestRecord,
    chapters: &[ChapterRecord],
    names: &[String],
    problems: &mut Vec<String>,
) -> Quest {
    Quest {
        id: quest.id,
        key: key_of(&quest.name),
        chapter: place_of(chapters, &quest.chapter).unwrap_or(0),
        icon: quest.icon,
        after: found(&quest, names, problems),
        tasks: counted(&quest.name, quest.tasks, problems),
        rewards: quest.rewards,
        name: quest.name,
    }
}

/// The places of the quests `quest` comes after, without those that are no quest.
fn found(quest: &QuestRecord, names: &[String], problems: &mut Vec<String>) -> Vec<usize> {
    let mut after = Vec::new();
    for before in &quest.after {
        match names.iter().position(|name| name == before) {
            Some(place) => after.push(place),
            None => problems.push(format!(
                "quest {} comes after {before}, which is no quest",
                quest.name
            )),
        }
    }
    after
}

/// The tasks of quest `quest` the book counts: those of a known kind, the first `TASKS_MOST`.
fn counted(quest: &str, mut tasks: Vec<Task>, problems: &mut Vec<String>) -> Vec<Task> {
    tasks.retain(|task| {
        let known = task.kind().is_some();
        if !known {
            problems.push(format!(
                "quest {quest}: task {} is of no known kind ({})",
                task.name, task.kind
            ));
        }
        known
    });
    if tasks.len() > TASKS_MOST {
        problems.push(format!(
            "quest {quest} has {} tasks, the book counts {TASKS_MOST}",
            tasks.len()
        ));
        tasks.truncate(TASKS_MOST);
    }
    if tasks.is_empty() {
        problems.push(format!(
            "quest {quest} asks for nothing, so nobody can finish it"
        ));
    }
    tasks
}

/// `chapter`, the one at `place`, with its quests out of `quests`.
fn with_quests(chapter: ChapterRecord, place: usize, quests: &[Quest]) -> Chapter {
    Chapter {
        id: chapter.id,
        name: chapter.name,
        icon: chapter.icon,
        tiles: State::VARIANTS
            .iter()
            .map(|state| format!("{}{}", chapter.tile, state.tile_suffix()))
            .collect(),
        own_page: chapter.own_page,
        quests: (0..quests.len())
            .filter(|&quest| quests[quest].chapter == place)
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::Kind;

    fn chapter(name: &str, order: i32) -> ChapterRecord {
        ChapterRecord {
            name: name.to_string(),
            order,
            ..ChapterRecord::default()
        }
    }

    /// A quest asking for one piece of coal.
    fn quest(name: &str, chapter: &str, order: i32) -> QuestRecord {
        QuestRecord {
            name: name.to_string(),
            chapter: chapter.to_string(),
            order,
            tasks: vec![Task::new(Kind::Have, "base:coal", 1)],
            ..QuestRecord::default()
        }
    }

    #[test]
    fn chapters_stand_by_order_and_then_by_name() {
        let chapters = vec![
            chapter("b:late", 20),
            chapter("b:early", 10),
            chapter("a:early", 10),
        ];
        let (book, problems) = Book::of(chapters, Vec::new());
        let tabs: Vec<&str> = book
            .chapters
            .iter()
            .map(|chapter| chapter.name.as_str())
            .collect();
        assert_eq!(tabs, ["a:early", "b:early", "b:late"]);
        assert!(problems.is_empty(), "{problems:?}");
    }

    #[test]
    fn a_quest_of_no_chapter_is_left_out_and_named() {
        let mut lost = quest("a:lost", "a:nowhere", 10);
        lost.id = 7;
        let (book, problems) = Book::of(vec![chapter("a:power", 10)], vec![lost]);
        assert_eq!(book.quest(7), None, "gone from the book");
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(
            problems[0].contains("a:lost") && problems[0].contains("a:nowhere"),
            "{problems:?}"
        );
    }

    #[test]
    fn what_is_no_quest_is_not_waited_for() {
        let mut odd = quest("a:odd", "a:power", 10);
        odd.after = vec!["a:ghost".to_string()];
        let (book, problems) = Book::of(vec![chapter("a:power", 10)], vec![odd]);
        assert!(book.quests[0].after.is_empty(), "it opens at once");
        assert_eq!(problems.len(), 1, "{problems:?}");
    }

    #[test]
    fn a_quest_that_asks_for_nothing_is_named() {
        let mut idle = quest("a:idle", "a:power", 10);
        idle.tasks.clear();
        let (_, problems) = Book::of(vec![chapter("a:power", 10)], vec![idle]);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("a:idle"), "{problems:?}");
    }

    #[test]
    fn unknown_and_surplus_tasks_are_dropped() {
        let mut busy = quest("a:busy", "a:power", 10);
        busy.tasks.push(Task {
            kind: "juggle".to_string(),
            ..Task::default()
        });
        let fitting =
            (0..=TASKS_MOST).map(|more| Task::new(Kind::Have, "base:coal", more as u32 + 1));
        busy.tasks.extend(fitting);
        let (book, problems) = Book::of(vec![chapter("a:power", 10)], vec![busy]);
        let kept = &book.quests[0].tasks;
        assert_eq!(kept.len(), TASKS_MOST, "the first that fit");
        assert!(
            kept.iter().all(|task| task.kind() == Some(Kind::Have)),
            "none of an unknown kind"
        );
        assert_eq!(problems.len(), 2, "{problems:?}");
    }
}
