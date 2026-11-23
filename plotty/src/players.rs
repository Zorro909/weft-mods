//! Who has played here: each player's key with the name they last joined under, kept in the
//! `players` table so that offline players have a name and commands can find them by it. A name
//! is shown and typed but never trusted: two players may share one.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use weft_sdk::{Arg, PLAYER_JOINED, Registry, ServerCommand, World};
use weftkit::PlayerKey;

use crate::tables::PLAYERS;

/// What Plotty knows of a player.
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Debug)]
pub struct PlayerEntry {
    /// The name they last joined under.
    pub name: String,
    /// Normal plots they may claim, where a moderator changed it from the setting.
    pub plot_allowance: Option<u32>,
    /// When they first joined, in seconds since 1970.
    pub first_seen: u64,
}

/// How a name typed in a command matched the players Plotty knows.
#[derive(PartialEq, Debug)]
pub enum Found {
    One(PlayerKey),
    Nobody,
    /// Several players have joined under that name.
    Several,
}

pub fn register(r: &mut Registry) {
    r.on_event(&PLAYER_JOINED, |batch| {
        let world = batch.world();
        for joined in batch.iter() {
            let Some(key) = PlayerKey::of(&world, joined.player) else {
                continue;
            };
            let name = world.player_name(joined.player).unwrap_or_default();
            crate::market::tell_sales(&world, joined.player, &key);
            if remember(key, name) {
                crate::woof::start(&key);
                crate::admin::spawn::send(&world, joined.player);
            }
        }
    });
}

/// Stores the name a player joined under; whether this is their first time.
fn remember(key: PlayerKey, name: String) -> bool {
    let mut first = false;
    PLAYERS.update(&key, |entry| {
        first = entry.first_seen == 0;
        if first {
            entry.first_seen = now();
        }
        entry.name = name;
    });
    first
}

/// The player known by `name`, any case; where several players have used it, the one of them
/// online in `world`.
pub fn find(world: &World, name: &str) -> Found {
    let mut known = Vec::new();
    PLAYERS.for_each(|key, entry| known.push((key, entry.name)));
    let online = world
        .players_iter()
        .filter_map(|player| Some((PlayerKey::of(world, player)?, world.player_name(player)?)))
        .collect();
    among(matching(known, name), matching(online, name))
}

/// Who a name finds among everyone known (`known`), or among the players `online` with it where
/// it names several.
fn among(known: Found, online: Found) -> Found {
    match (known, online) {
        (Found::Several, Found::One(key)) => Found::One(key),
        (known, _) => known,
    }
}

/// The player a command's `name` parameter names, with that name; where it names nobody or
/// several players, the command says so.
pub fn named(cmd: &mut ServerCommand) -> Option<(PlayerKey, String)> {
    let name = cmd.text("name").unwrap_or_default().to_string();
    match find(&cmd.world(), &name) {
        Found::One(key) => Some((key, name)),
        Found::Nobody => {
            cmd.reply("plotty.unknown_name", &[Arg::Text(&name)]);
            None
        }
        Found::Several => {
            cmd.reply("plotty.shared_name", &[Arg::Text(&name)]);
            None
        }
    }
}

/// The name Plotty last saw `key` under; its key's start where it never saw it.
pub fn name_of(key: &PlayerKey) -> String {
    known_name(key).unwrap_or_else(|| key.as_str().chars().take(8).collect())
}

/// The name `key` last joined under, if they joined.
pub fn known_name(key: &PlayerKey) -> Option<String> {
    PLAYERS
        .get(key)
        .map(|entry| entry.name)
        .filter(|name| !name.is_empty())
}

/// Every key Plotty has seen join.
pub fn all_keys() -> Vec<PlayerKey> {
    let mut keys = Vec::new();
    PLAYERS.for_each(|key, _| keys.push(key));
    keys
}

/// Which of `known` (keys with names) is called `name`, any case.
fn matching(known: Vec<(PlayerKey, String)>, name: &str) -> Found {
    let mut named = known
        .into_iter()
        .filter(|(_, known_name)| known_name.eq_ignore_ascii_case(name));
    match (named.next(), named.next()) {
        (None, _) => Found::Nobody,
        (Some((key, _)), None) => Found::One(key),
        (Some(_), Some(_)) => Found::Several,
    }
}

/// Seconds since 1970, by the server's clock.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(text: &str) -> PlayerKey {
        PlayerKey::new(text).expect("a test key")
    }

    #[test]
    fn a_name_finds_its_player_in_any_case() {
        let known = vec![
            (key("a1"), "Ana".to_string()),
            (key("b2"), "Bo".to_string()),
        ];
        assert_eq!(
            matching(known, "ana"),
            Found::One(key("a1")),
            "case is ignored"
        );
    }

    #[test]
    fn an_unknown_name_finds_nobody() {
        let known = vec![(key("a1"), "Ana".to_string())];
        assert_eq!(matching(known, "Cy"), Found::Nobody, "nobody by that name");
    }

    #[test]
    fn a_shared_name_finds_the_one_online() {
        let (ana, other) = (key("a1"), key("a2"));
        assert_eq!(
            among(Found::Several, Found::One(ana)),
            Found::One(ana),
            "the one online"
        );
        assert_eq!(
            among(Found::Several, Found::Nobody),
            Found::Several,
            "neither online"
        );
        assert_eq!(
            among(Found::One(other), Found::One(ana)),
            Found::One(other),
            "a name only one player used finds them, online or not"
        );
    }

    #[test]
    fn a_shared_name_is_not_guessed() {
        let known = vec![
            (key("a1"), "Ana".to_string()),
            (key("a2"), "ANA".to_string()),
        ];
        assert_eq!(
            matching(known, "Ana"),
            Found::Several,
            "two players share it"
        );
    }
}
