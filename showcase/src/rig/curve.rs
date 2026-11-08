//! A smooth curve through points, walked at an even speed: a centripetal Catmull-Rom spline,
//! which never loops or overshoots between points that lie close together.

use weft_sdk::Vec3;

use super::math::{length, lerp3};

/// Samples per stretch between two points; enough that the chords follow the curve closely.
const SAMPLES: usize = 48;

/// Points along the curve and the distance travelled to each.
pub struct Curve {
    points: Vec<Vec3>,
    distance: Vec<f64>,
}

impl Curve {
    pub fn new(through: &[Vec3]) -> Curve {
        let points = sample(through);
        let mut distance = vec![0.0; points.len()];
        for sample in 1..points.len() {
            let chord = length(points[sample] - points[sample - 1]);
            distance[sample] = distance[sample - 1] + chord;
        }
        Curve { points, distance }
    }

    pub fn length(&self) -> f64 {
        self.distance.last().copied().unwrap_or(0.0)
    }

    /// Where the curve is after `u` of its length (0 to 1); `None` for a curve through no points.
    pub fn at(&self, u: f64) -> Option<Vec3> {
        let (sample, fraction) = self.locate(u);
        let here = *self.points.get(sample)?;
        Some(match self.points.get(sample + 1) {
            Some(&next) => lerp3(here, next, fraction),
            None => here,
        })
    }

    /// How far along the points it was made through the curve is after `u` of its length:
    /// 1.5 is halfway between the second and the third.
    pub fn progress(&self, u: f64) -> f64 {
        let (sample, fraction) = self.locate(u);
        (sample as f64 + fraction) / SAMPLES as f64
    }

    /// The sample at or before `u` of the length, and how far it is from there to the next.
    fn locate(&self, u: f64) -> (usize, f64) {
        let want = u.clamp(0.0, 1.0) * self.length();
        // The first sample at least `want` along: `want` lies on the chord that ends there.
        let end = self.distance.partition_point(|d| *d < want);
        let Some(start) = end.checked_sub(1) else {
            return (0, 0.0);
        };
        let Some(&to) = self.distance.get(end) else {
            return (start, 0.0);
        };
        let from = self.distance[start];
        let fraction = if to > from {
            (want - from) / (to - from)
        } else {
            1.0
        };
        (start, fraction)
    }
}

/// `SAMPLES` points along each stretch between two of `through`, and the last one.
fn sample(through: &[Vec3]) -> Vec<Vec3> {
    let Some(&last) = through.last() else {
        return Vec::new();
    };
    let mut points = Vec::with_capacity((through.len() - 1) * SAMPLES + 1);
    let last_index = through.len() as isize - 1;
    for stretch in 0..through.len() - 1 {
        // The stretch's ends and their neighbours, the first and last points standing in for
        // the neighbours beyond the ends.
        let at = |offset: isize| through[(stretch as isize + offset).clamp(0, last_index) as usize];
        let ends = [at(-1), at(0), at(1), at(2)];
        let along = |step: usize| step as f64 / SAMPLES as f64;
        points.extend((0..SAMPLES).map(|step| catmull_rom(ends, along(step))));
    }
    points.push(last);
    points
}

/// The centripetal Catmull-Rom point `t` (0 to 1) of the way from `p[1]` to `p[2]`.
fn catmull_rom(p: [Vec3; 4], t: f64) -> Vec3 {
    // Knot spacing by the square root of the distance; a little floor keeps doubled points apart.
    let gap = |a: Vec3, b: Vec3| length(b - a).sqrt().max(1e-4);
    let t1 = gap(p[0], p[1]);
    let t2 = t1 + gap(p[1], p[2]);
    let t3 = t2 + gap(p[2], p[3]);
    let t = t1 + (t2 - t1) * t;
    let mix = |a: Vec3, b: Vec3, ta: f64, tb: f64| lerp3(a, b, (t - ta) / (tb - ta));
    let a1 = mix(p[0], p[1], 0.0, t1);
    let a2 = mix(p[1], p[2], t1, t2);
    let a3 = mix(p[2], p[3], t2, t3);
    let b1 = mix(a1, a2, 0.0, t2);
    let b2 = mix(a2, a3, t1, t3);
    mix(b1, b2, t1, t2)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec3, b: impl Into<Vec3>) -> bool {
        length(a - b.into()) < 1e-6
    }

    fn through(points: &[[f64; 3]]) -> Curve {
        let points: Vec<Vec3> = points.iter().map(|&p| p.into()).collect();
        Curve::new(&points)
    }

    #[test]
    fn the_curve_passes_through_every_point() {
        let through = [
            [0.0, 0.0, 0.0],
            [4.0, 1.0, 0.0],
            [4.0, 1.0, 6.0],
            [0.0, 3.0, 9.0],
        ];
        let curve = self::through(&through);
        assert!(
            close(curve.at(0.0).unwrap(), through[0]),
            "starts at the first"
        );
        assert!(
            close(curve.at(1.0).unwrap(), through[3]),
            "ends at the last"
        );
        let hits = curve
            .points
            .iter()
            .filter(|point| through.iter().any(|&marked| close(**point, marked)))
            .count();
        assert_eq!(hits, 4, "goes through the middle ones");
    }

    #[test]
    fn a_straight_curve_is_walked_at_an_even_speed() {
        let curve = through(&[[0.0; 3], [1.0, 0.0, 0.0], [10.0, 0.0, 0.0]]);
        assert!(
            (curve.length() - 10.0).abs() < 1e-6,
            "length {}",
            curve.length()
        );
        let quarter = curve.at(0.25).unwrap();
        assert!(close(quarter, [2.5, 0.0, 0.0]), "a quarter: {quarter:?}");
    }

    #[test]
    fn progress_counts_the_points_passed() {
        let curve = through(&[[0.0; 3], [2.0, 0.0, 0.0], [4.0, 0.0, 0.0]]);
        assert!(curve.progress(0.0).abs() < 1e-9, "at the first");
        assert!(
            (curve.progress(0.5) - 1.0).abs() < 1e-6,
            "at the second: {}",
            curve.progress(0.5)
        );
        assert!(
            (curve.progress(1.0) - 2.0).abs() < 1e-9,
            "at the last: {}",
            curve.progress(1.0)
        );
    }

    #[test]
    fn one_point_is_a_curve_that_stays() {
        let curve = through(&[[1.0, 2.0, 3.0]]);
        assert_eq!(curve.at(0.7), Some(Vec3::new(1.0, 2.0, 3.0)));
    }

    #[test]
    fn no_points_are_nowhere() {
        assert_eq!(Curve::new(&[]).at(0.5), None);
    }
}
