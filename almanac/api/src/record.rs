//! What a quest and a chapter are in the registry: a record with the tag and components below.
//! The builders (`chapter`, `quest`, `edit`) write them, and any mod may read or change them with
//! the SDK's own `r.edit(..)`.

use strum::{IntoStaticStr, VariantArray};
use weft_sdk::{Component, Tag, abi};

/// On every chapter's record.
pub static CHAPTER: Tag = Tag::named("almanac:chapter");
/// On every quest's record.
pub static QUEST: Tag = Tag::named("almanac:quest");
/// The item a chapter's tab or a quest's tile shows, by name.
pub static ICON: Component<String> = Component::named("almanac:icon");
/// Where a chapter stands among the tabs, or a quest among its chapter's: lower comes first.
pub static ORDER: Component<i32> = Component::named("almanac:order");
/// The class of a chapter's tiles (`gui/<class>.png`, with `_done`, `_ready` and `_locked`).
pub static TILE: Component<String> = Component::named("almanac:tile");
/// On a chapter whose mod draws its page itself (`Chapter::page`).
pub static OWN_PAGE: Tag = Tag::named("almanac:own_page");
/// The chapter a quest is in, by its full name.
pub static IN_CHAPTER: Component<String> = Component::named("almanac:in_chapter");
/// The quests to finish before this one opens, by their full names.
pub static AFTER: Component<Vec<String>> = Component::named("almanac:after");
/// What a quest asks, in the order shown.
pub static TASKS: Component<Vec<Task>> = Component::named("almanac:tasks");
/// What a quest hands over when it is claimed.
pub static REWARDS: Component<Vec<Reward>> = Component::named("almanac:rewards");

/// Where a quest or chapter stands when nothing says: after those that do.
pub const LAST: i32 = 1000;
/// The tiles' class when a chapter names none.
pub const STOCK_TILE: &str = "almanac_tile";

abi::data! {
    /// One thing to do for a quest. `name` is how edits find it, `kind` a `Kind` by its name,
    /// `what` the things that count (a name or `#tag`, nothing for `Kind::Count`), `count` how
    /// many, and `text` the translation shown for it (empty: one made from the kind).
    pub struct Task { pub name: String, pub kind: String, pub what: String, pub count: u32, pub text: String }
}

abi::data! {
    /// Items a quest hands over when it is claimed. `name` is how edits find it.
    pub struct Reward { pub name: String, pub item: String, pub count: u32 }
}

/// What a task counts. A task's record holds it by its name in snake case (`have`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
pub enum Kind {
    /// Items in the inventory at once.
    Have,
    /// Blocks placed.
    Place,
    /// Blocks broken.
    Dig,
    /// Mobs killed.
    Kill,
    /// Items used: food eaten, a tool put to use.
    Use,
    /// Whatever the quest's mod reports with `advance`.
    Count,
}

impl Kind {
    pub fn name(self) -> &'static str {
        self.into()
    }

    /// The kind a record names, `None` for a word no kind has.
    pub fn named(name: &str) -> Option<Kind> {
        Kind::VARIANTS
            .iter()
            .copied()
            .find(|kind| kind.name() == name)
    }
}

impl Task {
    /// A task counting `count` of `what`, named `<kind>/<what>`.
    pub fn new(kind: Kind, what: &str, count: u32) -> Task {
        Task {
            name: format!("{}/{what}", kind.name()),
            kind: kind.name().to_string(),
            what: what.to_string(),
            count: count.max(1),
            text: String::new(),
        }
    }

    /// A task the quest's mod counts itself, shown as translation `text`.
    pub fn counted(name: &str, count: u32, text: &str) -> Task {
        Task {
            name: name.to_string(),
            kind: Kind::Count.name().to_string(),
            what: String::new(),
            count: count.max(1),
            text: text.to_string(),
        }
    }

    pub fn kind(&self) -> Option<Kind> {
        Kind::named(&self.kind)
    }
}

impl Reward {
    /// `count` of `item`, named like the item.
    pub fn new(item: &str, count: u32) -> Reward {
        Reward {
            name: item.to_string(),
            item: item.to_string(),
            count: count.max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use weft_sdk::abi::data::{Data, bytes_of, from_bytes};

    use super::*;

    #[test]
    fn every_kind_has_a_name_of_its_own() {
        for &kind in Kind::VARIANTS {
            assert_eq!(Kind::named(kind.name()), Some(kind), "{kind:?} by its name");
        }
        // Records hold these words: a changed one loses every task written with it.
        assert_eq!(Kind::Have.name(), "have");
        assert_eq!(Kind::Count.name(), "count");
        assert_eq!(Kind::named("juggle"), None, "an unknown word is no kind");
    }

    #[test]
    fn a_task_survives_the_registry() {
        let task = Task::new(Kind::Place, "cogwork:spring_engine", 3);
        assert_eq!(
            task.name, "place/cogwork:spring_engine",
            "named for what it counts"
        );
        let back: Task = from_bytes(&bytes_of(&task)).expect("a task reads back");
        assert_eq!(back, task, "as it was written");
        assert_eq!(back.kind(), Some(Kind::Place));
    }

    #[test]
    fn edits_find_tasks_and_rewards_by_name() {
        // List edits look an element up by its first field when that is `name:str`.
        assert!(
            Task::schema().starts_with("<name:str,"),
            "{}",
            Task::schema()
        );
        assert!(
            Reward::schema().starts_with("<name:str,"),
            "{}",
            Reward::schema()
        );
    }

    #[test]
    fn nothing_counts_to_zero() {
        assert_eq!(Task::new(Kind::Have, "base:coal", 0).count, 1, "a task");
        assert_eq!(Task::counted("burned", 0, "").count, 1, "a counted one");
        assert_eq!(Reward::new("base:coal", 0).count, 1, "a reward");
    }
}
