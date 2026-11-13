//! What a rehearsal goes through: the film stepped ahead of time into moments, the faults its
//! camera path shows without the world (a turn too fast to follow, a title too short to
//! read), and whether the camera at a moment is inside a block. Plain data, so it is tested
//! here and the game only supplies the blocks.

use std::collections::HashSet;

use showcase_api::{Pos, Step, Timeline};
use weft_sdk::Vec3;

use crate::playback::Playback;
use crate::rig::Pose;
use crate::rig::math::short_turn;

/// Seconds of film between two moments.
pub const STEP: f32 = 0.1;
/// Degrees a second past which a turn is too fast to follow.
const TURN_LIMIT: f64 = 75.0;
/// Seconds a title stays, at least and at most, to be read without holding up the film.
const TITLE_SECS: [f32; 2] = [3.0, 7.0];
/// Blocks around the eye that must be free of solid blocks, or their faces cut into the view.
const CLEARANCE: f64 = 0.3;
/// Seconds of a shot between two pictures of it, at most.
const STILL_SECS: f32 = 5.0;
/// Moments in the longest film a rehearsal follows: an hour of it.
const MOST_MOMENTS: usize = 36_000;

/// The film at one instant.
#[derive(Clone, Debug, PartialEq)]
pub struct Moment {
    /// Seconds into the film, every `wait_for` counted at its longest.
    pub at: f32,
    pub pose: Pose,
    /// The step of the film that plays now.
    pub step: usize,
    /// The cues reached since the moment before.
    pub cues: Vec<String>,
    /// The lang keys of the titles on screen.
    pub titles: Vec<String>,
    /// The file name of the picture taken here.
    pub still: Option<String>,
}

/// Something wrong with the film, and the second it shows at.
#[derive(Clone, Debug, PartialEq)]
pub struct Fault {
    pub at: f32,
    pub what: String,
}

/// The film of `timeline` from start to end, `STEP` seconds apart, for a camera that was at
/// `from` before it.
pub fn moments(timeline: Timeline, from: Pose) -> Vec<Moment> {
    let shots: Vec<bool> = (timeline.steps.iter())
        .map(|step| matches!(step, Step::Shot(_)))
        .collect();
    let mut playback = Playback::new(timeline, from);
    let mut signals = HashSet::new();
    let mut moments = Vec::new();
    while !playback.done() && moments.len() < MOST_MOMENTS {
        let cues = playback.advance(STEP, &mut signals, &|_| None);
        moments.push(Moment {
            at: playback.elapsed(),
            pose: playback.pose(),
            step: playback.step(),
            cues,
            titles: playback.titles.iter().map(|t| t.key.clone()).collect(),
            still: None,
        });
    }
    name_stills(&mut moments, &shots);
    moments
}

/// Gives moments of every shot the name of their picture, `<shot>_<number>_<seconds>s.png`, so
/// that sorted by name they are in the film's order: its first and last moment (the last a
/// step before its end, the moment of which is the next step's first) and moments between
/// them at most `STILL_SECS` apart, three to a shot at least.
fn name_stills(moments: &mut [Moment], shots: &[bool]) {
    let mut shot = 0;
    let mut from = 0;
    while from < moments.len() {
        let step = moments[from].step;
        let len = moments[from..]
            .iter()
            .take_while(|m| m.step == step)
            .count();
        if shots.get(step).copied().unwrap_or(false) {
            shot += 1;
            let stretches = ((len as f32 * STEP / STILL_SECS).ceil() as usize).max(2);
            for number in 0..=stretches {
                let moment = &mut moments[from + (len - 1) * number / stretches];
                moment.still = Some(format!("{shot:03}_{number:02}_{:06.1}s.png", moment.at));
            }
        }
        from += len;
    }
}

/// The stretches of a shot in which the camera turns faster than `TURN_LIMIT`. A cut from one
/// shot to the next is no turn.
pub fn fast_turns(moments: &[Moment]) -> Vec<Fault> {
    let mut faults = Vec::new();
    // The stretch being followed: when it began, and the fastest turn in it.
    let mut stretch: Option<(f32, f64)> = None;
    for pair in moments.windows(2) {
        let rate = turn_rate(&pair[0], &pair[1]);
        match (stretch, rate > TURN_LIMIT) {
            (Some((since, fastest)), true) => stretch = Some((since, fastest.max(rate))),
            (None, true) => stretch = Some((pair[0].at, rate)),
            (Some(over), false) => {
                faults.push(turn_fault(over, pair[0].at));
                stretch = None;
            }
            (None, false) => {}
        }
    }
    if let (Some(over), Some(last)) = (stretch, moments.last()) {
        faults.push(turn_fault(over, last.at));
    }
    faults
}

/// Degrees a second the camera turns from `before` to `after`; 0 across two steps.
fn turn_rate(before: &Moment, after: &Moment) -> f64 {
    if before.step != after.step {
        return 0.0;
    }
    let yaw = short_turn(before.pose.yaw, after.pose.yaw).abs();
    let pitch = (after.pose.pitch - before.pose.pitch).abs();
    yaw.max(pitch) / (after.at - before.at).max(1e-3) as f64
}

fn turn_fault((since, fastest): (f32, f64), until: f32) -> Fault {
    Fault {
        at: since,
        what: format!(
            "the camera turns up to {fastest:.0} degrees a second until {until:.1} s, past \
             the {TURN_LIMIT:.0} an eye follows"
        ),
    }
}

/// The titles that stay shorter or longer than `TITLE_SECS`.
pub fn title_faults(steps: &[Step], moments: &[Moment]) -> Vec<Fault> {
    let mut faults = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let Step::Title { key, secs } = step else {
            continue;
        };
        if (TITLE_SECS[0]..=TITLE_SECS[1]).contains(secs) {
            continue;
        }
        // A title takes no time: the first moment of a later step is when it comes up.
        let at = moments
            .iter()
            .find(|m| m.step > index)
            .map_or(0.0, |m| m.at);
        faults.push(Fault {
            at,
            what: format!(
                "the title {key} stays {secs} s; {} to {} are read in comfort",
                TITLE_SECS[0], TITLE_SECS[1]
            ),
        });
    }
    faults
}

/// The block `at` is in.
pub fn block_of(at: Vec3) -> Pos {
    Pos::new(
        at.x.floor() as i32,
        at.y.floor() as i32,
        at.z.floor() as i32,
    )
}

/// The solid block nearest in the list of those within `CLEARANCE` of `eye`, if any: the camera
/// is inside it, or so close that its faces cut into the picture.
pub fn clipped(eye: Vec3, solid: impl Fn(Pos) -> bool) -> Option<Pos> {
    let reach = [-CLEARANCE, 0.0, CLEARANCE];
    let mut near = Vec::new();
    for dx in reach {
        for dy in reach {
            for dz in reach {
                near.push(block_of(eye.offset(dx, dy, dz)));
            }
        }
    }
    near.into_iter().find(|&pos| solid(pos))
}

#[cfg(test)]
mod tests {
    use showcase_api::{Film, Look, dolly, hold};

    use super::*;

    const START: Pose = Pose {
        eye: Vec3::ZERO,
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
        fov: 70.0,
    };

    fn rehearse(film: Film) -> Vec<Moment> {
        let timeline = Timeline {
            film: "mymod:tour".into(),
            origin: Pos::default(),
            steps: film.steps,
        };
        moments(timeline, START)
    }

    fn stills(moments: &[Moment]) -> Vec<&str> {
        moments.iter().filter_map(|m| m.still.as_deref()).collect()
    }

    #[test]
    fn a_film_is_stepped_to_its_end() {
        let moments = rehearse(Film::new().shot(hold([1.0; 3]).secs(1.0)).wait(1.0));
        assert_eq!(moments.len(), 20, "two seconds, ten moments each");
        let last = moments.last().expect("a film of two seconds");
        assert!((last.at - 2.0).abs() < 1e-3, "it ends at {}", last.at);
        assert_eq!(
            last.pose.eye,
            Vec3::new(1.0, 1.0, 1.0),
            "where the shot left it"
        );
    }

    #[test]
    fn every_shot_has_a_still_at_its_start_middle_and_end() {
        let film = Film::new()
            .shot(hold([1.0; 3]).secs(1.0))
            .wait(1.0)
            .shot(hold([2.0; 3]).secs(2.0));
        let moments = rehearse(film);
        let expected = [
            "001_00_0000.1s.png",
            "001_01_0000.5s.png",
            "001_02_0000.9s.png",
            "002_00_0002.0s.png",
            "002_01_0002.9s.png",
            "002_02_0003.9s.png",
        ];
        // The moment a shot ends at belongs to the step after it.
        assert_eq!(stills(&moments), expected, "none of the wait");
    }

    #[test]
    fn a_long_shot_has_a_still_every_five_seconds() {
        let moments = rehearse(Film::new().shot(hold([1.0; 3]).secs(12.0)));
        let expected = [
            "001_00_0000.1s.png",
            "001_01_0004.0s.png",
            "001_02_0007.9s.png",
            "001_03_0011.9s.png",
        ];
        assert_eq!(stills(&moments), expected);
    }

    #[test]
    fn a_cue_and_a_title_come_at_their_moment() {
        let film = Film::new()
            .wait(0.5)
            .cue("start")
            .title("mymod.film.tour", 4.0)
            .wait(0.5);
        let moments = rehearse(film);
        let cued: Vec<usize> = (0..moments.len())
            .filter(|&i| !moments[i].cues.is_empty())
            .collect();
        assert_eq!(cued, [4], "half a second in");
        assert!(moments[3].titles.is_empty(), "no title before it");
        assert_eq!(moments[4].titles, ["mymod.film.tour"]);
    }

    #[test]
    fn a_fast_turn_is_one_fault_for_its_stretch() {
        let spin = hold([0.0; 3])
            .look(Look::Along(vec![[0.0, 0.0], [170.0, 0.0]]))
            .secs(1.0);
        let slow = hold([0.0; 3])
            .look(Look::Along(vec![[170.0, 0.0], [150.0, 0.0]]))
            .secs(2.0);
        let faults = fast_turns(&rehearse(Film::new().shot(spin).shot(slow)));
        assert_eq!(faults.len(), 1, "{faults:?}");
        assert!(faults[0].at < 1.0, "in the first shot: {faults:?}");
    }

    #[test]
    fn a_cut_between_shots_is_no_turn() {
        let north = dolly([0.0; 3], [0.0, 0.0, -5.0]).secs(1.0);
        let south = dolly([0.0; 3], [0.0, 0.0, 5.0]).secs(1.0);
        let faults = fast_turns(&rehearse(Film::new().shot(north).shot(south)));
        assert_eq!(
            faults,
            [],
            "the camera looks the other way from one frame to the next"
        );
    }

    #[test]
    fn a_title_too_short_or_too_long_is_a_fault() {
        let film = Film::new()
            .title("mymod.film.fine", 4.0)
            .wait(1.0)
            .title("mymod.film.short", 1.0)
            .title("mymod.film.long", 9.0)
            .wait(1.0);
        let faults = title_faults(&film.steps, &rehearse(film.clone()));
        assert_eq!(faults.len(), 2, "{faults:?}");
        assert!(
            faults[0].what.contains("mymod.film.short stays 1 s"),
            "{faults:?}"
        );
        assert!(
            (faults[0].at - 1.0).abs() < 0.11,
            "after the first wait: {faults:?}"
        );
    }

    #[test]
    fn an_eye_near_a_solid_block_is_clipped() {
        let wall = |pos: Pos| pos.x >= 5;
        assert_eq!(clipped(Vec3::new(2.5, 1.0, 0.5), wall), None, "well clear");
        let touched = clipped(Vec3::new(4.8, 1.0, 0.5), wall);
        assert_eq!(
            touched.map(|pos| pos.x),
            Some(5),
            "within reach of the wall"
        );
        assert!(
            clipped(Vec3::new(5.5, 1.0, 0.5), wall).is_some(),
            "inside it"
        );
    }
}
