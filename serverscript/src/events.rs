//! What happens in the game, handed to the scripts: an event to `on_<name>(e)` of every script
//! that has the function, a question to `before_<name>(q)`, which answers `"deny"`, `"allow"`
//! or nothing.
//!
//! The game takes handlers only while a mod registers, so the list here is fixed; a reload
//! changes what the scripts do with it.

use rhai::{Dynamic, Map};
use weft_sdk::abi::entry_kind;
use weft_sdk::{
    Ask, BlockChange, BlockPos, PLAYER_JOINED, PLAYER_RESPAWNED, Registry, Verdict, World, actions,
    name_of, warn,
};

use crate::{bindings, script};

/// What a script's question function says to refuse.
const DENY: &str = "deny";
/// What it says to let through what a step before refused.
const ALLOW: &str = "allow";

pub fn register(r: &mut Registry) {
    r.on_event(&PLAYER_JOINED, |batch| {
        for joined in batch.iter() {
            tell(batch.world(), "on_player_joined", || player(joined.player));
        }
    });
    r.on_event(&PLAYER_RESPAWNED, |batch| {
        for respawned in batch.iter() {
            tell(batch.world(), "on_player_respawned", || {
                player(respawned.player)
            });
        }
    });
    r.on_event(&actions::CHAT, |batch| {
        for message in batch.iter() {
            tell(batch.world(), "on_chat", || {
                said(message.player, message.text())
            });
        }
    });
    r.on_event(&actions::BREAK_BLOCK, |batch| {
        for change in batch.iter() {
            tell(batch.world(), "on_block_broken", || changed(change));
        }
    });
    r.on_event(&actions::PLACE_BLOCK, |batch| {
        for change in batch.iter() {
            tell(batch.world(), "on_block_placed", || changed(change));
        }
    });
    r.on_event(&actions::DIE, |batch| {
        for death in batch.iter() {
            tell(batch.world(), "on_death", || {
                died(death.entity, death.source)
            });
        }
    });
    r.before_event(&actions::CHAT, |ask| {
        question(ask, "before_chat", || said(ask.entity(), ask.text()))
    });
    r.before_event(&actions::BREAK_BLOCK, |ask| {
        question(ask, "before_block_break", || block(ask))
    });
    r.before_event(&actions::PLACE_BLOCK, |ask| {
        question(ask, "before_block_place", || block(ask))
    });
}

/// Calls `hook` in every script that has it, with what `argument` makes, and gives back what
/// they answered. Logs the scripts that failed.
fn call(world: World, hook: &str, argument: impl FnOnce() -> Map) -> Vec<Dynamic> {
    let Some(scripts) = script::current() else {
        return Vec::new();
    };
    if !scripts.has(hook) {
        return Vec::new();
    }
    let results = bindings::within(world, || scripts.notify(hook, &argument()));
    let answered = |result: Result<Dynamic, String>| result.map_err(|error| warn!("{error}")).ok();
    results.into_iter().filter_map(answered).collect()
}

fn tell(world: World, hook: &str, argument: impl FnOnce() -> Map) {
    call(world, hook, argument);
}

fn question(ask: &Ask, hook: &str, argument: impl FnOnce() -> Map) -> Verdict {
    verdict(&call(ask.world(), hook, argument))
}

/// What the scripts' answers come to: one `"deny"` refuses, else one `"allow"` lets through,
/// else it is as the steps before said.
fn verdict(answers: &[Dynamic]) -> Verdict {
    let said = |word: &str| {
        let is_word =
            |answer: &Dynamic| answer.clone().into_string().is_ok_and(|text| text == word);
        answers.iter().any(is_word)
    };
    if said(DENY) {
        Verdict::Deny
    } else if said(ALLOW) {
        Verdict::Allow
    } else {
        Verdict::Abstain
    }
}

fn fields<const N: usize>(fields: [(&str, Dynamic); N]) -> Map {
    fields
        .into_iter()
        .map(|(name, value)| (name.into(), value))
        .collect()
}

/// An entity's id as scripts hold it.
fn id(entity: u64) -> Dynamic {
    Dynamic::from(entity as i64)
}

fn player(entity: u64) -> Map {
    fields([("player", id(entity))])
}

fn said(entity: u64, text: &str) -> Map {
    fields([("player", id(entity)), ("text", text.into())])
}

fn died(entity: u64, killer: u64) -> Map {
    fields([("entity", id(entity)), ("killer", id(killer))])
}

/// A block and who acts on it: `player` is 0 when the world did it.
fn at(entity: u64, pos: BlockPos, state: u32) -> Map {
    fields([
        ("player", id(entity)),
        ("x", i64::from(pos.x).into()),
        ("y", i64::from(pos.y).into()),
        ("z", i64::from(pos.z).into()),
        ("block", name_of(entry_kind::BLOCK, state).into()),
    ])
}

fn changed(change: &BlockChange) -> Map {
    at(change.player, change.pos, change.state)
}

fn block(ask: &Ask) -> Map {
    at(ask.entity(), ask.pos(), ask.state())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn answers(words: &[&str]) -> Vec<Dynamic> {
        words
            .iter()
            .map(|word| Dynamic::from(word.to_string()))
            .collect()
    }

    #[test]
    fn one_deny_refuses_whatever_the_others_say() {
        assert_eq!(verdict(&answers(&["allow", "deny"])), Verdict::Deny);
    }

    #[test]
    fn an_allow_lets_through_when_nobody_denies() {
        assert_eq!(verdict(&answers(&["allow", "other"])), Verdict::Allow);
    }

    #[test]
    fn no_answer_leaves_it_as_it_was() {
        assert_eq!(
            verdict(&[Dynamic::UNIT, Dynamic::from(1_i64)]),
            Verdict::Abstain
        );
        assert_eq!(verdict(&[]), Verdict::Abstain);
    }

    #[test]
    fn an_argument_holds_its_fields_by_name() {
        let map = said(7, "hello");
        assert_eq!(map["player"].as_int(), Ok(7));
        assert_eq!(map["text"].clone().into_string().as_deref(), Ok("hello"));
        assert_eq!(died(3, 4)["killer"].as_int(), Ok(4));
        assert_eq!(player(9)["player"].as_int(), Ok(9));
    }
}
