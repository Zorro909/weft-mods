//! The interface between a mod's quests and the `almanac` mod that keeps the book.
//!
//! A quest is a record in the registry: its chapter, icon, tasks and rewards are components on
//! it (`record`). A mod writes its own with `chapter` and `quest`, and another mod changes or
//! removes them with `edit` and `remove`, as it would a recipe. The almanac mod reads them all
//! once the game is loaded, counts what players do, and shows the book; this crate has the parts
//! a mod draws a page of its own from (`Page`).
//!
//! It talks to a running mod. A mod whose quests are an extra lists it as
//! `declare_mod!(deps: [..], after: ["almanac"], ..)`: without the almanac mod its records are
//! read by nobody. One that needs the book lists `almanac` in `deps`.

mod book;
mod chapter;
mod events;
mod layout;
mod names;
mod page;
mod progress;
mod quest;
mod read;
mod record;

pub use book::{Book, Chapter as BookChapter, ChapterRecord, Quest, QuestRecord, State};
pub use chapter::{Chapter, chapter};
pub use events::{ADVANCE, Advance, CLAIM, CLAIMED, Claim, DONE, Done, SHOW, advance};
pub use names::{Key, NO_KEY, chapter_record, chapter_text, key_of, quest_record, quest_text};
pub use page::Page;
pub use progress::{
    COUNTING, COUNTING_MOST, Count, Counting, FINISHED, FINISHED_MOST, Finished, NOTHING, SHOWN,
    Shown, TASKS_MOST,
};
pub use quest::{QuestBuilder, edit, quest, remove};
pub use read::book;
pub use record::{
    AFTER, CHAPTER, ICON, IN_CHAPTER, Kind, ORDER, OWN_PAGE, QUEST, REWARDS, Reward, TASKS, TILE,
    Task,
};
use weft_sdk::Menu;

/// The book's screen. A chapter's own page is a part of it (`Chapter::page`).
pub static BOOK: Menu = Menu::named("almanac:book");
