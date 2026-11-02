//! `--showcase <mod:film>`: makes a fresh world, plays the film once and quits, for recording.
//! `--showcase-dim` picks the dimension (`base:flatlands` by default, `none` to stay where the
//! world starts), `--showcase-seed` the world's seed.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use weft_sdk::client::client_flags as cf;
use weft_sdk::*;

use crate::director;

/// How long the film's mod has to answer before the run gives up.
const ANSWER_TIME: Duration = Duration::from_secs(30);
/// Frames to stand in the new dimension before the film is asked for.
const SETTLE_FRAMES: u32 = 60;

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
    *LAUNCH.lock().unwrap_or_else(|e| e.into_inner()) = Some(Launch {
        film,
        stage: Stage::Start,
    });
    r.client_frame(|c| {
        let mut guard = LAUNCH.lock().unwrap_or_else(|e| e.into_inner());
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
            let seed = launch_option("showcase-seed")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            c.new_world(&format!("showcase-{}", film.replace(':', "-")), seed);
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
        Stage::Settling(frames) if frames >= SETTLE_FRAMES && c.state().flags & cf::LOADED != 0 => {
            c.run_command(&format!("/showcase play {film}"));
            Stage::Watching {
                since: Instant::now(),
            }
        }
        Stage::Settling(frames) => Stage::Settling(frames + 1),
        Stage::Watching { since } => watch(c, film, since),
        stage => stage,
    }
}

/// Commands that make the world the same on every run.
fn setup() -> Vec<String> {
    let mut lines = vec!["/rule core:mob_spawning 0".to_string()];
    let dim = launch_option("showcase-dim").unwrap_or_else(|| "base:flatlands".into());
    if dim != "none" {
        lines.push(format!("/dim {dim}"));
    }
    lines
}

fn watch(c: &Client, film: &str, since: Instant) -> Stage {
    if director::finished() > 0 {
        info!("showcase: {film} is over");
        c.quit(0);
        return Stage::Done;
    }
    if !director::busy() && since.elapsed() > ANSWER_TIME {
        error!("showcase: no film {film} came; is its mod loaded, built with its films?");
        c.quit(1);
        return Stage::Done;
    }
    Stage::Watching { since }
}
