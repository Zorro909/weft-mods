//! Client half: receives a film's timeline, loads the world along its shots, then flies the
//! camera through them frame by frame, showing its titles and reporting its cues.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Instant;

use showcase_api::{SIGNAL, SUBJECT, TIMELINE, Timeline, name, name_text};
use weft_sdk::client::client_flags as cf;
use weft_sdk::*;

use crate::menus::Menus;
use crate::playback::Playback;
use crate::rig::Pose;
use crate::rig::math::{add, angles_along, sub};
use crate::server::{FINISHED, MISSING, REACHED, Reached, STARTED};
use crate::view::{camera, missing_notice, put, set_fov, titles};

/// Frames the camera stays at each stop before the film, however soon the world says it is loaded.
const STOP_FRAMES: u32 = 20;
/// Longest frame the film steps by: a hitch skips ahead instead of jumping.
const MAX_DT: f32 = 0.1;
/// Seconds a film that never came is reported on screen.
const NOTICE_SECS: f32 = 6.0;

#[derive(Default)]
struct Director {
    phase: Phase,
    signals: HashSet<String>,
    /// Subjects by name: an entity, or a position where the entity is 0.
    subjects: HashMap<String, (u64, [f64; 3])>,
    /// The player's own field of view while a film plays, given back at its end.
    own_fov: Option<f32>,
    /// Films played to the end since the game started.
    finished: u32,
    /// The last film asked for that never came, and when that was known.
    missing: Option<(String, Instant)>,
    menus: Menus,
}

#[derive(Default)]
enum Phase {
    #[default]
    Idle,
    /// Visiting `stops[at]` for `frames` frames so far.
    Loading {
        playback: Playback,
        stops: Vec<[f64; 3]>,
        at: usize,
        frames: u32,
    },
    Rolling(Playback),
}

static DIRECTOR: Mutex<Option<Director>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut Director) -> R) -> R {
    let mut guard = DIRECTOR.lock().unwrap_or_else(|e| e.into_inner());
    f(guard.get_or_insert_with(Director::default))
}

pub fn register(r: &mut Registry) {
    r.client_packet(&TIMELINE, |c, timeline| {
        c.send(&STARTED, &());
        play(c, timeline);
    });
    r.client_packet(&SIGNAL, |_, s| {
        with(|d| d.signals.insert(name_text(&s.name).to_string()));
    });
    r.client_packet(&SUBJECT, |_, s| {
        with(|d| {
            d.subjects
                .insert(name_text(&s.name).to_string(), (s.entity, s.pos))
        });
    });
    r.client_packet(&MISSING, |_, m| {
        let film = name_text(&m.film).to_string();
        error!("showcase: no mod sent the film {film}; is it loaded, built with its films?");
        with(|d| d.missing = Some((film, Instant::now())));
    });
    r.client_frame(|c| {
        with(|d| {
            frame(c, d);
            notice(c, d);
        })
    });
}

/// Whether a film is loading or playing.
pub fn busy() -> bool {
    with(|d| !matches!(d.phase, Phase::Idle))
}

/// How many films have played to the end.
pub fn finished() -> u32 {
    with(|d| d.finished)
}

/// Whether a film was asked for and never came.
pub fn missing() -> bool {
    with(|d| d.missing.is_some())
}

/// Plays `timeline` on this client, after loading the world along it.
pub fn play(c: &Client, timeline: Timeline) {
    info!("showcase: loading the world along {}", timeline.film);
    let mut from = camera(c);
    // A film started over another keeps the field of view the player had before the first.
    from.fov = with(|d| *d.own_fov.get_or_insert(from.fov));
    let playback = Playback::new(timeline, from);
    let stops = playback.stops();
    with(|d| {
        d.phase = Phase::Loading {
            playback,
            stops,
            at: 0,
            frames: 0,
        }
    });
}

fn frame(c: &Client, d: &mut Director) {
    match std::mem::take(&mut d.phase) {
        Phase::Idle => {}
        Phase::Loading {
            playback,
            stops,
            at,
            frames,
        } => d.phase = load(c, playback, stops, at, frames),
        Phase::Rolling(mut playback) => {
            roll(c, d, &mut playback);
            d.phase = if playback.done() {
                finish(c, d, &playback)
            } else {
                Phase::Rolling(playback)
            };
        }
    }
}

/// One frame at a stop: on to the next once the world there is loaded, rolling after the last.
fn load(c: &Client, playback: Playback, stops: Vec<[f64; 3]>, at: usize, frames: u32) -> Phase {
    let Some(&stop) = stops.get(at) else {
        mark("start", &playback.film);
        return Phase::Rolling(playback);
    };
    let (yaw, pitch) = angles_along(sub(block_center(playback.origin), stop)).unwrap_or_default();
    // The field of view is left alone until the film rolls.
    put(
        c,
        Pose {
            eye: stop,
            yaw,
            pitch,
            fov: 0.0,
        },
    );
    let ready = frames >= STOP_FRAMES && c.state().flags & cf::LOADED != 0;
    let (at, frames) = if ready { (at + 1, 0) } else { (at, frames + 1) };
    Phase::Loading {
        playback,
        stops,
        at,
        frames,
    }
}

fn roll(c: &Client, d: &mut Director, playback: &mut Playback) {
    let dt = c.state().dt.min(MAX_DT);
    let subjects = &d.subjects;
    let find = |n: &str| subject(c, subjects.get(n)?);
    let cues = playback.advance(dt, &mut d.signals, &find);
    for cue in cues {
        c.send(
            &REACHED,
            &Reached {
                film: name(&playback.film),
                origin: playback.origin,
                name: name(&cue),
            },
        );
    }
    let pose = playback.pose();
    put(c, pose);
    set_fov(pose.fov);
    titles(c, &playback.titles, playback.elapsed(), pose);
    d.menus.show(c, playback.menu.as_ref());
}

fn finish(c: &Client, d: &mut Director, playback: &Playback) -> Phase {
    mark("stop", &playback.film);
    d.menus.show(c, None);
    c.send(&FINISHED, &());
    if let Some(own) = d.own_fov.take() {
        set_fov(own);
    }
    d.signals.clear();
    d.subjects.clear();
    d.finished += 1;
    Phase::Idle
}

/// A line in the log for `tools/showcase.sh` to cut the recording by: the wall clock in
/// seconds since 1970 when the film starts or stops.
fn mark(what: &str, film: &str) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    info!("REC {what} {film} {:.3}", now.as_secs_f64());
}

/// Where a subject is now: the middle of its entity, or its place.
fn subject(c: &Client, &(entity, pos): &(u64, [f64; 3])) -> Option<[f64; 3]> {
    if entity == 0 {
        return Some(pos);
    }
    let e = c.entity(entity)?;
    Some(add(e.pos, [0.0, e.size[1] as f64 / 2.0, 0.0]))
}

/// Says on screen for a while that the film asked for never came.
fn notice(c: &Client, d: &Director) {
    if let Some((film, since)) = &d.missing
        && since.elapsed().as_secs_f32() < NOTICE_SECS
    {
        missing_notice(c, film);
    }
}
