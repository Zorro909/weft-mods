//! What the film waits for: the oil engine running (`RUNNING`). A camera's player is noted
//! when its film starts to wait, and is sent the signal once, or forgotten when the wait is
//! over.

use std::sync::{Mutex, MutexGuard, PoisonError};

use showcase_api::{Set, signal};
use weft_sdk::{Registry, World};

use super::engine_bay::CONTROLLER;
use super::site::SITE;
use crate::lib::TypedWorld;

pub const RUNNING: &str = "running";
/// Seconds the film waits for the engine at most.
pub const WAIT_SECS: f32 = 20.0;
/// The same in ticks, 20 a second.
const WAIT_TICKS: u64 = 20 * WAIT_SECS as u64;

/// A player whose film waits for the engine, and the game time its wait is over at.
struct Waiting {
    player: u64,
    until: u64,
}

static WAITING: Mutex<Vec<Waiting>> = Mutex::new(Vec::new());

fn waiting() -> MutexGuard<'static, Vec<Waiting>> {
    WAITING.lock().unwrap_or_else(PoisonError::into_inner)
}

pub fn register(r: &mut Registry) {
    r.world_tick(send_when_running);
}

/// Starts watching for the film of `set`'s player, which now waits.
pub fn watch(set: &Set) {
    let player = set.player();
    let until = set.world().game_time() + WAIT_TICKS;
    let mut waiting = waiting();
    waiting.retain(|wait| wait.player != player);
    waiting.push(Waiting { player, until });
}

fn send_when_running(world: &World) {
    let mut waiting = waiting();
    if waiting.is_empty() {
        return;
    }
    // A wait from a world left since would end at a time far from this world's.
    let now = world.game_time();
    waiting.retain(|wait| (now..=now + WAIT_TICKS).contains(&wait.until));
    if world.block_typed(SITE + CONTROLLER).prop("working") != Some("true") {
        return;
    }
    for wait in waiting.drain(..) {
        signal(world, wait.player, RUNNING);
    }
}
