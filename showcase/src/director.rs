//! Client half: receives a film's timeline, loads the world along its shots, then flies the
//! camera through them frame by frame, showing its titles and reporting its cues.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Instant;

use showcase_api::{Name, SIGNAL, SUBJECT, TIMELINE, Timeline};
use weft_sdk::client::client_flags as cf;
use weft_sdk::*;

use crate::lock;
use crate::menus::Menus;
use crate::playback::Playback;
use crate::rig::Pose;
use crate::rig::math::{add, angles_along, sub};
use crate::server::{FINISHED, MISSING, REACHED, Reached, STARTED};
use crate::timeline::decode;
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
    subjects: HashMap<String, Subject>,
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
    Loading(Loading),
    Rolling(Playback),
}

/// A film waiting for the world along it to load: the camera visits each stop in turn.
struct Loading {
    playback: Playback,
    stops: Vec<[f64; 3]>,
    /// The stop the camera is at, and for how many frames so far.
    at: usize,
    frames: u32,
}

/// What a film's shots follow or look at.
enum Subject {
    Entity(u64),
    Place([f64; 3]),
}

static DIRECTOR: Mutex<Option<Director>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut Director) -> R) -> R {
    f(lock(&DIRECTOR).get_or_insert_with(Director::default))
}

pub fn register(r: &mut Registry) {
    r.client_packet(&TIMELINE, |c, bytes| {
        let Some(timeline) = decode(&bytes) else {
            let len = bytes.len();
            error!("showcase: a film arrived that this version can't read ({len} bytes)");
            return;
        };
        c.send(&STARTED, &());
        play(c, timeline);
    });
    r.client_packet(&SIGNAL, |_, signal| {
        with(|d| d.signals.insert(text(&signal.name)));
    });
    r.client_packet(&SUBJECT, |_, subject| {
        let tracked = match subject.entity {
            0 => Subject::Place(subject.pos),
            entity => Subject::Entity(entity),
        };
        with(|d| d.subjects.insert(text(&subject.name), tracked));
    });
    r.client_packet(&MISSING, |_, missing| {
        let film = text(&missing.film);
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

fn text(name: &Name) -> String {
    <&str>::from(name).to_string()
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
    let loading = Loading {
        playback,
        stops,
        at: 0,
        frames: 0,
    };
    with(|d| d.phase = Phase::Loading(loading));
}

fn frame(c: &Client, d: &mut Director) {
    match std::mem::take(&mut d.phase) {
        Phase::Idle => {}
        Phase::Loading(loading) => d.phase = load(c, loading),
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
fn load(c: &Client, mut loading: Loading) -> Phase {
    let Some(&stop) = loading.stops.get(loading.at) else {
        mark("start", &loading.playback.film);
        return Phase::Rolling(loading.playback);
    };
    let origin = block_center(loading.playback.origin.into());
    let (yaw, pitch) = angles_along(sub(origin, stop)).unwrap_or_default();
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
    if loading.frames >= STOP_FRAMES && c.state().flags & cf::LOADED != 0 {
        loading.at += 1;
        loading.frames = 0;
    } else {
        loading.frames += 1;
    }
    Phase::Loading(loading)
}

fn roll(c: &Client, d: &mut Director, playback: &mut Playback) {
    let dt = c.state().dt.min(MAX_DT);
    let subjects = &d.subjects;
    let find = |name: &str| locate(c, subjects.get(name)?);
    let cues = playback.advance(dt, &mut d.signals, &find);
    for cue in cues {
        let reached = Reached {
            film: Name::from(playback.film.as_str()),
            origin: playback.origin,
            name: Name::from(cue.as_str()),
        };
        c.send(&REACHED, &reached);
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
fn locate(c: &Client, subject: &Subject) -> Option<[f64; 3]> {
    match *subject {
        Subject::Place(pos) => Some(pos),
        Subject::Entity(entity) => {
            let e = c.entity(entity)?;
            Some(add(e.pos, [0.0, e.size[1] as f64 / 2.0, 0.0]))
        }
    }
}

/// Says on screen for a while that the film asked for never came.
fn notice(c: &Client, d: &Director) {
    if let Some((film, since)) = &d.missing
        && since.elapsed().as_secs_f32() < NOTICE_SECS
    {
        missing_notice(c, film);
    }
}
