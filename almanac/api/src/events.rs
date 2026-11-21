//! What crosses between mods, and between a player's client and the server, while the game
//! runs. Quests and chapters are named by their records' ids here, which every mod and both
//! sides share; `Book::quest(id)` finds one.

use weft_sdk::{Channel, Event, Record, World, value};

use crate::names::{Key, key_of, quest_record};
use crate::progress::Shown;

value! {
    /// `player` finished `quest` of `chapter`, or took its rewards.
    pub struct Done { pub player: u64, pub quest: u32, pub chapter: u32 }
}

/// Fired on the server when a player finishes a quest's tasks.
pub static DONE: Event<Done> = Event::named("almanac:done");
/// Fired on the server when a player takes a quest's rewards: by the button, or for a quest that
/// hands over no items right after `DONE`, once everyone has heard that.
pub static CLAIMED: Event<Done> = Event::named("almanac:claimed");

value! {
    /// `player` got `amount` further with the task keyed `task` of `quest`.
    pub struct Advance { pub player: u64, pub task: Key, pub quest: u32, pub amount: u32 }
}

/// What `advance` fires for the almanac mod to count.
pub static ADVANCE: Event<Advance> = Event::named("almanac:advance");

value! {
    /// The player asks for the rewards of `quest`.
    pub struct Claim { pub quest: u32 }
}

/// Client to server, from the book's claim button.
pub static CLAIM: Channel<Claim> = Channel::named("almanac:claim");
/// Client to server: the player turned the book to this chapter and quest.
pub static SHOW: Channel<Shown> = Channel::named("almanac:show");

/// Counts `amount` more for `player` on task `task` of quest `quest` (`cogwork:burning_bright`),
/// a task made with `QuestBuilder::count`. On the server; nothing happens for a quest nobody
/// registered, or while the almanac mod isn't loaded.
///
/// ```ignore
/// almanac_api::advance(&world, player, "cogwork:burning_bright", "fuel_burned", burned);
/// ```
pub fn advance(world: &World, player: u64, quest: &str, task: &str, amount: u32) {
    let Some(quest) = Record::lookup(&quest_record(quest)) else {
        return;
    };
    world.emit(
        &ADVANCE,
        Advance {
            player,
            task: key_of(task),
            quest,
            amount,
        },
    );
}
