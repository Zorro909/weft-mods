//! How quests and chapters are named. A quest is `<mod>:<name>` (`cogwork:wound_up`) wherever a
//! mod or a player writes it; in the registry it is the record `<mod>:quest/<name>`, in a save
//! the hash of its name, and in the language files `quest.<mod>.<name>`.

use twox_hash::XxHash3_64;

/// A quest or chapter as a save keeps it: the hash of its full name, `NO_KEY` for none.
pub type Key = u64;
/// No quest: what a free place in a save holds.
pub const NO_KEY: Key = 0;

/// The key to save `name` under. XXH3's values are fixed by its specification, so keys saved
/// with a world stay right.
pub fn key_of(name: &str) -> Key {
    // 0 is no quest.
    XxHash3_64::oneshot(name.as_bytes()).max(1)
}

/// `name` with `namespace` in front, unless it has one or is a tag (`#logs`).
pub fn qualified(namespace: &str, name: &str) -> String {
    if name.starts_with('#') || name.contains(':') {
        name.to_string()
    } else {
        format!("{namespace}:{name}")
    }
}

/// The record quest `quest` is kept as: `cogwork:quest/wound_up`.
pub fn quest_record(quest: &str) -> String {
    record(quest, "quest")
}

/// The record chapter `chapter` is kept as: `cogwork:chapter/cogwork`.
pub fn chapter_record(chapter: &str) -> String {
    record(chapter, "chapter")
}

/// The quest record `record` keeps, `None` for a name of another shape.
pub fn quest_of(record: &str) -> Option<String> {
    kept(record, "quest")
}

/// The chapter record `record` keeps, `None` for a name of another shape.
pub fn chapter_of(record: &str) -> Option<String> {
    kept(record, "chapter")
}

/// The translation that names quest `quest`, `quest.cogwork.wound_up`; its description is the
/// same key with `.desc` after it.
pub fn quest_text(quest: &str) -> String {
    format!("quest.{}", quest.replace(':', "."))
}

/// The translation that names chapter `chapter`, `chapter.cogwork.cogwork`.
pub fn chapter_text(chapter: &str) -> String {
    format!("chapter.{}", chapter.replace(':', "."))
}

fn record(name: &str, folder: &str) -> String {
    match name.split_once(':') {
        Some((namespace, path)) => format!("{namespace}:{folder}/{path}"),
        None => format!("{folder}/{name}"),
    }
}

fn kept(record: &str, folder: &str) -> Option<String> {
    let (namespace, path) = record.split_once(':')?;
    let name = path.strip_prefix(folder)?.strip_prefix('/')?;
    Some(format!("{namespace}:{name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_quest_and_its_record_name_each_other() {
        let record = quest_record("cogwork:wound_up");
        assert_eq!(record, "cogwork:quest/wound_up", "the record");
        assert_eq!(
            quest_of(&record).as_deref(),
            Some("cogwork:wound_up"),
            "and back"
        );
    }

    #[test]
    fn a_chapter_is_no_quest() {
        let record = chapter_record("cogwork:cogwork");
        assert_eq!(record, "cogwork:chapter/cogwork", "the record");
        assert_eq!(quest_of(&record), None, "a chapter's record holds no quest");
        assert_eq!(
            chapter_of(&record).as_deref(),
            Some("cogwork:cogwork"),
            "and back"
        );
    }

    #[test]
    fn records_of_another_shape_are_refused() {
        assert_eq!(quest_of("structgen:hut"), None, "another mod's record");
        assert_eq!(quest_of("quest/wound_up"), None, "no namespace");
        assert_eq!(
            quest_of("cogwork:questionable"),
            None,
            "only the folder's start"
        );
    }

    #[test]
    fn names_take_the_namespace_they_lack() {
        assert_eq!(qualified("cogwork", "iron_cog"), "cogwork:iron_cog", "bare");
        assert_eq!(
            qualified("cogwork", "base:coal"),
            "base:coal",
            "already full"
        );
        assert_eq!(
            qualified("cogwork", "#ingots/iron"),
            "#ingots/iron",
            "a tag"
        );
    }

    #[test]
    fn texts_are_keyed_by_mod_and_name() {
        assert_eq!(quest_text("cogwork:wound_up"), "quest.cogwork.wound_up");
        assert_eq!(chapter_text("cogwork:power"), "chapter.cogwork.power");
    }

    /// XXH3 of `cogwork:wound_up`.
    const PINNED: Key = 0x9127_22c2_97a7_3c02;

    #[test]
    fn keys_are_pinned() {
        // Saved with players: a changed hash would forget every finished quest.
        assert_eq!(key_of("cogwork:wound_up"), PINNED);
        assert_ne!(
            key_of("cogwork:wound_up"),
            key_of("cogwork:wound_down"),
            "names differ"
        );
        assert_ne!(key_of(""), NO_KEY, "never the empty key");
    }
}
