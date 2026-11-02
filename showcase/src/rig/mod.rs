//! The camera rig: where a shot puts the eye and where it looks, at any moment of it.

pub mod curve;
pub mod ease;

use showcase_api::{Look, Path, Shot};

use self::curve::Curve;
use self::ease::{add, angles_along, eased, lerp, lerp_angle, lerp3, sub};

/// A camera: eye position, and yaw, pitch and field of view in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub eye: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
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
    pub fn new(shot: Shot, origin: [i32; 3], own_fov: f32) -> Rig {
        let origin = origin.map(f64::from);
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
        let pose = Pose {
            eye,
            yaw,
            pitch,
            fov,
        };
        if clock >= self.shot.blend {
            return pose;
        }
        blend(
            before,
            pose,
            eased(showcase_api::Ease::Smooth, (clock / self.shot.blend) as f64),
        )
    }

    fn eye(&self, u: f64, subjects: Subjects) -> Option<[f64; 3]> {
        let at = |p: &[f64; 3]| add(self.origin, *p);
        Some(match &self.shot.path {
            Path::Still(p) => at(p),
            Path::Line(a, b) => lerp3(at(a), at(b), u),
            Path::Spline(_) => self.curve.as_ref()?.at(u),
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
        fov: lerp(from.fov as f64, to.fov as f64, u) as f32,
    }
}

#[cfg(test)]
mod tests {
    use showcase_api::{Look, dolly, hold, orbit};

    use super::*;

    const START: Pose = Pose {
        eye: [0.0; 3],
        yaw: 0.0,
        pitch: 0.0,
        fov: 70.0,
    };

    fn none(_: &str) -> Option<[f64; 3]> {
        None
    }

    #[test]
    fn an_orbit_keeps_its_distance_and_looks_at_the_center() {
        let rig = Rig::new(
            orbit([0.5, 0.0, 0.5], 8.0).height(0.0, 0.0).secs(4.0), [100, 64, -20], 70.0);
        let center = [100.5, 64.0, -19.5];
        for clock in [0.0, 1.0, 2.5, 4.0] {
            let pose = rig.pose(clock, &none, START);
            let d = sub(center, pose.eye);
            assert!((ease::length(d) - 8.0).abs() < 1e-9, "distance at {clock}");
            let (yaw, pitch) = angles_along(d).expect("a direction");
            assert!(
                (yaw - pose.yaw).abs() < 1e-9 && (pitch - pose.pitch).abs() < 1e-9,
                "aim at {clock}"
            );
        }
    }

    #[test]
    fn a_dolly_looks_the_way_it_moves() {
        let rig = Rig::new(dolly([0.0; 3], [0.0, 0.0, -10.0]), [0; 3], 70.0);
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
                .blend(1.0), [0; 3], 70.0);
        assert_eq!(rig.pose(0.0, &none, START), START, "at the start");
        assert_eq!(rig.pose(1.0, &none, START).eye, [5.0; 3], "after the blend");
    }

    #[test]
    fn a_blend_into_a_zoom_starts_at_the_players_own_view() {
        let rig = Rig::new(hold([0.0; 3]).fov(40.0, 40.0).blend(1.0), [0; 3], 70.0);
        assert_eq!(
            rig.pose(0.0, &none, START).fov,
            70.0,
            "where the blend starts"
        );
        assert_eq!(rig.pose(1.0, &none, START).fov, 40.0, "after it");
        let plain = Rig::new(hold([0.0; 3]), [0; 3], 70.0);
        assert_eq!(
            plain.pose(0.5, &none, START).fov,
            70.0,
            "a shot without a zoom"
        );
    }

    #[test]
    fn an_unknown_subject_keeps_the_camera_where_it_was() {
        let rig = Rig::new(showcase_api::follow("item", [0.0, 1.0, 0.0]), [0; 3], 70.0);
        assert_eq!(rig.pose(1.0, &none, START), START);
    }
}
