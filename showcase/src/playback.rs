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

/// A block's menu on screen until `until` seconds into the film.
#[derive(Clone, Debug, PartialEq)]
pub struct ShownMenu {
    pub menu: String,
    /// The block, in the world.
    pub at: [i32; 3],
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
    /// The player's field of view, for shots that keep it.
    own_fov: f32,
    pub titles: Vec<Title>,
    pub menu: Option<ShownMenu>,
}

impl Playback {
    pub fn new(timeline: Timeline, from: Pose) -> Playback {
        Playback {
            film: timeline.film,
            origin: timeline.origin,
            steps: timeline.steps,
            index: 0,
            clock: 0.0,
            elapsed: 0.0,
            rig: None,
            pose: from,
            start: from,
            own_fov: from.fov,
            titles: Vec::new(),
            menu: None,
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
        // Every step that ends within this frame, so steps that take no time all happen now.
        while let Some(used) = self.run_step(signals, subjects, &mut cues) {
            self.clock = (self.clock - used).max(0.0);
            self.index += 1;
            self.rig = None;
            self.start = self.pose;
        }
        let now = self.elapsed;
        self.titles.retain(|t| t.until > now);
        self.menu = self.menu.take().filter(|m| m.until > now);
        cues
    }

    /// Runs the current step for the clock; `Some(seconds it took)` once it is over, `None`
    /// while it goes on or after the last.
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
            Step::Menu { menu, at, secs } => {
                self.menu = Some(ShownMenu {
                    menu: menu.clone(),
                    at: std::array::from_fn(|i| self.origin[i] + at[i]),
                    until: self.elapsed - clock + secs,
                });
                Some(0.0)
            }
            Step::Wait(secs) => (clock >= *secs).then_some(*secs),
            Step::WaitFor { name, max } => {
                if signals.remove(name) {
                    Some(clock)
                } else {
                    (clock >= *max).then_some(*max)
                }
            }
            Step::Shot(shot) => {
                let rig = self
                    .rig
                    .get_or_insert_with(|| Rig::new(shot.clone(), self.origin, self.own_fov));
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
            let rig = Rig::new(shot.clone().blend(0.0), self.origin, self.own_fov);
            out.extend([0.0, 0.5, 1.0].map(|f| rig.pose(rig.secs() * f, &none, self.start).eye));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use showcase_api::{Film, dolly, follow, hold};

    use super::*;

    const START: Pose = Pose {
        eye: [0.0; 3],
        yaw: 0.0,
        pitch: 0.0,
        fov: 70.0,
    };

    fn play(film: Film) -> Playback {
        play_at([0; 3], film)
    }

    fn play_at(origin: [i32; 3], film: Film) -> Playback {
        Playback::new(
            Timeline {
                film: "mymod:tour".into(),
                origin,
                steps: film.steps,
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

    #[test]
    fn a_menu_opens_on_the_block_in_the_set_for_its_time() {
        let film = Film::new()
            .wait(1.0)
            .menu("mymod:press", [2, 0, -1], 2.0)
            .wait(5.0);
        let mut p = play_at([10, 64, 10], film);
        let mut signals = HashSet::new();
        p.advance(1.5, &mut signals, &none);
        let open = p.menu.clone().expect("open after the first wait");
        assert_eq!((open.at, open.until), ([12, 64, 9], 3.0));
        p.advance(1.6, &mut signals, &none);
        assert_eq!(p.menu, None, "closed at three seconds");
    }

    #[test]
    fn the_world_is_loaded_along_every_shot_but_a_follow() {
        let film = Film::new()
            .shot(dolly([0.0; 3], [10.0, 0.0, 0.0]))
            .shot(follow("item", [0.0, 1.0, 0.0]));
        let p = play_at([100, 64, 0], film);
        let stops = [[100.0, 64.0, 0.0], [105.0, 64.0, 0.0], [110.0, 64.0, 0.0]];
        assert_eq!(
            p.stops(),
            stops,
            "the dolly's start, middle and end in the world"
        );
    }
}
