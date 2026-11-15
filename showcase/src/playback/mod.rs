//! A film playing: which step it is at, how far into it, and where that puts the camera. Plain
//! data stepped by the frame's time, so that the director only applies what comes out.

mod stops;

use std::collections::HashSet;

use showcase_api::{Pos, Step, Timeline};

use crate::rig::{Pose, Rig, Subjects};
use crate::screens::Screens;
use crate::weather::{Layer, WeatherFade};

/// Most blocks between two places the world is loaded at before a film, well inside what the
/// world loads round the camera.
const STOP_SPACING: f64 = 64.0;
/// Points looked at along each shot to place its stops.
const SAMPLES: usize = 100;

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
    pub at: Pos,
    pub until: f32,
}

pub struct Playback {
    pub film: String,
    pub origin: Pos,
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
    /// Blocks walked by the swaying shots so far, for the next to sway on in step.
    walked: f64,
    pub titles: Vec<Title>,
    pub menu: Option<ShownMenu>,
    weather: WeatherFade,
    pub screens: Screens,
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
            walked: 0.0,
            titles: Vec::new(),
            menu: None,
            weather: WeatherFade::default(),
            screens: Screens::default(),
        }
    }

    pub fn pose(&self) -> Pose {
        self.pose
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// The film's weather now; none once the film is over.
    pub fn weather(&self) -> Vec<Layer> {
        if self.done() {
            return Vec::new();
        }
        self.weather.at(self.elapsed)
    }

    /// The step of the film that plays now, counted from 0.
    pub fn step(&self) -> usize {
        self.index
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
            if let Some(rig) = self.rig.take() {
                self.walked = rig.walked();
            }
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
                    at: self.origin + *at,
                    until: self.elapsed - clock + secs,
                });
                Some(0.0)
            }
            Step::Weather(weather) => {
                let now = self.elapsed - clock;
                self.weather.set(weather.clone(), now);
                Some(0.0)
            }
            Step::Screen {
                shader,
                strength,
                secs,
            } => {
                let now = self.elapsed - clock;
                self.screens.set(shader, *strength, *secs, now);
                Some(0.0)
            }
            Step::Wait(secs) => (clock >= *secs).then_some(*secs),
            Step::WaitFor { name, max } => {
                if signals.remove(name) {
                    // The signal came at some point in this frame: the next step starts afresh.
                    Some(clock)
                } else {
                    (clock >= *max).then_some(*max)
                }
            }
            Step::Shot(_) => self.advance_shot(subjects),
        }
    }

    /// Moves the camera along the current shot for the clock, setting up its rig on its first
    /// frame; `Some(its length)` once the clock is past its end.
    fn advance_shot(&mut self, subjects: Subjects) -> Option<f32> {
        let Some(Step::Shot(shot)) = self.steps.get(self.index) else {
            return None;
        };
        let rig = self.rig.get_or_insert_with(|| {
            Rig::new(shot.clone(), self.origin, self.own_fov).after_walking(self.walked)
        });
        let secs = rig.secs();
        self.pose = rig.pose(self.clock.min(secs), subjects, self.start);
        (self.clock >= secs).then_some(secs)
    }
}

#[cfg(test)]
mod tests {
    use showcase_api::{Film, Weather, dolly, hold};
    use weft_sdk::Vec3;

    use super::*;
    use crate::weather::Part;

    const START: Pose = Pose {
        eye: Vec3::ZERO,
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
        fov: 70.0,
    };

    fn play(film: Film) -> Playback {
        play_at(Pos::default(), film)
    }

    fn play_at(origin: Pos, film: Film) -> Playback {
        Playback::new(
            Timeline {
                film: "mymod:tour".into(),
                origin,
                steps: film.steps,
            },
            START,
        )
    }

    fn none(_: &str) -> Option<Vec3> {
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
    fn a_signal_sent_before_its_wait_is_kept() {
        let mut p = play(Film::new().wait(1.0).wait_for("dug", 60.0).cue("after"));
        let mut signals = HashSet::from(["dug".to_string()]);
        assert!(
            p.advance(0.5, &mut signals, &none).is_empty(),
            "still in the first wait"
        );
        assert!(signals.contains("dug"), "kept for the wait_for");
        assert_eq!(
            p.advance(0.6, &mut signals, &none),
            vec!["after".to_string()]
        );
        assert!(signals.is_empty(), "used up there");
    }

    #[test]
    fn a_signal_ends_its_wait_with_no_time_over() {
        let mut p = play(Film::new().wait_for("dug", 60.0).wait(1.0).cue("after"));
        let mut signals = HashSet::new();
        p.advance(5.0, &mut signals, &none);
        signals.insert("dug".to_string());
        p.advance(0.5, &mut signals, &none);
        assert!(
            p.advance(0.9, &mut signals, &none).is_empty(),
            "the next wait started at the signal"
        );
        assert_eq!(
            p.advance(0.1, &mut signals, &none),
            vec!["after".to_string()]
        );
    }

    #[test]
    fn a_timed_out_wait_carries_its_extra_time_on() {
        let mut p = play(Film::new().wait_for("never", 2.0).wait(1.0).cue("after"));
        let mut signals = HashSet::new();
        assert!(
            p.advance(2.5, &mut signals, &none).is_empty(),
            "half a second into the next wait"
        );
        assert_eq!(
            p.advance(0.5, &mut signals, &none),
            vec!["after".to_string()]
        );
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
            .menu("mymod:press", Pos::new(2, 0, -1), 2.0)
            .wait(5.0);
        let mut p = play_at(Pos::new(10, 64, 10), film);
        let mut signals = HashSet::new();
        p.advance(1.5, &mut signals, &none);
        let open = p.menu.clone().expect("open after the first wait");
        assert_eq!((open.at, open.until), (Pos::new(12, 64, 9), 3.0));
        p.advance(1.6, &mut signals, &none);
        assert_eq!(p.menu, None, "closed at three seconds");
    }

    #[test]
    fn a_fade_out_holds_the_camera_until_the_picture_is_black() {
        let film = Film::new().fade_out(2.0).fade_in(1.0).cue("after");
        let mut p = play(film);
        let mut signals = HashSet::new();
        assert!(p.advance(1.5, &mut signals, &none).is_empty(), "fading");
        let black = |p: &Playback| p.screens.at(p.elapsed()).first().map_or(0.0, |s| s.1);
        assert!(black(&p) > 0.5 && black(&p) < 1.0, "on its way");
        assert_eq!(p.advance(0.5, &mut signals, &none), ["after"], "on at 2 s");
        assert_eq!(black(&p), 1.0, "black then");
        p.advance(1.0, &mut signals, &none);
        assert_eq!(black(&p), 0.0, "back a second later");
    }

    #[test]
    fn swaying_shots_sway_on_in_step_across_a_title() {
        let walk = |z: f64| dolly([0.0, 0.0, z], [0.0, 0.0, z - 10.0]).bob().secs(2.0);
        let film = Film::new()
            .shot(walk(0.0))
            .title("mymod.title", 3.0)
            .shot(walk(-10.0));
        let mut p = play(film);
        p.advance(2.05, &mut HashSet::new(), &none);
        assert_eq!(p.step(), 2, "into the second walk");
        assert!((p.walked - 10.0).abs() < 1e-6, "ten blocks on");
    }

    #[test]
    fn a_weather_fades_with_film_time_and_the_film_end_lifts_it() {
        let film = Film::new()
            .wait(1.0)
            .weather(Weather::dusk().fade(2.0))
            .wait(4.0)
            .weather(Weather::mist().fade(2.0))
            .wait(2.0);
        let mut p = play(film);
        let mut signals = HashSet::new();
        let coming = |p: &Playback| {
            (p.weather().into_iter())
                .find(|layer| layer.part == Part::Coming)
                .map(|layer| (layer.weather, layer.weight))
        };
        p.advance(2.0, &mut signals, &none);
        assert_eq!(
            coming(&p),
            Some((Weather::dusk().fade(2.0), 0.5)),
            "dusk half in"
        );
        p.advance(2.0, &mut signals, &none);
        assert_eq!(
            coming(&p),
            Some((Weather::dusk().fade(2.0), 1.0)),
            "in by 3 s"
        );
        p.advance(2.0, &mut signals, &none);
        let layers = p.weather();
        assert_eq!(layers.len(), 3, "dusk going as mist comes: {layers:?}");
        assert_eq!(coming(&p).map(|(_, weight)| weight), Some(0.5));
        p.advance(1.0, &mut signals, &none);
        assert!(p.done(), "over at 7 s");
        assert_eq!(p.weather(), [], "lifted at its end, mist half in or not");
    }
}
