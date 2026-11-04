//! `--showcase <mod:film>`: makes a fresh world, plays the film once and quits, for recording.
//! `--showcase gallery:<mod>` plays the gallery of a mod's blocks instead.
//! The world is a normal one from `SEED`, the film played where it spawns the player;
//! `--showcase-seed` picks another seed, `--showcase-dim` another dimension (`base:flatlands`).

use std::sync::Mutex;
use std::time::{Duration, Instant};

use weft_sdk::client::client_flags as cf;
use weft_sdk::*;

use crate::{director, lock};

/// How long the run waits when even the server never answers (the mod missing altogether).
const ANSWER_TIME: Duration = Duration::from_secs(30);
/// A seed whose spawn is a gentle meadow by a lake with a forest behind: room for a set, and
/// something to see around it.
const SEED: u64 = 42;
/// Frames to stand in the new world before the film is asked for.
const SETTLE_FRAMES: u32 = 60;
/// Frames after which the film starts even if far terrain is still generating.
const SETTLE_MAX: u32 = 60 * 30;

enum Stage {
    Start,
    Joining,
    /// Running the setup commands one by one; `waiting` is the one sent last.
    Setup {
        left: Vec<String>,
        waiting: Option<u32>,
    },
    Settling(u32),
    Watching {
        since: Instant,
    },
    Done,
}

struct Launch {
    film: String,
    stage: Stage,
}

static LAUNCH: Mutex<Option<Launch>> = Mutex::new(None);

pub fn register(r: &mut Registry) {
    let Some(film) = launch_option("showcase") else {
        return;
    };
    *lock(&LAUNCH) = Some(Launch {
        film,
        stage: Stage::Start,
    });
    r.client_frame(|c| {
        let mut guard = lock(&LAUNCH);
        if let Some(launch) = guard.as_mut() {
            launch.stage = step(
                c,
                &launch.film,
                std::mem::replace(&mut launch.stage, Stage::Start),
            );
        }
    });
}

fn step(c: &Client, film: &str, stage: Stage) -> Stage {
    match stage {
        Stage::Start => {
            c.world_create(&format!("showcase-{}", film.replace(':', "-")), seed());
            Stage::Joining
        }
        Stage::Joining if c.state().flags & cf::JOINED != 0 => Stage::Setup {
            left: setup(),
            waiting: None,
        },
        Stage::Setup { mut left, waiting } => {
            if waiting.is_some_and(|id| c.command_reply(id).is_none()) {
                return Stage::Setup { left, waiting };
            }
            if left.is_empty() {
                return Stage::Settling(0);
            }
            let line = left.remove(0);
            Stage::Setup {
                waiting: Some(c.run_command(&line)),
                left,
            }
        }
        Stage::Settling(frames) if settled(c, frames) => {
            c.run_command(&command(film));
            Stage::Watching {
                since: Instant::now(),
            }
        }
        Stage::Settling(frames) => Stage::Settling(frames + 1),
        Stage::Watching { since } => watch(c, film, since),
        stage => stage,
    }
}

/// The world's seed: `--showcase-seed`, or `SEED` without it.
fn seed() -> u64 {
    let Some(given) = launch_option("showcase-seed") else {
        return SEED;
    };
    given.parse().unwrap_or_else(|_| {
        warn!("showcase: --showcase-seed {given} is not a whole number from 0 up; using {SEED}");
        SEED
    })
}

/// Whether the world around is ready for the film: loaded, and done generating as far as the
/// player sees, or as done as it gets in `SETTLE_MAX` frames.
fn settled(c: &Client, frames: u32) -> bool {
    let flags = c.state().flags;
    let idle = flags & cf::IDLE != 0 || frames >= SETTLE_MAX;
    frames >= SETTLE_FRAMES && flags & cf::LOADED != 0 && idle
}

/// The command that starts `film`.
fn command(film: &str) -> String {
    match film.strip_prefix("gallery:") {
        Some(mod_id) => format!("/showcase gallery {mod_id}"),
        None => format!("/showcase play {film}"),
    }
}

/// Commands that make the world the same on every run.
fn setup() -> Vec<String> {
    let mut lines = vec!["/rule core:mob_spawning 0".to_string()];
    lines.extend(launch_option("showcase-dim").map(|dim| format!("/dim {dim}")));
    lines
}

fn watch(c: &Client, film: &str, since: Instant) -> Stage {
    if director::finished() > 0 {
        info!("showcase: {film} is over");
        c.quit(0);
        return Stage::Done;
    }
    // The server says so when no mod answers; the time is for a request that never got there.
    if director::missing() || (!director::busy() && since.elapsed() > ANSWER_TIME) {
        error!("showcase: {film} never came");
        c.quit(1);
        return Stage::Done;
    }
    Stage::Watching { since }
}
