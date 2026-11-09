//! The heart of a settlement and the roads that grow out of it. A road takes a step at a time,
//! each time the way of three that climbs least, so it winds along the lie of the land.

use std::f32::consts::{FRAC_PI_2, PI};

use structgen_api::Random;

use super::grid::{Cell, Grid};
use super::shape::{Point, add, blob, heading, inside, scale, smooth};

/// How far a road goes in one step, in blocks.
const STEP: f32 = 7.0;
/// How much a step may turn, either way, in radians: about 20°.
const BEND: f32 = 0.35;

/// A road: its middle line, curved.
pub struct Road {
    pub line: Vec<Point>,
}

/// A number from 0 to 1.
pub fn unit(random: &mut Random) -> f32 {
    random.below(1 << 16) as f32 / 65536.0
}

/// The heart's outline: a blob round `middle`, stretched along `angle`.
pub fn heart(random: &mut Random, middle: Point, angle: f32, size: [f32; 2]) -> Vec<Point> {
    let jitter: Vec<f32> = (0..9).map(|_| unit(random)).collect();
    blob(middle, angle, size, &jitter)
}

/// Where a line from `middle` along `angle` leaves `area`.
pub fn edge(area: &[Point], middle: Point, angle: f32) -> Point {
    let way = heading(angle);
    let mut at = middle;
    while inside(area, at) {
        at = add(at, way);
    }
    at
}

/// The main road through the heart: out from its edge both ways.
pub fn main_road(
    grid: &Grid,
    random: &mut Random,
    heart: &[Point],
    middle: Point,
    angle: f32,
    steps: usize,
) -> [Road; 2] {
    [angle, angle + PI].map(|way| grow(grid, random, edge(heart, middle, way), way, steps))
}

/// A road branching off `from` at the point `index` along it, to one side or the other.
pub fn branch(grid: &Grid, random: &mut Random, from: &Road, index: usize, steps: usize) -> Road {
    let at = from.line[index.min(from.line.len() - 2)];
    let next = from.line[index.min(from.line.len() - 2) + 1];
    let along = (next[1] - at[1]).atan2(next[0] - at[0]);
    let side = if random.chance(0.5) { 1.0 } else { -1.0 };
    let angle = along + side * (FRAC_PI_2 + (unit(random) - 0.5) * 0.8);
    grow(grid, random, at, angle, steps)
}

/// A road from `start` taking up to `steps` steps along `angle`, each the way of three that
/// climbs least. It stops where the ground ends or before it would run into another road.
fn grow(grid: &Grid, random: &mut Random, start: Point, mut angle: f32, steps: usize) -> Road {
    let mut points = vec![start];
    let mut at = start;
    for step in 0..steps {
        let Some(here) = grid.height(column(at)) else {
            break;
        };
        let wobble = (unit(random) - 0.5) * BEND;
        let options = [-BEND, 0.0, BEND].map(|turn| angle + turn + wobble);
        let climb = |a: f32| {
            let next = add(at, scale(heading(a), STEP));
            let open = step == 0 || !grid.get(column(next)).is_road();
            let height = grid.height(column(next)).filter(|_| open)?;
            Some(((height - here).abs() as f32 + (a - angle).abs(), next, a))
        };
        let best = options
            .into_iter()
            .filter_map(climb)
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let Some((_, next, way)) = best else {
            break;
        };
        (at, angle) = (next, way);
        points.push(at);
    }
    Road {
        line: smooth(&points),
    }
}

/// The column a point is in.
pub fn column(at: Point) -> [i32; 2] {
    [at[0].floor() as i32, at[1].floor() as i32]
}

/// Draws the roads onto the grid: lanes in a village, streets with kerbs in a town.
pub fn put(grid: &mut Grid, roads: &[Road], width: f32, cell: Cell) {
    for road in roads {
        if road.line.len() > 1 {
            grid.put_road(&road.line, width, cell);
        }
    }
}
