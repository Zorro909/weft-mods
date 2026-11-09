//! Plane geometry for laying out a settlement: lines that curve, outlines, and how far a column
//! is from them. Points are `[x, z]` in blocks; a column's middle is `x + 0.5`.

use std::f32::consts::TAU;

pub type Point = [f32; 2];

/// `line` with its corners cut twice over, so it bends in curves (Chaikin's way). Its ends stay.
pub fn smooth(line: &[Point]) -> Vec<Point> {
    let mut points = line.to_vec();
    for _ in 0..2 {
        let Some(&last) = points.last() else {
            return points;
        };
        let mut cut = vec![points[0]];
        for pair in points.windows(2) {
            cut.push(lerp(pair[0], pair[1], 0.25));
            cut.push(lerp(pair[0], pair[1], 0.75));
        }
        cut.push(last);
        points = cut;
    }
    points
}

/// How far `at` is from the nearest point of `line`.
pub fn distance(line: &[Point], at: Point) -> f32 {
    let segments = line.windows(2).map(|pair| {
        let [a, b] = [pair[0], pair[1]];
        let along = sub(b, a);
        let length = dot(along, along).max(f32::EPSILON);
        let t = (dot(sub(at, a), along) / length).clamp(0.0, 1.0);
        length_of(sub(at, lerp(a, b, t)))
    });
    segments.fold(f32::MAX, f32::min)
}

/// Whether `at` is inside the outline `area`.
pub fn inside(area: &[Point], at: Point) -> bool {
    let mut inside = false;
    for i in 0..area.len() {
        let [a, b] = [area[i], area[(i + 1) % area.len()]];
        let crosses = (a[1] > at[1]) != (b[1] > at[1]);
        if crosses && at[0] < a[0] + (at[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) {
            inside = !inside;
        }
    }
    inside
}

/// Points along `line` a block apart, each with the way the line runs there.
pub fn walk(line: &[Point]) -> Vec<(Point, Point)> {
    let mut points = Vec::new();
    for pair in line.windows(2) {
        let along = sub(pair[1], pair[0]);
        let length = length_of(along);
        let way = scale(along, 1.0 / length.max(f32::EPSILON));
        for step in 0..length.ceil() as usize {
            points.push((add(pair[0], scale(way, step as f32)), way));
        }
    }
    points
}

/// The direction `angle` (radians) points, a block long: 0 is east, a quarter turn south.
pub fn heading(angle: f32) -> Point {
    [angle.cos(), angle.sin()]
}

/// An irregular blob of `corners` points round `middle`: `long` blocks from it along `angle`,
/// `wide` across, each corner pushed in or out by up to a fifth by `jitter` (0 to 1 each).
pub fn blob(middle: Point, angle: f32, [long, wide]: [f32; 2], jitter: &[f32]) -> Vec<Point> {
    let (along, across) = (heading(angle), heading(angle + TAU / 4.0));
    let corners = jitter.len();
    (0..corners)
        .map(|i| {
            let a = TAU * i as f32 / corners as f32;
            let reach = 0.8 + 0.4 * jitter[i];
            let local = add(scale(along, a.cos() * long), scale(across, a.sin() * wide));
            add(middle, scale(local, reach))
        })
        .collect()
}

pub fn add(a: Point, b: Point) -> Point {
    [a[0] + b[0], a[1] + b[1]]
}

pub fn sub(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1]]
}

pub fn scale(a: Point, by: f32) -> Point {
    [a[0] * by, a[1] * by]
}

pub fn dot(a: Point, b: Point) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

pub fn length_of(a: Point) -> f32 {
    dot(a, a).sqrt()
}

fn lerp(a: Point, b: Point, t: f32) -> Point {
    add(a, scale(sub(b, a), t))
}

/// A number from 0 to 1 that looks random but is always the same for a column.
pub fn noise(x: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (z as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 13;
    (h & 0xFFFF) as f32 / 65536.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_smoothed_line_keeps_its_ends_and_cuts_its_corners() {
        let line = smooth(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]);
        assert_eq!(line[0], [0.0, 0.0], "the start");
        assert_eq!(line[line.len() - 1], [10.0, 10.0], "the end");
        assert!(distance(&line, [10.0, 0.0]) > 1.0, "the corner is cut");
    }

    #[test]
    fn a_walk_steps_a_block_at_a_time() {
        let steps = walk(&[[0.0, 0.0], [5.0, 0.0]]);
        assert_eq!(steps.len(), 5, "five blocks");
        assert_eq!(steps[3], ([3.0, 0.0], [1.0, 0.0]), "east, three along");
    }

    #[test]
    fn a_blob_holds_its_middle_and_reaches_further_along_than_across() {
        let jitter = [0.5; 8];
        let area = blob([0.0, 0.0], 0.0, [12.0, 5.0], &jitter);
        assert!(inside(&area, [0.0, 0.0]), "its middle");
        assert!(inside(&area, [10.0, 0.0]), "far along");
        assert!(!inside(&area, [0.0, 7.0]), "not as far across");
    }
}
