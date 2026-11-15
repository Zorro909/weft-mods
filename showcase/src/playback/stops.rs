//! Where a film takes the camera before it rolls: the places the world is loaded at along its
//! shots, and its opening view.

use showcase_api::{Path, Step};
use weft_sdk::Vec3;

use super::{Playback, SAMPLES, STOP_SPACING};
use crate::rig::math::length;
use crate::rig::{Pose, Rig};

impl Playback {
    /// Where the camera is as the film starts: the start of its first shot. `None` for a film
    /// that opens following a subject, or has no shots.
    pub fn opening(&self) -> Option<Pose> {
        let shot = self.steps.iter().find_map(|step| match step {
            Step::Shot(shot) => Some(shot),
            _ => None,
        })?;
        if matches!(shot.path, Path::Follow { .. }) {
            return None;
        }
        let rig = Rig::new(shot.clone().blend(0.0), self.origin, self.own_fov);
        Some(rig.pose(0.0, &|_: &str| None, self.start))
    }

    /// Eye positions the film's shots pass, to load the world around before it starts: the
    /// start and end of every shot that doesn't follow a subject, and points at most
    /// `STOP_SPACING` blocks apart between, so a long flight finds the land along it ready.
    /// The film's opening comes again last, so the world there is still loaded when it starts.
    pub fn stops(&self) -> Vec<Vec3> {
        let none = |_: &str| None;
        let mut out = Vec::new();
        for step in &self.steps {
            let Step::Shot(shot) = step else { continue };
            if matches!(shot.path, Path::Follow { .. }) {
                continue;
            }
            let rig = Rig::new(shot.clone().blend(0.0), self.origin, self.own_fov);
            let eye = |i: usize| {
                rig.pose(rig.secs() * i as f32 / SAMPLES as f32, &none, self.start)
                    .eye
            };
            let (mut last, mut before) = (eye(0), eye(0));
            out.push(last);
            for i in 1..=SAMPLES {
                let next = eye(i);
                if length(next - last) > STOP_SPACING {
                    out.push(before);
                    last = before;
                }
                before = next;
            }
            out.push(before);
        }
        out.extend(self.opening().map(|pose| pose.eye));
        out
    }
}

#[cfg(test)]
mod tests {
    use showcase_api::{Film, Pos, Timeline, dolly, follow};

    use super::*;

    fn play_at(origin: Pos, film: Film) -> Playback {
        let start = Pose {
            eye: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            fov: 70.0,
        };
        let timeline = Timeline {
            film: "mymod:tour".into(),
            origin,
            steps: film.steps,
        };
        Playback::new(timeline, start)
    }

    #[test]
    fn the_world_is_loaded_along_every_shot_but_a_follow() {
        let film = Film::new()
            .shot(dolly([0.0; 3], [10.0, 0.0, 0.0]))
            .shot(follow("item", [0.0, 1.0, 0.0]));
        let p = play_at(Pos::new(100, 64, 0), film);
        let stops = [100.0, 110.0, 100.0].map(|x| Vec3::new(x, 64.0, 0.0));
        assert_eq!(
            p.stops(),
            stops,
            "the dolly's start and end in the world, then its start again"
        );
    }

    #[test]
    fn the_film_opens_where_its_first_shot_starts() {
        let film = Film::new()
            .title("mymod.film.tour", 4.0)
            .shot(dolly([0.0; 3], [10.0, 0.0, 0.0]).look_at([0.0, 0.0, -10.0]));
        let opening = play_at(Pos::new(100, 64, 0), film)
            .opening()
            .expect("a shot");
        assert_eq!(opening.eye, Vec3::new(100.0, 64.0, 0.0), "where it starts");
        assert!(opening.yaw.abs() < 1e-9, "looking north: {}", opening.yaw);
    }

    #[test]
    fn a_long_flight_is_loaded_all_along() {
        let film = Film::new().shot(dolly([0.0; 3], [1000.0, 0.0, 0.0]));
        let mut stops = play_at(Pos::default(), film).stops();
        stops.pop();
        let mut gaps = stops.windows(2).map(|w| w[1].x - w[0].x);
        assert!(gaps.all(|gap| gap <= STOP_SPACING), "{stops:?}");
        assert_eq!(
            stops.last(),
            Some(&Vec3::new(1000.0, 0.0, 0.0)),
            "to its end"
        );
    }
}
