//! A walker's sway, for shots that take `Shot::bob`: vanilla's view bobbing, worked out from how
//! far the camera has gone and how fast. The engine's own bob, which the film camera's
//! `EFFECTS` would bring, stays still in a film: it follows the player walking on the ground,
//! and a film's player flies (`animate_bob` and `bobbed` in Weft's `client/mod.rs`).

use std::f64::consts::PI;

use super::math::forward;
use super::{Pose, Rig};

/// Vanilla numbers: the sway's phase goes on 0.6 of a half turn for each block walked, and its
/// size is the speed in blocks a tick (20 ticks a second), at most 0.1. It moves the view half
/// that far sideways and that far up, and tilts it 3 degrees of roll and 5 of pitch for each
/// block a tick.
const STRIDE: f64 = 0.6;
const TICKS_PER_SEC: f64 = 20.0;
const MOST: f64 = 0.1;
const SIDEWAYS: f64 = 0.5;
const ROLL: f64 = 3.0;
const PITCH: f64 = 5.0;

/// Seconds either side of a moment that the speed a shot sways at is measured over.
const SPEED_SPAN: f32 = 0.02;

/// How far the sway moves and tilts the camera: in blocks to its right and up, and in degrees.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sway {
    pub right: f64,
    pub up: f64,
    pub roll: f64,
    pub pitch: f64,
}

/// The sway `walked` blocks into a walk at `speed` blocks a second.
pub fn sway(walked: f64, speed: f64) -> Sway {
    let phase = -walked * STRIDE * PI;
    let size = (speed.abs() / TICKS_PER_SEC).min(MOST);
    // The engine moves and tilts the world by these, so the camera goes the other way.
    Sway {
        right: -phase.sin() * size * SIDEWAYS,
        up: (phase.cos() * size).abs(),
        roll: phase.sin() * size * ROLL,
        pitch: -((phase - 0.2).cos() * size).abs() * PITCH,
    }
}

impl Sway {
    /// `pose` swayed: moved across the picture and tilted.
    pub fn applied(self, pose: Pose) -> Pose {
        let right = forward(pose.yaw - 90.0, 0.0);
        let up = forward(pose.yaw, pose.pitch + 90.0);
        Pose {
            eye: pose.eye + right * self.right + up * self.up,
            pitch: pose.pitch + self.pitch,
            roll: pose.roll + self.roll,
            ..pose
        }
    }
}

impl Rig {
    /// The shot swaying on from a walk of `blocks` before it, in step with the shot it follows.
    pub fn after_walking(mut self, blocks: f64) -> Rig {
        if let Some(walk) = &mut self.walk {
            walk[0] = blocks;
        }
        self
    }

    /// How far the walk has gone at the shot's end: for the shot after, if it sways too.
    pub fn walked(&self) -> f64 {
        self.walk.map_or(0.0, |[before, length]| before + length)
    }

    /// `pose`, `clock` seconds into a shot that sways, swayed by how far and fast it has gone.
    pub(super) fn swayed(&self, pose: Pose, clock: f32) -> Pose {
        let Some([before, length]) = self.walk else {
            return pose;
        };
        let span = [(clock - SPEED_SPAN).max(0.0), clock + SPEED_SPAN];
        let moved = self.progress(span[1]) - self.progress(span[0]);
        let speed = length * moved / (span[1] - span[0]) as f64;
        sway(before + length * self.progress(clock), speed).applied(pose)
    }
}

#[cfg(test)]
mod tests {
    use showcase_api::{Ease, Pos, dolly, hold};
    use weft_sdk::Vec3;

    use super::*;

    const START: Pose = Pose {
        eye: Vec3::ZERO,
        yaw: 0.0,
        pitch: 0.0,
        roll: 0.0,
        fov: 70.0,
    };

    fn none(_: &str) -> Option<Vec3> {
        None
    }

    #[test]
    fn only_a_shot_that_bobs_sways() {
        let walk = dolly([0.0; 3], [0.0, 0.0, -20.0])
            .ease(Ease::Linear)
            .secs(5.0);
        let steady = Rig::new(walk.clone(), Pos::default(), 70.0).pose(1.3, &none, START);
        assert!(steady.eye.y.abs() < 1e-9 && steady.roll == 0.0, "level");
        let swaying = Rig::new(walk.bob(), Pos::default(), 70.0).pose(1.3, &none, START);
        assert!(swaying.eye.y > 0.05, "lifted mid-step: {swaying:?}");
        assert!(swaying.roll != 0.0, "and tilted");
    }

    #[test]
    fn a_walk_counts_on_from_the_shot_before() {
        let walk = dolly([0.0; 3], [0.0, 0.0, -20.0]).bob();
        let rig = Rig::new(walk, Pos::default(), 70.0).after_walking(7.0);
        assert!((rig.walked() - 27.0).abs() < 1e-6, "{}", rig.walked());
        let steady = Rig::new(hold([0.0; 3]), Pos::default(), 70.0).after_walking(7.0);
        assert_eq!(
            steady.walked(),
            0.0,
            "a shot that doesn't sway ends the walk"
        );
    }

    #[test]
    fn standing_still_does_not_sway() {
        let still = sway(12.3, 0.0);
        assert_eq!(still.right.abs() + still.up + still.roll.abs(), 0.0);
        assert_eq!(still.pitch, 0.0, "no nod either");
    }

    #[test]
    fn walking_sways_by_vanillas_numbers() {
        // 4.3 blocks a second is past 2, where the sway is at its fullest.
        let start = sway(0.0, 4.3);
        assert!((start.up - MOST).abs() < 1e-9, "a tenth up, mid-step");
        assert!(start.right.abs() < 1e-9, "none sideways yet");
        let stride = sway(1.0 / (2.0 * STRIDE), 4.3);
        assert!(stride.up.abs() < 1e-9, "down again at the footfall");
        assert!(
            (stride.right.abs() - MOST * SIDEWAYS).abs() < 1e-9,
            "fully aside"
        );
        assert!((stride.roll.abs() - MOST * ROLL).abs() < 1e-9, "and tilted");
    }

    #[test]
    fn slow_walking_sways_less() {
        let slow = sway(0.0, 1.0);
        assert!(
            (slow.up - 0.05).abs() < 1e-9,
            "half as much at 1 block a second"
        );
    }
}
