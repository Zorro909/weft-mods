//! The arithmetic the rig shares: easing, mixing, vectors and angles.

use showcase_api::Ease;
use weft_sdk::Vec3;

/// Progress through a shot after easing; `u` is the share of its time gone, 0 to 1.
pub fn eased(ease: Ease, u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    match ease {
        Ease::Linear => u,
        Ease::Smooth => u * u * u * (u * (u * 6.0 - 15.0) + 10.0),
        Ease::In => u * u * u,
        Ease::Out => 1.0 - (1.0 - u).powi(3),
    }
}

pub fn lerp(a: f64, b: f64, u: f64) -> f64 {
    a + (b - a) * u
}

pub fn lerp3(a: Vec3, b: Vec3, u: f64) -> Vec3 {
    a + (b - a) * u
}

pub fn length(v: Vec3) -> f64 {
    v.length_squared().sqrt()
}

/// From `a` toward `b` the short way round, in degrees.
pub fn lerp_angle(a: f64, b: f64, u: f64) -> f64 {
    a + short_turn(a, b) * u
}

/// The turn from `a` to `b` the short way round, -180 to 180 degrees.
pub fn short_turn(a: f64, b: f64) -> f64 {
    (b - a + 540.0).rem_euclid(360.0) - 180.0
}

/// Yaw and pitch in degrees that look along `dir` (the game's: yaw 0 looks toward -z, pitch
/// up is positive); `None` for no direction.
pub fn angles_along(dir: Vec3) -> Option<(f64, f64)> {
    let flat = dir.x.hypot(dir.z);
    if flat < 1e-9 && dir.y.abs() < 1e-9 {
        return None;
    }
    let yaw = (-dir.x).atan2(-dir.z).to_degrees();
    Some((yaw, dir.y.atan2(flat).to_degrees()))
}

/// The unit vector a camera at `yaw` and `pitch` (degrees) looks along.
pub fn forward(yaw: f64, pitch: f64) -> Vec3 {
    let (yaw, pitch) = (yaw.to_radians(), pitch.to_radians());
    Vec3::new(
        -yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ease_starts_at_0_and_ends_at_1() {
        for ease in [Ease::Linear, Ease::Smooth, Ease::In, Ease::Out] {
            assert!(eased(ease, 0.0).abs() < 1e-12, "{ease:?} at the start");
            assert!(
                (eased(ease, 1.0) - 1.0).abs() < 1e-12,
                "{ease:?} at the end"
            );
        }
    }

    #[test]
    fn angles_turn_the_short_way() {
        assert!(
            (lerp_angle(350.0, 10.0, 0.5).rem_euclid(360.0)).abs() < 1e-9,
            "through north"
        );
        assert!(
            (lerp_angle(10.0, 350.0, 1.0).rem_euclid(360.0) - 350.0).abs() < 1e-9,
            "arrives"
        );
    }

    #[test]
    fn looking_along_a_direction_points_forward_along_it() {
        let dir = Vec3::new(3.0, -2.0, 5.0);
        let (yaw, pitch) = angles_along(dir).expect("a direction");
        let f = forward(yaw, pitch);
        let off = length(f - dir * (1.0 / length(dir)));
        assert!(off < 1e-9, "{f:?}");
    }
}
