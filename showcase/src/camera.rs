//! `/camera`, for animators: camera paths made in the game and played on this client alone.
//! `/camera point` marks where the camera is and where it looks, `/camera play` flies through
//! the points looking the way each was marked, `/camera orbit` circles a point in front of
//! the camera, `/camera clear` forgets the points. The camera flies through walls; the player,
//! moved along for the world to load around, is stopped by them unless a spectator.

use std::sync::Mutex;

use showcase_api::{Film, Look, Pos, Timeline, flyby, orbit};
use weft_sdk::*;

use crate::director::play;
use crate::lock;
use crate::rig::Pose;
use crate::rig::math::forward;
use crate::view::camera;

/// The points marked, in order.
static POINTS: Mutex<Vec<Pose>> = Mutex::new(Vec::new());
/// Seconds per point when `/camera play` isn't told how long.
const SECS_PER_POINT: f64 = 2.5;
/// Seconds of `/camera orbit` when it isn't told how long.
const ORBIT_SECS: f64 = 12.0;
/// Blocks ahead of the camera that `/camera orbit` circles when it isn't told how far.
const ORBIT_RADIUS: f64 = 8.0;
/// The nearest `/camera orbit` circles, in blocks: any closer and the camera spins on the spot.
const MIN_ORBIT_RADIUS: f64 = 0.5;

pub fn register(r: &mut Registry) {
    let usage = "point | play [secs:float] | orbit [radius:float] [secs:float] | clear";
    r.client_command("camera", usage, |cmd| {
        let c = cmd.client();
        if cmd.given("point") {
            let count = {
                let mut points = lock(&POINTS);
                points.push(camera(&c));
                points.len()
            };
            cmd.reply("showcase.camera.marked", &[Arg::Int(count as i64)]);
        } else if cmd.given("play") {
            play_points(&c, cmd);
        } else if cmd.given("orbit") {
            let radius = cmd.float("radius").unwrap_or(ORBIT_RADIUS);
            let radius = radius.max(MIN_ORBIT_RADIUS);
            let secs = cmd.float("secs").unwrap_or(ORBIT_SECS);
            play(&c, local(orbit_ahead(camera(&c), radius, secs)));
        } else if cmd.given("clear") {
            lock(&POINTS).clear();
            cmd.reply("showcase.camera.cleared", &[]);
        }
    });
}

fn play_points(c: &Client, cmd: &mut ClientCommand) {
    let points = lock(&POINTS).clone();
    if points.len() < 2 {
        cmd.reply("showcase.camera.too_few", &[]);
        return;
    }
    let secs = cmd
        .float("secs")
        .unwrap_or(SECS_PER_POINT * (points.len() - 1) as f64);
    let eyes: Vec<[f64; 3]> = points.iter().map(|p| p.eye.into()).collect();
    let angles = points
        .iter()
        .map(|p| [p.yaw as f32, p.pitch as f32])
        .collect();
    let shot = flyby(&eyes).look(Look::Along(angles)).secs(secs as f32);
    play(c, local(Film::new().shot(shot)));
}

/// A full turn around the point `radius` blocks in front of `from`, starting where it is.
fn orbit_ahead(from: Pose, radius: f64, secs: f64) -> Film {
    let center = from.eye + forward(from.yaw, from.pitch) * radius;
    let Vec3 { x, y, z } = from.eye - center;
    let start = z.atan2(x).to_degrees();
    let shot = orbit(center.into(), x.hypot(z))
        .arc(start, start + 360.0)
        .height(y, y)
        .secs(secs as f32);
    Film::new().shot(shot)
}

/// A film of this client's own, around the world's origin so that its positions are the world's.
fn local(film: Film) -> Timeline {
    Timeline {
        film: "showcase:camera".into(),
        origin: Pos::default(),
        steps: film.steps,
    }
}

#[cfg(test)]
mod tests {
    use showcase_api::{Path, Step};

    use super::*;
    use crate::rig::math::length;

    #[test]
    fn an_orbit_starts_where_the_camera_is() {
        let from = Pose {
            eye: Vec3::new(10.0, 70.0, -4.0),
            yaw: 30.0,
            pitch: -20.0,
            roll: 0.0,
            fov: 70.0,
        };
        let film = orbit_ahead(from, 8.0, 12.0);
        let Some(Step::Shot(shot)) = film.steps.first() else {
            panic!("one shot")
        };
        let Path::Orbit {
            center,
            radius,
            height,
            angle,
        } = &shot.path
        else {
            panic!("an orbit")
        };
        let a = angle[0].to_radians();
        let center = Vec3::from(*center);
        let eye = center.offset(a.cos() * radius[0], height[0], a.sin() * radius[0]);
        assert!(length(eye - from.eye) < 1e-9, "{eye:?}");
        assert!(
            (length(from.eye - center) - 8.0).abs() < 1e-9,
            "the point 8 blocks ahead"
        );
    }
}
