//! Easing curves and the angle and vector arithmetic the rig shares.

use showcase_api::Ease;

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

pub fn lerp3(a: [f64; 3], b: [f64; 3], u: f64) -> [f64; 3] {
    [0, 1, 2].map(|i| lerp(a[i], b[i], u))
}

pub fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|i| a[i] + b[i])
}

pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|i| a[i] - b[i])
}

pub fn length(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// From `a` toward `b` the short way round, in degrees.
pub fn lerp_angle(a: f64, b: f64, u: f64) -> f64 {
    a + (b - a + 540.0).rem_euclid(360.0) * u - 180.0 * u
}

/// Yaw and pitch in degrees that look along `dir` (the game's: yaw 0 looks toward -z, pitch
/// up is positive); `None` for no direction.
pub fn angles_along(dir: [f64; 3]) -> Option<(f64, f64)> {
    let flat = (dir[0] * dir[0] + dir[2] * dir[2]).sqrt();
    if flat < 1e-9 && dir[1].abs() < 1e-9 {
        return None;
    }
    let yaw = (-dir[0]).atan2(-dir[2]).to_degrees();
    Some((yaw, dir[1].atan2(flat).to_degrees()))
}

/// The unit vector a camera at `yaw` and `pitch` (degrees) looks along.
pub fn forward(yaw: f64, pitch: f64) -> [f64; 3] {
    let (yaw, pitch) = (yaw.to_radians(), pitch.to_radians());
    [
        -yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    ]
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
        let dir = [3.0, -2.0, 5.0];
        let (yaw, pitch) = angles_along(dir).expect("a direction");
        let f = forward(yaw, pitch);
        let n = length(dir);
        assert!((0..3).all(|i| (f[i] - dir[i] / n).abs() < 1e-9), "{f:?}");
    }
}
