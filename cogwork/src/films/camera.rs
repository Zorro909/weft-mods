//! Camera math shared across the film's sections: easing speeds, points on an orbit, and linear
//! interpolation so one section's last shot meets the next section's first without a cut or stall.

use showcase_api::Ease;

/// From `from` blocks a second to `to`: a shot going on from the one before at the speed that
/// one ended at, so the camera doesn't stop between them.
pub const fn speeds(from: f32, to: f32) -> Ease {
    Ease::Speeds { from, to }
}

/// Where an orbit round `center` is at `angle` (0 toward +x, 90 toward +z), `radius` out and
/// `height` up: the shots before and after an orbit meet it there.
pub fn on_orbit(center: [f64; 3], radius: f64, height: f64, angle: f64) -> [f64; 3] {
    let (sin, cos) = angle.to_radians().sin_cos();
    [
        center[0] + cos * radius,
        center[1] + height,
        center[2] + sin * radius,
    ]
}

/// The value `part` of the way from `ends[0]` to `ends[1]`.
pub fn between(ends: [f64; 2], part: f64) -> f64 {
    ends[0] + (ends[1] - ends[0]) * part
}

/// The point halfway from `from` to `to`.
pub fn halfway(from: [f64; 3], to: [f64; 3]) -> [f64; 3] {
    [0, 1, 2].map(|axis| between([from[axis], to[axis]], 0.5))
}
