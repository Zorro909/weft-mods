//! The hooks that keep the spawn and towns safe: the engine asks before a player breaks, places
//! or uses a block, before anyone is hurt and before a creature spawns, and Plotty answers from
//! `rules::allowed`. A denial runs in the last phase, the one the SDK keeps for claims, so no
//! other mod's yes gets past it. What changes blocks without asking (pistons, flowing liquids, a
//! mod's `fill` or `set_block`) is not stopped: `ROADMAP.md`.

use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard};

use weft_sdk::{Ask, Output, Registry, Verdict, World, abi, actions, permission, phase};
use weftkit::{BlockState, PlayerKey, Pos};

use crate::plot::post;
use crate::rules::{self, Action, Zone};
use crate::{admin::spawn, city, dims, market, settings};

/// Moderators who switched protection off for themselves, until they switch it back or the
/// server stops.
static BYPASSING: Mutex<Option<HashSet<PlayerKey>>> = Mutex::new(None);

pub fn register(r: &mut Registry) {
    r.before_event(&actions::BREAK_BLOCK, |ask| by_player(ask, Action::Change))
        .phase(&phase::FINAL);
    r.before_event(&actions::PLACE_BLOCK, |ask| by_player(ask, Action::Change))
        .phase(&phase::FINAL);
    r.before_event(&actions::USE_BLOCK, |ask| by_player(ask, Action::Use))
        .phase(&phase::FINAL);
    r.before_event(&actions::HURT, hurt).phase(&phase::FINAL);
    r.before_event(&actions::SPAWN, spawned)
        .phase(&phase::FINAL);
}

/// Switches protection off or back on for a moderator; whether it is off now.
pub fn toggle_bypass(key: PlayerKey) -> bool {
    let mut bypassing = lock();
    let set = bypassing.get_or_insert_with(HashSet::new);
    if set.remove(&key) {
        return false;
    }
    set.insert(key);
    true
}

/// A player breaking, placing or using the block the question names.
fn by_player(ask: &Ask, action: Action) -> Verdict {
    let world = ask.world();
    let player = ask.entity();
    let Some(key) = PlayerKey::of(&world, player) else {
        return Verdict::Abstain;
    };
    if action == Action::Use && for_everyone(ask.state()) {
        return Verdict::Abstain;
    }
    let action = on_stalls(action, ask.state());
    let pos = Pos::from(ask.pos());
    if action == Action::BuildStall {
        let owner = market::owner_in(dims::key_of(&world), pos);
        if !market::may_break(owner.as_ref(), &key, bypassing(&world, player, &key)) {
            world.hint("plotty.refused.not_your_stall", &[]).to(player);
            return Verdict::Deny;
        }
    }
    let zone = zone_at(&world, pos, Some(&key));
    if decide(&world, zone, action, player, &key) {
        return Verdict::Abstain;
    }
    let said = match action {
        Action::Change => Some(refusal(zone)),
        Action::BuildStall => Some("plotty.refused.stall"),
        _ => None,
    };
    if let Some(key) = said {
        world.hint(key, &[]).to(player);
    }
    Verdict::Deny
}

/// `action` as it is on the block `state`: building or trading where it is part of a stall.
fn on_stalls(action: Action, state: u32) -> Action {
    match (action, market::part_of(state).is_some()) {
        (Action::Change, true) => Action::BuildStall,
        (Action::Use, true) => Action::UseStall,
        (action, _) => action,
    }
}

/// Whether the block `state` is one of Plotty's that everyone may use: posts and stones say
/// what they stand for.
fn for_everyone(state: u32) -> bool {
    let block = BlockState::from(state);
    block.is(&post::PLOT_POST) || block.is(&post::CITY_STONE)
}

/// One entity hurting another: players fighting, or a player hurting a creature.
fn hurt(ask: &Ask) -> Verdict {
    let world = ask.world();
    let query = ask.query();
    let Some(attacker) = PlayerKey::of(&world, query.other) else {
        return Verdict::Abstain;
    };
    let Some(victim) = world.entity(query.entity) else {
        return Verdict::Abstain;
    };
    let action = if PlayerKey::of(&world, query.entity).is_some() {
        Action::PlayerFight
    } else {
        Action::HurtCreature
    };
    let zone = zone_at(&world, Pos::from(victim.pos.block()), Some(&attacker));
    verdict(decide(&world, zone, action, query.other, &attacker))
}

/// A creature appearing by natural spawning; mods, commands and eggs are left alone.
fn spawned(ask: &Ask) -> Verdict {
    if ask.query().cause != abi::spawn_cause::NATURAL {
        return Verdict::Abstain;
    }
    let world = ask.world();
    let zone = zone_at(&world, Pos::from(ask.pos()), None);
    verdict(rules::allowed(
        zone,
        Action::NaturalSpawn,
        false,
        settings::get().wilderness,
    ))
}

/// The rules for `player` (`key`), with their bypass where they are still a moderator.
fn decide(world: &World, zone: Zone, action: Action, player: u64, key: &PlayerKey) -> bool {
    rules::allowed(
        zone,
        action,
        bypassing(world, player, key),
        settings::get().wilderness,
    )
}

/// Whether `player` (`key`) switched protection off, and is still a moderator.
fn bypassing(world: &World, player: u64, key: &PlayerKey) -> bool {
    world.permission(player) >= permission::MODERATOR
        && lock().as_ref().is_some_and(|set| set.contains(key))
}

/// What the block at `pos` is part of, for the player `viewer` where a player asks.
fn zone_at(world: &World, pos: Pos, viewer: Option<&PlayerKey>) -> Zone {
    let dim = dims::key_of(world);
    if let Some(zone) = city::zone(dim, pos, viewer) {
        return zone;
    }
    match spawn::get() {
        Some(spawn) if spawn.protects(dim, pos) => Zone::Spawn,
        _ => Zone::Wilderness,
    }
}

/// The line a refused player sees.
fn refusal(zone: Zone) -> &'static str {
    match zone {
        Zone::Spawn => "plotty.protected.spawn",
        Zone::Wilderness => "plotty.protected.wilderness",
        Zone::Public => "plotty.protected.public",
        Zone::FreePlot => "plotty.protected.free_plot",
        Zone::OwnPlot | Zone::OthersPlot => "plotty.protected.plot",
        Zone::OwnPlace => "plotty.protected.place",
    }
}

fn verdict(allowed: bool) -> Verdict {
    if allowed {
        Verdict::Abstain
    } else {
        Verdict::Deny
    }
}

fn lock() -> MutexGuard<'static, Option<HashSet<PlayerKey>>> {
    BYPASSING
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
