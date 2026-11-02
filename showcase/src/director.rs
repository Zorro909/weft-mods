//! Client half: receives a film's timeline, loads the world along its shots, then flies the
//! camera through them frame by frame, showing its titles and reporting its cues.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

use showcase_api::{SIGNAL, SUBJECT, TIMELINE, decode, name, name_text};
use weft_sdk::abi::{PlayerControl, control_mask as cm};
use weft_sdk::client::client_flags as cf;
use weft_sdk::*;

use crate::playback::Playback;
use crate::rig::Pose;
use crate::rig::ease::{add, angles_along, forward, sub};
use crate::server::{FINISHED, REACHED, Reached};

/// Frames the camera stays at each stop before the film, however soon the world says it is loaded.
const STOP_FRAMES: u32 = 20;
/// Longest frame the film steps by: a hitch skips ahead instead of jumping.
const MAX_DT: f32 = 0.1;

#[derive(Default)]
struct Director {
    phase: Phase,
    signals: HashSet<String>,
    /// Subjects by name: an entity, or a position where the entity is 0.
    subjects: HashMap<String, (u64, [f64; 3])>,
    /// The player's own field of view while a film changes it.
    own_fov: Option<f32>,
    /// Films played to the end since the game started.
    finished: u32,
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
    r.client_packet(&TIMELINE, |c, bytes| start(c, &bytes));
    r.client_packet(&SIGNAL, |_, s| {
        with(|d| d.signals.insert(name_text(&s.name).to_string()));
    });
    r.client_packet(&SUBJECT, |_, s| {
        with(|d| {
            d.subjects
                .insert(name_text(&s.name).to_string(), (s.entity, s.pos))
        });
    });
    r.client_frame(|c| with(|d| frame(c, d)));
}

/// Whether a film is loading or playing.
pub fn busy() -> bool {
    with(|d| !matches!(d.phase, Phase::Idle))
}

/// How many films have played to the end.
pub fn finished() -> u32 {
    with(|d| d.finished)
}

fn start(c: &Client, bytes: &[u8]) {
    let Some(timeline) = decode(bytes) else {
        error!(
            "showcase: a film arrived that this version can't read ({} bytes)",
            bytes.len()
        );
        return;
    };
    info!("showcase: loading the world along {}", timeline.film);
    let playback = Playback::new(timeline, camera(c));
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
    let center = playback.origin.map(|v| v as f64 + 0.5);
    let (yaw, pitch) = angles_along(sub(center, stop)).unwrap_or((0.0, 0.0));
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
    set_fov(d, pose.fov);
    titles(c, playback, pose);
}

fn finish(c: &Client, d: &mut Director, playback: &Playback) -> Phase {
    mark("stop", &playback.film);
    c.send(&FINISHED, &());
    set_fov(d, 0.0);
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

/// The camera now, from the local player.
fn camera(c: &Client) -> Pose {
    let s = c.state();
    let eye = add(s.pos, [0.0, eye_height(c), 0.0]);
    Pose {
        eye,
        yaw: (s.yaw as f64).to_degrees(),
        pitch: (s.pitch as f64).to_degrees(),
        fov: 0.0,
    }
}

fn eye_height(c: &Client) -> f64 {
    c.entity(0).map_or(1.62, |e| e.size[2] as f64)
}

/// Moves the local player so that its eyes are the camera.
fn put(c: &Client, pose: Pose) {
    c.control(&PlayerControl {
        mask: cm::POS | cm::VEL | cm::LOOK | cm::FLYING,
        pos: sub(pose.eye, [0.0, eye_height(c), 0.0]),
        vel: [0.0; 3],
        yaw: pose.yaw.to_radians() as f32,
        pitch: pose.pitch.to_radians() as f32,
        flying: 1,
        ..PlayerControl::DEFAULT
    });
}

/// Sets the field of view a shot asks for (0: the player's own), only when it changes.
fn set_fov(d: &mut Director, fov: f32) {
    use weft_sdk::settings::FOV;
    if fov > 0.0 {
        d.own_fov.get_or_insert_with(|| FOV.get());
        if (FOV.get() - fov).abs() > 0.05 {
            FOV.set(fov);
        }
    } else if let Some(own) = d.own_fov.take() {
        FOV.set(own);
    }
}

/// The film's titles, faded in and out, low in the middle of the picture. They are drawn as
/// labels in the world in front of the camera, which stay visible while the HUD is hidden.
fn titles(c: &Client, playback: &Playback, pose: Pose) {
    const FADE: f32 = 0.6;
    let now = playback.elapsed();
    let ahead = forward(pose.yaw, pose.pitch - 9.0);
    let at = add(pose.eye, ahead.map(|v| v * 12.0));
    for title in &playback.titles {
        let shown = ((now - title.from) / FADE)
            .min((title.until - now) / FADE)
            .clamp(0.0, 1.0);
        let alpha = (shown * 255.0) as u32;
        c.draw_text(None, at, &title.key, &[], 0xFFFFFF00 | alpha, 2.5);
    }
}
