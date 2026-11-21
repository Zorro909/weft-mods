//! Reads the book out of the registry. Only once the game is loaded: records can't be read
//! while mods register, since only then are everyone's edits in.

use std::sync::OnceLock;

use weft_sdk::abi::entry_kind::RECORD;
use weft_sdk::{Item, entries_with, name_of, warn};

use crate::book::{Book, ChapterRecord, QuestRecord};
use crate::names::{chapter_of, quest_of};
use crate::record::{
    AFTER, CHAPTER, ICON, IN_CHAPTER, LAST, ORDER, OWN_PAGE, QUEST, REWARDS, STOCK_TILE, TASKS,
    TILE,
};

/// Every mod's chapters and quests, read the first time it is asked for: records don't change
/// while the game runs. Each mod that asks reads its own.
pub fn book() -> &'static Book {
    static BOOK: OnceLock<Book> = OnceLock::new();
    BOOK.get_or_init(|| {
        let (book, problems) = Book::of(chapters(), quests());
        for problem in problems {
            warn!("almanac: {problem}");
        }
        book
    })
}

fn chapters() -> Vec<ChapterRecord> {
    entries_with(RECORD, CHAPTER.name())
        .into_iter()
        .filter_map(|id| {
            Some(ChapterRecord {
                id,
                name: chapter_of(name_of(RECORD, id))?,
                icon: icon(id),
                order: ORDER.on_record(id).unwrap_or(LAST),
                tile: TILE.on_record(id).unwrap_or_else(|| STOCK_TILE.to_string()),
                own_page: OWN_PAGE.on_record(id),
            })
        })
        .collect()
}

fn quests() -> Vec<QuestRecord> {
    entries_with(RECORD, QUEST.name())
        .into_iter()
        .filter_map(|id| {
            Some(QuestRecord {
                id,
                name: quest_of(name_of(RECORD, id))?,
                chapter: IN_CHAPTER.on_record(id).unwrap_or_default(),
                icon: icon(id),
                order: ORDER.on_record(id).unwrap_or(LAST),
                after: AFTER.on_record(id).unwrap_or_default(),
                tasks: TASKS.on_record(id).unwrap_or_default(),
                rewards: REWARDS.on_record(id).unwrap_or_default(),
            })
        })
        .collect()
}

/// The item record `id` names as its icon, 0 for none or for one no loaded mod has.
fn icon(id: u32) -> u32 {
    ICON.on_record(id)
        .and_then(|name| Item::lookup(&name))
        .unwrap_or(0)
}
