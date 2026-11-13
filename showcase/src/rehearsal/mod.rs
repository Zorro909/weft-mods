//! A rehearsal: a film gone through as fast as the world loads, without recording it, to find
//! its faults before a take is spent on them. With `--showcase-rehearse <folder>` a film that
//! is asked for is rehearsed instead of played: the camera is put at every moment of it in
//! turn (`plan`), the film's cues fire as it passes them, pictures are taken along every shot
//! into the folder, at most five seconds of film apart, and `report.txt` there lists the faults:
//! where the camera is inside a block, turns too fast, or a title stays too short or too long.
//! Machines run only for as long as the rehearsal takes, so what they do is for the take.

mod plan;

use showcase_api::{Name, Pos, Timeline};
use weft_sdk::client::client_flags as cf;
use weft_sdk::{Client, Vec3, block_info, block_name, info, launch_option, warn};

use self::plan::{Fault, Moment, block_of, clipped, fast_turns, moments, title_faults};
use crate::playback::Title;
use crate::rig::Pose;
use crate::server::{REACHED, Reached};
use crate::view::{put, titles};

/// Frames the camera waits at a moment for the world around it before going on unchecked.
const LOAD_FRAMES: u32 = 600;
/// Frames the camera stays before a picture, at least, and at most while the world still
/// loads further out.
const STILL_FRAMES: [u32; 2] = [20, 300];
/// Frames a picture takes to be written after it is asked for.
const SHUTTER_FRAMES: u32 = 3;

pub struct Rehearsal {
    film: String,
    origin: Pos,
    /// Where the pictures and the report go.
    folder: String,
    moments: Vec<Moment>,
    /// The moment the camera is at, and for how many frames so far.
    at: usize,
    frames: u32,
    /// The frame of this moment its picture was asked for at.
    shutter: Option<u32>,
    /// Whether the camera was inside a block at the moment before.
    clipping: bool,
    /// Moments whose world never loaded, so nothing is known of blocks there.
    unchecked: u32,
    faults: Vec<Fault>,
}

/// The folder of `--showcase-rehearse`, if the game was started to rehearse.
pub fn folder() -> Option<String> {
    launch_option("showcase-rehearse")
}

impl Rehearsal {
    pub fn new(timeline: Timeline, from: Pose, folder: String) -> Rehearsal {
        let (film, origin) = (timeline.film.clone(), timeline.origin);
        let steps = timeline.steps.clone();
        let moments = moments(timeline, from);
        let mut faults = fast_turns(&moments);
        faults.extend(title_faults(&steps, &moments));
        info!("showcase: rehearsing {film}, {} moments", moments.len());
        Rehearsal {
            film,
            origin,
            folder,
            moments,
            at: 0,
            frames: 0,
            shutter: None,
            clipping: false,
            unchecked: 0,
            faults,
        }
    }

    /// One frame of the rehearsal; whether it is over, its report written.
    pub fn frame(&mut self, c: &Client) -> bool {
        let Some(moment) = self.moments.get(self.at).cloned() else {
            self.report();
            return true;
        };
        if self.frames == 0 {
            self.send_cues(c, &moment);
        }
        self.show(c, &moment);
        self.frames += 1;
        let loaded = c.block(block_of(moment.pose.eye)).is_some();
        if !loaded && self.frames < LOAD_FRAMES {
            return false;
        }
        if moment.still.is_some() && !self.photographed(c, &moment) {
            return false;
        }
        self.check_blocks(c, &moment, loaded);
        self.at += 1;
        self.frames = 0;
        self.shutter = None;
        false
    }

    /// How many faults the rehearsal found.
    pub fn faults(&self) -> usize {
        self.faults.len()
    }

    fn send_cues(&self, c: &Client, moment: &Moment) {
        for cue in &moment.cues {
            let reached = Reached {
                film: Name::from(self.film.as_str()),
                origin: self.origin,
                name: Name::from(cue.as_str()),
            };
            c.send(&REACHED, &reached);
        }
    }

    /// The camera at the moment, with its titles in full.
    fn show(&self, c: &Client, moment: &Moment) {
        put(c, moment.pose);
        let shown: Vec<Title> = (moment.titles.iter())
            .map(|key| Title {
                key: key.clone(),
                from: f32::MIN,
                until: f32::MAX,
            })
            .collect();
        titles(c, &shown, moment.at, moment.pose);
    }

    /// Takes the moment's picture once the view has settled; whether it is written.
    fn photographed(&mut self, c: &Client, moment: &Moment) -> bool {
        if let Some(asked) = self.shutter {
            return self.frames >= asked + SHUTTER_FRAMES;
        }
        let flags = c.state().flags;
        let drawn = flags & cf::LOADED != 0 && self.frames >= STILL_FRAMES[0];
        let settled = flags & cf::IDLE != 0 || self.frames >= STILL_FRAMES[1];
        if let (true, true, Some(name)) = (drawn, settled, &moment.still) {
            c.screenshot(&format!("{}/{name}", self.folder));
            self.shutter = Some(self.frames);
        }
        false
    }

    /// Notes where the camera comes inside a block, once for every stretch of it.
    fn check_blocks(&mut self, c: &Client, moment: &Moment, loaded: bool) {
        if !loaded {
            self.unchecked += 1;
            return;
        }
        let solid = |pos: Pos| solid_block(c, pos).is_some();
        let hit = clipped(moment.pose.eye, solid);
        if let (Some(pos), false) = (hit, self.clipping) {
            let block = solid_block(c, pos).unwrap_or("a block");
            let Vec3 { x, y, z } = moment.pose.eye;
            self.faults.push(Fault {
                at: moment.at,
                what: format!(
                    "the camera at {x:.1} {y:.1} {z:.1} is inside or against {block} at {} {} {}",
                    pos.x, pos.y, pos.z
                ),
            });
        }
        self.clipping = hit.is_some();
    }

    /// Writes the faults in the film's order to `report.txt` and the log.
    fn report(&mut self) {
        self.faults.sort_by(|a, b| a.at.total_cmp(&b.at));
        let mut lines: Vec<String> = (self.faults.iter())
            .map(|fault| format!("{:7.1} s  {}", fault.at, fault.what))
            .collect();
        if self.unchecked > 0 {
            lines.push(format!(
                "{} moments went unchecked for blocks: the world never loaded there",
                self.unchecked
            ));
        }
        lines.push(format!("{} faults in {}", self.faults.len(), self.film));
        for line in &lines {
            warn!("showcase: rehearsal: {line}");
        }
        let path = format!("{}/report.txt", self.folder);
        if let Err(error) = std::fs::write(&path, lines.join("\n") + "\n") {
            warn!("showcase: could not write {path}: {error}");
        }
    }
}

/// The name of the block at `pos` if it is solid and loaded.
fn solid_block(c: &Client, pos: Pos) -> Option<&'static str> {
    let state = c.block(pos)?;
    block_info(state)
        .is_some_and(|block| block.solid())
        .then(|| block_name(state))
}
