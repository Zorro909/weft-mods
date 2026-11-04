//! `/camera`, for animators: camera paths made in the game and played on this client alone.
//! `/camera point` marks where the camera is and where it looks, `/camera play` flies through
//! the points looking the way each was marked, `/camera orbit` circles a point in front of
//! the camera, `/camera clear` forgets the points. Spectators fly through walls; anyone else
//! is stopped by them.

use std::sync::Mutex;

use showcase_api::{Film, Look, Pos, Timeline, flyby, orbit};
use weft_sdk::*;

use crate::director::play;
use crate::lock;
use crate::rig::Pose;
use crate::rig::math::{add, forward, sub};
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
        if cmd.has("point") {
            let count = {
                let mut points = lock(&POINTS);
                points.push(camera(&c));
                points.len()
            };
            cmd.reply("showcase.camera.marked", &[Arg::Int(count as i64)]);
        } else if cmd.has("play") {
            play_points(&c, cmd);
        } else if cmd.has("orbit") {
            let radius = cmd.float("radius").unwrap_or(ORBIT_RADIUS);
            let radius = radius.max(MIN_ORBIT_RADIUS);
            let secs = cmd.float("secs").unwrap_or(ORBIT_SECS);
            play(&c, local(orbit_ahead(camera(&c), radius, secs)));
        } else if cmd.has("clear") {
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
    let eyes: Vec<[f64; 3]> = points.iter().map(|p| p.eye).collect();
    let angles = points
        .iter()
        .map(|p| [p.yaw as f32, p.pitch as f32])
        .collect();
    let shot = flyby(&eyes).look(Look::Along(angles)).secs(secs as f32);
    play(c, local(Film::new().shot(shot)));
}

/// A full turn around the point `radius` blocks in front of `from`, starting where it is.
fn orbit_ahead(from: Pose, radius: f64, secs: f64) -> Film {
    let center = add(from.eye, forward(from.yaw, from.pitch).map(|v| v * radius));
    let [dx, dy, dz] = sub(from.eye, center);
    let start = dz.atan2(dx).to_degrees();
    let shot = orbit(center, dx.hypot(dz))
        .arc(start, start + 360.0)
        .height(dy, dy)
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
            eye: [10.0, 70.0, -4.0],
            yaw: 30.0,
            pitch: -20.0,
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
        let eye = [
            center[0] + a.cos() * radius[0],
            center[1] + height[0],
            center[2] + a.sin() * radius[0],
        ];
        assert!(
            (0..3).all(|i| (eye[i] - from.eye[i]).abs() < 1e-9),
            "{eye:?}"
        );
        assert!(
            (length(sub(from.eye, *center)) - 8.0).abs() < 1e-9,
            "the point 8 blocks ahead"
        );
    }
}
