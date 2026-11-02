//! Turning smoothly through a list of yaw and pitch angles (`Look::Along`): a Catmull-Rom curve
//! through them, yaw taking the short way round between neighbours, so the camera turns without
//! stopping at each one.

/// Yaw and pitch in degrees at `at` along `angles`: 0 the first, 1.5 halfway from the second to
/// the third. `None` for no angles.
pub fn along(angles: &[[f32; 2]], at: f64) -> Option<(f64, f64)> {
    let last = angles.len().checked_sub(1)?;
    let unwrapped = unwrap(angles);
    let at = at.clamp(0.0, last as f64);
    let i = (at.floor() as usize).min(last.saturating_sub(1));
    let t = at - i as f64;
    let get = |k: isize| unwrapped[(i as isize + k).clamp(0, last as isize) as usize];
    let (p0, p1, p2, p3) = (get(-1), get(0), get(1), get(2));
    let pick = |n: usize| catmull_rom(p0[n], p1[n], p2[n], p3[n], t);
    Some((pick(0), pick(1)))
}

/// The angles with each yaw moved by whole turns to within half a turn of the one before.
fn unwrap(angles: &[[f32; 2]]) -> Vec<[f64; 2]> {
    let mut out: Vec<[f64; 2]> = Vec::with_capacity(angles.len());
    for a in angles {
        let mut yaw = a[0] as f64;
        if let Some(before) = out.last() {
            yaw = before[0] + (yaw - before[0] + 540.0).rem_euclid(360.0) - 180.0;
        }
        out.push([yaw, a[1] as f64]);
    }
    out
}

/// The uniform Catmull-Rom value `t` (0 to 1) of the way from `b` to `c`.
fn catmull_rom(a: f64, b: f64, c: f64, d: f64, t: f64) -> f64 {
    let t2 = t * t;
    0.5 * (2.0 * b
        + (c - a) * t
        + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
        + (3.0 * b - a - 3.0 * c + d) * t2 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_turn_passes_every_angle() {
        let angles = [[0.0, 0.0], [90.0, 10.0], [180.0, -20.0]];
        for (i, a) in angles.iter().enumerate() {
            let (yaw, pitch) = along(&angles, i as f64).expect("angles");
            assert!(
                (yaw - a[0] as f64).abs() < 1e-9 && (pitch - a[1] as f64).abs() < 1e-9,
                "at {i}"
            );
        }
    }

    #[test]
    fn yaw_turns_the_short_way_round() {
        let (yaw, _) = along(&[[170.0, 0.0], [-170.0, 0.0]], 0.5).expect("angles");
        assert!((yaw - 180.0).abs() < 1e-9, "through 180, not 0: {yaw}");
    }

    #[test]
    fn one_angle_holds() {
        assert_eq!(along(&[[30.0, 5.0]], 0.7), Some((30.0, 5.0)));
        assert_eq!(along(&[], 0.0), None);
    }
}
