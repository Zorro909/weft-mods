//! The camera rig: where a shot puts the eye and where it looks, at any moment of it.

mod curve;
pub mod math;
mod turn;

use showcase_api::{Ease, Look, Path, Pos, Shot};

use self::curve::Curve;
use self::math::{add, angles_along, eased, lerp, lerp_angle, lerp3, sub};

/// A camera: eye position, and yaw, pitch, roll and field of view in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub eye: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub fov: f32,
}

/// Where a subject of the film is now, in the world; `None` while it is unknown.
pub type Subjects<'a> = &'a dyn Fn(&str) -> Option<[f64; 3]>;

/// One shot made ready to sample.
pub struct Rig {
    shot: Shot,
    origin: [f64; 3],
    curve: Option<Curve>,
    /// The player's field of view, which a shot's 0 stands for.
    own_fov: f32,
}

impl Rig {
    pub fn new(shot: Shot, origin: Pos, own_fov: f32) -> Rig {
        let origin: [f64; 3] = origin.into();
        let curve = match &shot.path {
            Path::Spline(points) => Some(Curve::new(
                &points.iter().map(|p| add(origin, *p)).collect::<Vec<_>>(),
            )),
            _ => None,
        };
        Rig {
            shot,
            origin,
            curve,
            own_fov,
        }
    }

    pub fn secs(&self) -> f32 {
        self.shot.secs
    }

    /// The camera `clock` seconds into the shot. `before` is where the camera was when it
    /// began, which a blend starts from and a shot without a direction keeps looking along.
    pub fn pose(&self, clock: f32, subjects: Subjects, before: Pose) -> Pose {
        let u = eased(self.shot.ease, (clock / self.shot.secs.max(1e-3)) as f64);
        let eye = self.eye(u, subjects).unwrap_or(before.eye);
        let (yaw, pitch) = self
            .aim(u, eye, subjects)
            .unwrap_or((before.yaw, before.pitch));
        let fov = self
            .shot
            .fov
            .map(|v| if v == 0.0 { self.own_fov } else { v });
        let fov = lerp(fov[0] as f64, fov[1] as f64, u) as f32;
        let [from, to] = self.shot.roll;
        let pose = Pose {
            eye,
            yaw,
            pitch,
            roll: lerp(from as f64, to as f64, u),
            fov,
        };
        if clock >= self.shot.blend {
            return pose;
        }
        blend(
            before,
            pose,
            eased(Ease::Smooth, (clock / self.shot.blend) as f64),
        )
    }

    fn eye(&self, u: f64, subjects: Subjects) -> Option<[f64; 3]> {
        let at = |p: &[f64; 3]| add(self.origin, *p);
        Some(match &self.shot.path {
            Path::Still(p) => at(p),
            Path::Line(a, b) => lerp3(at(a), at(b), u),
            Path::Spline(_) => self.curve.as_ref()?.at(u)?,
            Path::Orbit {
                center,
                radius,
                height,
                angle,
            } => {
                let a = lerp(angle[0], angle[1], u).to_radians();
                let r = lerp(radius[0], radius[1], u);
                add(
                    at(center),
                    [a.cos() * r, lerp(height[0], height[1], u), a.sin() * r],
                )
            }
            Path::Follow { subject, offset } => add(subjects(subject)?, *offset),
        })
    }

    fn aim(&self, u: f64, eye: [f64; 3], subjects: Subjects) -> Option<(f64, f64)> {
        let toward = |p: [f64; 3]| angles_along(sub(p, eye));
        match &self.shot.look {
            Look::At(p) => toward(add(self.origin, *p)),
            Look::Pan(a, b) => toward(add(self.origin, lerp3(*a, *b, u))),
            Look::Subject(name) => toward(subjects(name)?),
            Look::Angles { yaw, pitch } => Some((*yaw as f64, *pitch as f64)),
            Look::Ahead => self.ahead(u, subjects),
            Look::Along(angles) => {
                // A path that doesn't move (turning on the spot) spreads the angles over time.
                let at = match &self.curve {
                    Some(curve) if curve.length() > 1e-6 => curve.progress(u),
                    _ => u * angles.len().saturating_sub(1) as f64,
                };
                turn::along(angles, at)
            }
        }
    }

    /// Along the path at `u`: toward a point a little further on (or from one a little back).
    fn ahead(&self, u: f64, subjects: Subjects) -> Option<(f64, f64)> {
        const STEP: f64 = 0.01;
        let (a, b) = if u + STEP <= 1.0 {
            (u, u + STEP)
        } else {
            (u - STEP, u)
        };
        angles_along(sub(self.eye(b, subjects)?, self.eye(a, subjects)?))
    }
}

/// `from` mixed into `to` by `u` (0 all `from`).
pub fn blend(from: Pose, to: Pose, u: f64) -> Pose {
    Pose {
        eye: lerp3(from.eye, to.eye, u),
        yaw: lerp_angle(from.yaw, to.yaw, u),
        pitch: lerp(from.pitch, to.pitch, u),
        roll: lerp(from.roll, to.roll, u),
        fov: lerp(from.fov as f64, to.fov as f64, u) as f32,
    }
}

#[cfg(test)]
mod tests {
    use showcase_api::{dolly, flyby, follow, hold, orbit};

    use super::*;

    const START: Pose = Pose {
        eye: [0.0; 3],
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
        fov: 70.0,
    };

    fn none(_: &str) -> Option<[f64; 3]> {
        None
    }

    #[test]
    fn an_orbit_keeps_its_distance_and_looks_at_the_center() {
        let rig = Rig::new(
            orbit([0.5, 0.0, 0.5], 8.0).height(0.0, 0.0).secs(4.0),
            Pos::new(100, 64, -20),
            70.0,
        );
        let center = [100.5, 64.0, -19.5];
        for clock in [0.0, 1.0, 2.5, 4.0] {
            let pose = rig.pose(clock, &none, START);
            let d = sub(center, pose.eye);
            assert!((math::length(d) - 8.0).abs() < 1e-9, "distance at {clock}");
            let (yaw, pitch) = angles_along(d).expect("a direction");
            assert!(
                (yaw - pose.yaw).abs() < 1e-9 && (pitch - pose.pitch).abs() < 1e-9,
                "aim at {clock}"
            );
        }
    }

    #[test]
    fn a_dolly_looks_the_way_it_moves() {
        let rig = Rig::new(dolly([0.0; 3], [0.0, 0.0, -10.0]), Pos::default(), 70.0);
        let pose = rig.pose(2.0, &none, START);
        assert!(
            pose.yaw.abs() < 1e-6 && pose.pitch.abs() < 1e-6,
            "toward -z: {pose:?}"
        );
    }

    #[test]
    fn a_blend_starts_where_the_camera_was() {
        let rig = Rig::new(
            hold([5.0, 5.0, 5.0])
                .look(Look::Angles {
                    yaw: 90.0,
                    pitch: 0.0,
                })
                .blend(1.0),
            Pos::default(),
            70.0,
        );
        assert_eq!(rig.pose(0.0, &none, START), START, "at the start");
        assert_eq!(rig.pose(1.0, &none, START).eye, [5.0; 3], "after the blend");
    }

    #[test]
    fn a_blend_into_a_zoom_starts_at_the_players_own_view() {
        let rig = Rig::new(
            hold([0.0; 3]).fov(40.0, 40.0).blend(1.0),
            Pos::default(),
            70.0,
        );
        assert_eq!(
            rig.pose(0.0, &none, START).fov,
            70.0,
            "where the blend starts"
        );
        assert_eq!(rig.pose(1.0, &none, START).fov, 40.0, "after it");
        let plain = Rig::new(hold([0.0; 3]), Pos::default(), 70.0);
        assert_eq!(
            plain.pose(0.5, &none, START).fov,
            70.0,
            "a shot without a zoom"
        );
    }

    #[test]
    fn a_roll_tilts_through_the_shot_and_blends_in_from_level() {
        let shot = hold([0.0; 3]).roll(10.0, 30.0).ease(Ease::Linear);
        let rig = Rig::new(shot.clone(), Pos::default(), 70.0);
        assert_eq!(rig.pose(2.5, &none, START).roll, 20.0, "halfway");
        let rig = Rig::new(shot.blend(1.0), Pos::default(), 70.0);
        assert_eq!(rig.pose(0.0, &none, START).roll, 0.0, "level as it was");
        let after = rig.pose(1.0, &none, START).roll;
        assert!(
            (after - 14.0).abs() < 1e-6,
            "the shot's own after the blend: {after}"
        );
    }

    #[test]
    fn a_marked_path_looks_where_it_was_marked() {
        let points = [[0.0; 3], [5.0, 0.0, 0.0], [5.0, 0.0, 5.0]];
        let angles = vec![[0.0, 0.0], [90.0, -30.0], [180.0, 0.0]];
        let shot = flyby(&points).look(Look::Along(angles)).ease(Ease::Linear);
        let rig = Rig::new(shot, Pos::default(), 70.0);
        let middle = rig.pose(2.5, &none, START);
        assert!(
            (middle.eye[0] - 5.0).abs() < 1e-3 && middle.eye[2].abs() < 1e-3,
            "at the second point"
        );
        assert!(
            (middle.yaw - 90.0).abs() < 0.5 && (middle.pitch + 30.0).abs() < 0.5,
            "{middle:?}"
        );
    }

    #[test]
    fn a_path_marked_on_the_spot_still_turns() {
        let angles = vec![[0.0, 0.0], [90.0, 0.0]];
        let shot = flyby(&[[1.0; 3], [1.0; 3]])
            .look(Look::Along(angles))
            .ease(Ease::Linear);
        let rig = Rig::new(shot, Pos::default(), 70.0);
        let yaw = rig.pose(2.5, &none, START).yaw;
        assert!((yaw - 45.0).abs() < 1e-6, "halfway through the turn: {yaw}");
    }

    #[test]
    fn a_flyby_through_no_points_keeps_the_camera_where_it_was() {
        let before = Pose {
            eye: [3.0, 70.0, -8.0],
            ..START
        };
        let rig = Rig::new(flyby(&[]), Pos::new(100, 64, -20), 70.0);
        assert_eq!(rig.pose(1.0, &none, before), before);
    }

    #[test]
    fn an_unknown_subject_keeps_the_camera_where_it_was() {
        let rig = Rig::new(follow("item", [0.0, 1.0, 0.0]), Pos::default(), 70.0);
        assert_eq!(rig.pose(1.0, &none, START), START);
    }
}
