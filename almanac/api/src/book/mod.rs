//! Every mod's chapters and quests, joined: each quest in its chapter, behind the quests it
//! comes after. `read::book()` makes one from the registry once the game is loaded.

mod join;

use strum::{IntoStaticStr, VariantArray};

use crate::layout;
use crate::names::Key;
use crate::progress::Finished;
use crate::record::{Reward, Task};

/// A chapter as its record says.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChapterRecord {
    pub id: u32,
    pub name: String,
    pub icon: u32,
    pub order: i32,
    pub tile: String,
    pub own_page: bool,
}

/// A quest as its record says: its chapter and the quests it comes after still by name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QuestRecord {
    pub id: u32,
    pub name: String,
    pub chapter: String,
    pub icon: u32,
    pub order: i32,
    pub after: Vec<String>,
    pub tasks: Vec<Task>,
    pub rewards: Vec<Reward>,
}

/// A chapter as the book holds it, with its quests found.
#[derive(Debug, PartialEq)]
pub struct Chapter {
    /// Its record's id, which `Shown` and the channels name it by.
    pub id: u32,
    /// `cogwork:cogwork`.
    pub name: String,
    /// The item on its tab, 0 for none.
    pub icon: u32,
    /// The class of its tiles.
    pub tile: String,
    /// Whether its mod draws the page.
    pub own_page: bool,
    /// Its quests in page order, as places in `Book::quests`.
    pub quests: Vec<usize>,
}

/// A quest as the book holds it, with its chapter and the quests before it found.
#[derive(Debug, PartialEq)]
pub struct Quest {
    /// Its record's id, which events and channels name it by.
    pub id: u32,
    /// `cogwork:wound_up`.
    pub name: String,
    /// What a save holds it as.
    pub key: Key,
    /// Its chapter's place in `Book::chapters`.
    pub chapter: usize,
    /// The item on its tile, 0 for none.
    pub icon: u32,
    /// The quests to finish first, as places in `Book::quests`.
    pub after: Vec<usize>,
    /// At most `TASKS_MOST`, each of a known kind.
    pub tasks: Vec<Task>,
    pub rewards: Vec<Reward>,
}

/// How far a player is with a quest. The screen names one by its name in snake case
/// (`almanac.state.ready`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum State {
    /// A quest it comes after isn't finished.
    Locked,
    Open,
    /// Finished, its rewards waiting.
    Ready,
    /// Finished, its rewards taken.
    Claimed,
}

/// Every chapter and quest, joined once the game is loaded.
#[derive(Debug, Default, PartialEq)]
pub struct Book {
    /// In tab order.
    pub chapters: Vec<Chapter>,
    /// Chapter by chapter, each chapter's in page order.
    pub quests: Vec<Quest>,
}

impl Book {
    /// The place of the quest whose record has `id`.
    pub fn quest(&self, id: u32) -> Option<usize> {
        self.quests.iter().position(|quest| quest.id == id)
    }

    /// The place of quest `name` (`cogwork:wound_up`).
    pub fn quest_named(&self, name: &str) -> Option<usize> {
        self.quests.iter().position(|quest| quest.name == name)
    }

    /// The place of the chapter whose record has `id`.
    pub fn chapter(&self, id: u32) -> Option<usize> {
        self.chapters.iter().position(|chapter| chapter.id == id)
    }

    /// The place of chapter `name` (`cogwork:cogwork`).
    pub fn chapter_named(&self, name: &str) -> Option<usize> {
        self.chapters
            .iter()
            .position(|chapter| chapter.name == name)
    }

    /// The first quest `quest` still waits for.
    pub fn waits_for(&self, quest: usize, finished: &Finished) -> Option<usize> {
        self.quests[quest]
            .after
            .iter()
            .copied()
            .find(|&before| !finished.has(self.quests[before].key))
    }

    pub fn state(&self, quest: usize, finished: &Finished) -> State {
        let key = self.quests[quest].key;
        if finished.claimed(key) {
            State::Claimed
        } else if finished.has(key) {
            State::Ready
        } else if self.waits_for(quest, finished).is_some() {
            State::Locked
        } else {
            State::Open
        }
    }

    /// The chains of chapter `chapter`, a row each (`layout::rows`).
    pub fn rows(&self, chapter: usize) -> Vec<Vec<usize>> {
        layout::rows(&self.chapters[chapter].quests, |quest| {
            self.quests[quest]
                .after
                .iter()
                .copied()
                .find(|&before| self.quests[before].chapter == chapter)
        })
    }

    /// The quest a chapter opens at: the first whose rewards wait, else the first open one, else
    /// its first.
    pub fn opens_at(&self, chapter: usize, finished: &Finished) -> Option<usize> {
        let quests = &self.chapters[chapter].quests;
        let first = |state: State| {
            quests
                .iter()
                .copied()
                .find(|&quest| self.state(quest, finished) == state)
        };
        first(State::Ready)
            .or_else(|| first(State::Open))
            .or_else(|| quests.first().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::names::key_of;
    use crate::record::Kind;

    fn chapter(name: &str, order: i32) -> ChapterRecord {
        ChapterRecord {
            name: name.to_string(),
            order,
            ..ChapterRecord::default()
        }
    }

    fn quest(name: &str, chapter: &str, order: i32, after: &[&str]) -> QuestRecord {
        QuestRecord {
            name: name.to_string(),
            chapter: chapter.to_string(),
            order,
            after: after.iter().map(|name| name.to_string()).collect(),
            tasks: vec![Task::new(Kind::Have, "base:coal", 1)],
            ..QuestRecord::default()
        }
    }

    /// Two chapters: `a:power` with a chain of three, and `b:tools` with one quest.
    fn two_chapters() -> Book {
        let chapters = vec![chapter("b:tools", 20), chapter("a:power", 10)];
        let quests = vec![
            quest("a:third", "a:power", 30, &["a:second"]),
            quest("b:hammer", "b:tools", 10, &["a:first"]),
            quest("a:first", "a:power", 10, &[]),
            quest("a:second", "a:power", 20, &["a:first"]),
        ];
        let (book, problems) = Book::of(chapters, quests);
        assert!(problems.is_empty(), "{problems:?}");
        book
    }

    fn names(book: &Book, quests: &[usize]) -> Vec<&'static str> {
        let known = ["a:first", "a:second", "a:third", "b:hammer"];
        quests
            .iter()
            .map(|&quest| {
                known
                    .iter()
                    .copied()
                    .find(|name| *name == book.quests[quest].name)
                    .unwrap_or("?")
            })
            .collect()
    }

    #[test]
    fn chapters_and_quests_stand_in_their_order() {
        let book = two_chapters();
        let tabs: Vec<&str> = book
            .chapters
            .iter()
            .map(|chapter| chapter.name.as_str())
            .collect();
        assert_eq!(tabs, ["a:power", "b:tools"], "tabs by order");
        assert_eq!(
            names(&book, &book.chapters[0].quests),
            ["a:first", "a:second", "a:third"]
        );
        assert_eq!(names(&book, &book.chapters[1].quests), ["b:hammer"]);
    }

    #[test]
    fn a_chain_is_a_row_and_another_chapters_quest_starts_one() {
        let book = two_chapters();
        assert_eq!(book.rows(0).len(), 1, "the chain");
        assert_eq!(
            book.rows(1),
            [[3]],
            "what it comes after is on another page"
        );
    }

    #[test]
    fn a_quest_opens_when_what_it_comes_after_is_finished() {
        let book = two_chapters();
        let mut finished = Finished::NONE;
        let second = book
            .quest_named("a:second")
            .expect("a:second is in the book");
        assert_eq!(book.state(second, &finished), State::Locked);
        assert_eq!(
            names(
                &book,
                &[book.waits_for(second, &finished).expect("it waits")]
            ),
            ["a:first"]
        );
        finished.finish(key_of("a:first"));
        assert_eq!(
            book.state(second, &finished),
            State::Open,
            "the first is done"
        );
        assert_eq!(book.state(0, &finished), State::Ready, "its rewards wait");
        finished.claim(key_of("a:first"));
        assert_eq!(book.state(0, &finished), State::Claimed);
    }

    #[test]
    fn a_chapter_opens_at_waiting_rewards_then_at_open_work() {
        let book = two_chapters();
        let mut finished = Finished::NONE;
        assert_eq!(
            names(&book, &[book.opens_at(0, &finished).expect("a quest")]),
            ["a:first"],
            "the open one"
        );
        finished.finish(key_of("a:first"));
        assert_eq!(
            names(&book, &[book.opens_at(0, &finished).expect("a quest")]),
            ["a:first"],
            "its rewards wait"
        );
        finished.claim(key_of("a:first"));
        assert_eq!(
            names(&book, &[book.opens_at(0, &finished).expect("a quest")]),
            ["a:second"],
            "the next open one"
        );
    }

    #[test]
    fn an_empty_chapter_opens_at_nothing() {
        let (book, _) = Book::of(vec![chapter("a:power", 10)], Vec::new());
        assert_eq!(book.opens_at(0, &Finished::NONE), None);
        assert!(book.rows(0).is_empty());
    }
}
