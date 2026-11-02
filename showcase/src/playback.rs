//! A film playing: which step it is at, how far into it, and where that puts the camera. Plain
//! data stepped by the frame's time, so that the director only applies what comes out.

use std::collections::HashSet;

use showcase_api::{Path, Step, Timeline};

use crate::rig::{Pose, Rig, Subjects};

/// A line of text on screen, from `from` to `until` seconds into the film.
pub struct Title {
    pub key: String,
    pub from: f32,
    pub until: f32,
}

pub struct Playback {
    pub film: String,
    pub origin: [i32; 3],
    steps: Vec<Step>,
    index: usize,
    /// Seconds into the current step.
    clock: f32,
    /// Seconds into the film.
    elapsed: f32,
    rig: Option<Rig>,
    pose: Pose,
    /// Where the camera was when the current step began.
    start: Pose,
    pub titles: Vec<Title>,
}

impl Playback {
    pub fn new(timeline: Timeline, from: Pose) -> Playback {
        Playback {
            film: timeline.film,
            origin: timeline.origin,
            steps: timeline.steps.steps,
            index: 0,
            clock: 0.0,
            elapsed: 0.0,
            rig: None,
            pose: from,
            start: from,
            titles: Vec::new(),
        }
    }

    pub fn pose(&self) -> Pose {
        self.pose
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn done(&self) -> bool {
        self.index >= self.steps.len()
    }

    /// Goes `dt` seconds on and returns the cues reached. A signal a `wait_for` was waiting for
    /// is taken out of `signals`.
    pub fn advance(
        &mut self,
        dt: f32,
        signals: &mut HashSet<String>,
        subjects: Subjects,
    ) -> Vec<String> {
        self.elapsed += dt;
        self.clock += dt;
        let mut cues = Vec::new();
        // Steps that take no time all happen in this frame; the bound only guards the loop.
        for _ in 0..=self.steps.len() {
            let Some(used) = self.run_step(signals, subjects, &mut cues) else {
                break;
            };
            self.clock = (self.clock - used).max(0.0);
            self.index += 1;
            self.rig = None;
            self.start = self.pose;
        }
        let now = self.elapsed;
        self.titles.retain(|t| t.until > now);
        cues
    }

    /// Runs the current step for the clock; `Some(seconds it took)` once it is over.
    fn run_step(
        &mut self,
        signals: &mut HashSet<String>,
        subjects: Subjects,
        cues: &mut Vec<String>,
    ) -> Option<f32> {
        let clock = self.clock;
        match self.steps.get(self.index)? {
            Step::Title { key, secs } => {
                let from = self.elapsed - clock;
                self.titles.push(Title {
                    key: key.clone(),
                    from,
                    until: from + secs,
                });
                Some(0.0)
            }
            Step::Cue(name) => {
                cues.push(name.clone());
                Some(0.0)
            }
            Step::Wait(secs) => (clock >= *secs).then_some(*secs),
            Step::WaitFor { name, max } => {
                let came = signals.remove(name);
                (came || clock >= *max).then_some(if came { clock } else { *max })
            }
            Step::Shot(shot) => {
                let rig = self
                    .rig
                    .get_or_insert_with(|| Rig::new(shot.clone(), self.origin));
                let secs = rig.secs();
                self.pose = rig.pose(clock.min(secs), subjects, self.start);
                (clock >= secs).then_some(secs)
            }
        }
    }

    /// Eye positions the film's shots pass, to load the world around before it starts: the
    /// start, middle and end of every shot that doesn't follow a subject.
    pub fn stops(&self) -> Vec<[f64; 3]> {
        let none = |_: &str| None;
        let mut out = Vec::new();
        for step in &self.steps {
            let Step::Shot(shot) = step else { continue };
            if matches!(shot.path, Path::Follow { .. }) {
                continue;
            }
            let rig = Rig::new(shot.clone().blend(0.0), self.origin);
            out.extend([0.0, 0.5, 1.0].map(|f| rig.pose(rig.secs() * f, &none, self.start).eye));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use showcase_api::{Film, hold};

    use super::*;

    const START: Pose = Pose {
        eye: [0.0; 3],
        yaw: 0.0,
        pitch: 0.0,
        fov: 0.0,
    };

    fn play(film: Film) -> Playback {
        Playback::new(
            Timeline {
                film: "mymod:tour".into(),
                origin: [0; 3],
                steps: film,
            },
            START,
        )
    }

    fn none(_: &str) -> Option<[f64; 3]> {
        None
    }

    #[test]
    fn time_left_over_from_a_shot_goes_into_the_next() {
        let mut p = play(
            Film::new()
                .shot(hold([1.0; 3]).secs(1.0))
                .wait(1.0)
                .cue("end"),
        );
        let mut signals = HashSet::new();
        assert!(
            p.advance(1.5, &mut signals, &none).is_empty(),
            "half a second into the wait"
        );
        assert_eq!(
            p.advance(0.5, &mut signals, &none),
            vec!["end".to_string()],
            "then the cue"
        );
        assert!(p.done(), "and the film is over");
    }

    #[test]
    fn a_wait_for_holds_until_its_signal() {
        let mut p = play(Film::new().wait_for("dug", 60.0).cue("after"));
        let mut signals = HashSet::new();
        assert!(
            p.advance(5.0, &mut signals, &none).is_empty(),
            "still waiting"
        );
        signals.insert("dug".to_string());
        assert_eq!(
            p.advance(0.1, &mut signals, &none),
            vec!["after".to_string()]
        );
        assert!(signals.is_empty(), "the signal is used up");
    }

    #[test]
    fn a_wait_for_gives_up_after_its_longest_wait() {
        let mut p = play(Film::new().wait_for("never", 2.0));
        p.advance(2.5, &mut HashSet::new(), &none);
        assert!(p.done());
    }

    #[test]
    fn titles_last_their_time_from_where_they_start() {
        let mut p = play(Film::new().wait(1.0).title("mymod.title", 2.0).wait(5.0));
        let mut signals = HashSet::new();
        p.advance(1.5, &mut signals, &none);
        assert_eq!(p.titles.len(), 1, "shown");
        assert_eq!(p.titles[0].from, 1.0, "from the end of the first wait");
        p.advance(1.6, &mut signals, &none);
        assert!(p.titles.is_empty(), "gone at three seconds");
    }
}
