//! A smooth curve through points, walked at an even speed: a centripetal Catmull-Rom spline,
//! which never loops or overshoots between points that lie close together.

use super::ease::{length, lerp3, sub};

/// Samples per stretch between two points; enough that the chords follow the curve closely.
const SAMPLES: usize = 48;

/// Points along the curve and the distance travelled to each.
pub struct Curve {
    points: Vec<[f64; 3]>,
    distance: Vec<f64>,
}

impl Curve {
    pub fn new(through: &[[f64; 3]]) -> Curve {
        let mut points = Vec::new();
        if through.len() < 2 {
            points.extend(through.first().copied());
        }
        for i in 0..through.len().saturating_sub(1) {
            let at =
                |k: isize| through[(i as isize + k).clamp(0, through.len() as isize - 1) as usize];
            let ends = [at(-1), at(0), at(1), at(2)];
            points.extend((0..SAMPLES).map(|s| catmull_rom(ends, s as f64 / SAMPLES as f64)));
        }
        points.extend(through.last().filter(|_| through.len() >= 2).copied());
        let mut distance = vec![0.0; points.len()];
        for i in 1..points.len() {
            distance[i] = distance[i - 1] + length(sub(points[i], points[i - 1]));
        }
        Curve { points, distance }
    }

    pub fn length(&self) -> f64 {
        self.distance.last().copied().unwrap_or(0.0)
    }

    /// Where the curve is after `u` of its length (0 to 1).
    pub fn at(&self, u: f64) -> [f64; 3] {
        let Some(first) = self.points.first() else {
            return [0.0; 3];
        };
        let want = u.clamp(0.0, 1.0) * self.length();
        let i = self.distance.partition_point(|d| *d < want);
        if i == 0 {
            return *first;
        }
        let Some(&b) = self.points.get(i) else {
            return self.points[i - 1];
        };
        let (da, db) = (self.distance[i - 1], self.distance[i]);
        let f = if db > da {
            (want - da) / (db - da)
        } else {
            1.0
        };
        lerp3(self.points[i - 1], b, f)
    }
}

/// The centripetal Catmull-Rom point `t` (0 to 1) of the way from `p[1]` to `p[2]`.
fn catmull_rom(p: [[f64; 3]; 4], t: f64) -> [f64; 3] {
    // Knot spacing by the square root of the distance; a little floor keeps doubled points apart.
    let gap = |a: [f64; 3], b: [f64; 3]| length(sub(b, a)).sqrt().max(1e-4);
    let t1 = gap(p[0], p[1]);
    let t2 = t1 + gap(p[1], p[2]);
    let t3 = t2 + gap(p[2], p[3]);
    let t = t1 + (t2 - t1) * t;
    let mix = |a: [f64; 3], b: [f64; 3], ta: f64, tb: f64| lerp3(a, b, (t - ta) / (tb - ta));
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

    fn close(a: [f64; 3], b: [f64; 3]) -> bool {
        length(sub(a, b)) < 1e-6
    }

    #[test]
    fn the_curve_passes_through_every_point() {
        let through = [
            [0.0, 0.0, 0.0],
            [4.0, 1.0, 0.0],
            [4.0, 1.0, 6.0],
            [0.0, 3.0, 9.0],
        ];
        let curve = Curve::new(&through);
        assert!(close(curve.at(0.0), through[0]), "starts at the first");
        assert!(close(curve.at(1.0), through[3]), "ends at the last");
        let hits = curve
            .points
            .iter()
            .filter(|p| through.iter().any(|t| close(**p, *t)))
            .count();
        assert_eq!(hits, 4, "goes through the middle ones");
    }

    #[test]
    fn a_straight_curve_is_walked_at_an_even_speed() {
        let curve = Curve::new(&[[0.0; 3], [1.0, 0.0, 0.0], [10.0, 0.0, 0.0]]);
        assert!(
            (curve.length() - 10.0).abs() < 1e-6,
            "length {}",
            curve.length()
        );
        assert!(
            close(curve.at(0.25), [2.5, 0.0, 0.0]),
            "a quarter: {:?}",
            curve.at(0.25)
        );
    }

    #[test]
    fn one_point_is_a_curve_that_stays() {
        let curve = Curve::new(&[[1.0, 2.0, 3.0]]);
        assert!(close(curve.at(0.7), [1.0, 2.0, 3.0]));
    }
}
