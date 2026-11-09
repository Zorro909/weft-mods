//! The heart of a settlement and the roads that grow out of it. A road takes a step at a time,
//! each time the way of three that climbs least, so it winds along the lie of the land.

use std::f32::consts::{FRAC_PI_2, PI};

use structgen_api::Random;

use super::grid::{Cell, Grid, column};
use super::shape::{Point, add, blob, heading, inside, length_of, scale, smooth, sub, walk};
use super::{Kind, Settlement, unit};

/// How far a road goes in one step, in blocks.
pub const STEP: f32 = 7.0;
/// How much a step may turn, either way, in radians: about 20°.
const BEND: f32 = 0.35;

/// A road: its middle line, curved.
pub struct Road {
    pub line: Vec<Point>,
}

impl Road {
    /// How far apart its ends are.
    pub fn span(&self) -> f32 {
        match self.line[..] {
            [first, .., last] => length_of(sub(last, first)),
            _ => 0.0,
        }
    }

    /// The road's last point and the way it runs there.
    pub fn end(&self) -> Option<(Point, Point)> {
        let [.., before, end] = self.line[..] else {
            return None;
        };
        let way = sub(end, before);
        let length = length_of(way);
        (length > 0.0).then(|| (end, scale(way, 1.0 / length)))
    }
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

/// The heart and the roads out of it, marked on `grid`: the main road both ways, then the
/// branches, each leaving it a third of the way along or further, to either side in turn. A
/// branch the ground stops short is tried again elsewhere, a few times.
pub fn lay(
    random: &mut Random,
    grid: &mut Grid,
    s: &Settlement,
    heart: &[Point],
    middle: Point,
    angle: f32,
) -> Vec<Road> {
    let (width, road) = (s.kind.width(), s.kind.road());
    grid.put_area(
        heart,
        if s.kind == Kind::Village {
            Cell::Green
        } else {
            Cell::Square
        },
    );
    let half = width / 2.0;
    let main = main_road(grid, random, heart, middle, angle, s.steps);
    let mut roads: Vec<Road> = main.map(|road| walkable_part(grid, road, half)).into();
    for main in &roads {
        grid.put_road(&main.line, width, road);
    }
    if s.kind == Kind::Village {
        let mut ring = heart.to_vec();
        ring.push(heart[0]);
        grid.put_road(&smooth(&ring), width, road);
    }
    let wanted = random.between(s.branches[0], s.branches[1]) as usize;
    let mut side = if random.chance(0.5) { 1.0 } else { -1.0 };
    for _ in 0..wanted * 3 {
        if roads.len() == 2 + wanted {
            break;
        }
        let from = &roads[random.below(2) as usize];
        let n = from.line.len() as i32;
        let index = random.between(n / 3, (n - 3).max(n / 3)) as usize;
        let tried = branch(grid, random, from, index, side, branch_steps(s.steps));
        let tried = tried.map(|road| walkable_part(grid, road, half));
        side = -side;
        let Some(branch) = tried.filter(|branch| branch.span() >= 2.0 * STEP) else {
            continue;
        };
        grid.put_road(&branch.line, width, road);
        roads.push(branch);
    }
    roads
}

/// A branch's steps: about half the main road's, so it is about as long as the few homes it
/// gets, and not under the shortest branch kept.
fn branch_steps(main: usize) -> usize {
    (main / 2 + 1).max(3)
}

/// The main road through the heart: out from its edge both ways, bending a little as it passes.
fn main_road(
    grid: &Grid,
    random: &mut Random,
    heart: &[Point],
    middle: Point,
    angle: f32,
    steps: usize,
) -> [Road; 2] {
    let bend = (unit(random) - 0.5) * 1.2;
    [angle, angle + PI + bend].map(|way| grow(grid, random, edge(heart, middle, way), way, steps))
}

/// A road branching off `from` at the point `index` along it, to its left for `side` 1 and
/// its right for -1; `None` off a road too short to run any way.
fn branch(
    grid: &Grid,
    random: &mut Random,
    from: &Road,
    index: usize,
    side: f32,
    steps: usize,
) -> Option<Road> {
    let (at, along) = fork(from, index)?;
    let angle = along + side * (FRAC_PI_2 + (unit(random) - 0.5) * 0.8);
    Some(grow(grid, random, at, angle, steps))
}

/// The point `index` along `road`, or its last but one, and the angle the road runs at there;
/// `None` for a road of a single point.
fn fork(road: &Road, index: usize) -> Option<(Point, f32)> {
    let index = index.min(road.line.len().checked_sub(2)?);
    let (at, next) = (road.line[index], road.line[index + 1]);
    Some((at, (next[1] - at[1]).atan2(next[0] - at[0])))
}

/// A road from `start` taking up to `steps` steps along `angle`, each the way of three that
/// climbs least, smoothed. It stops where the ground ends or grows too steep to walk, and before
/// it would cross another road or the heart; only its first step, leaving the road it branches
/// off, may.
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
            let way = heading(a);
            let free = |k: i32| grid.get(column(add(at, scale(way, k as f32)))) == Cell::Free;
            let reach = STEP as i32;
            let open = if step == 0 {
                free(reach)
            } else {
                (1..=reach).all(free)
            };
            let height = grid.walkable(at, way, reach).filter(|_| open)?;
            Some((
                (height - here).abs() as f32 + (a - angle).abs(),
                add(at, scale(way, STEP)),
                a,
            ))
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

/// `road` up to its first stretch that can't be walked, along its middle or across its width
/// out to `half` either side: smoothing cuts corners across columns the steps never looked at,
/// and a street's sides may cross a ledge its middle passes beside.
fn walkable_part(grid: &Grid, road: Road, half: f32) -> Road {
    let mut line = road.line;
    let mut last: Option<i32> = None;
    for i in 1..line.len() {
        if !grid.level(&line[i - 1..=i], half) {
            line.truncate(i);
            break;
        }
        let stretch = walk(&line[i - 1..=i]).into_iter().map(|(at, _)| at);
        for at in stretch.chain([line[i]]) {
            let height = grid.ground(at);
            let step = last.zip(height).is_some_and(|(a, b)| (a - b).abs() > 1);
            if height.is_none() || step {
                line.truncate(i);
                return Road { line };
            }
            last = height;
        }
    }
    Road { line }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settlement::grid::row;

    #[test]
    fn nothing_branches_off_a_road_of_one_point() {
        let stub = Road {
            line: vec![[3.0, 4.0]],
        };
        assert_eq!(fork(&stub, 0), None);
        let east = Road {
            line: vec![[0.0, 0.0], [5.0, 0.0]],
        };
        assert_eq!(
            fork(&east, 9),
            Some(([0.0, 0.0], 0.0)),
            "past the end: the last stretch"
        );
    }

    #[test]
    fn a_ledge_between_two_short_stretches_ends_the_road() {
        let ground = [Some(64), Some(64), Some(66), Some(66)];
        let line = vec![[0.5, 0.5], [1.8, 0.5], [2.2, 0.5], [2.6, 0.5]];
        let walked = walkable_part(&row(&ground), Road { line }, 0.0);
        assert_eq!(walked.line, [[0.5, 0.5], [1.8, 0.5]]);
    }

    #[test]
    fn a_road_ends_before_the_first_stretch_it_cannot_walk() {
        let ground = [
            Some(64),
            Some(64),
            Some(64),
            Some(64),
            None,
            Some(64),
            Some(64),
        ];
        let line = vec![[0.5, 0.5], [2.5, 0.5], [3.5, 0.5], [5.5, 0.5], [6.5, 0.5]];
        let walked = walkable_part(&row(&ground), Road { line }, 0.0);
        assert_eq!(walked.line, [[0.5, 0.5], [2.5, 0.5], [3.5, 0.5]]);
    }

    #[test]
    fn a_road_ends_where_its_side_would_cross_a_ledge() {
        let mut grid = row(&[Some(64); 10]);
        for z in 0..10 {
            for x in 0..10 {
                // A shelf beside the road's middle line, from x 7 on.
                grid.raise([x, z], if z == 6 && x >= 7 { 66 } else { 64 });
            }
        }
        let line = vec![[2.5, 5.5], [4.5, 5.5], [8.5, 5.5]];
        let walked = walkable_part(&grid, Road { line: line.clone() }, 0.0);
        assert_eq!(walked.line, line, "its middle line is level");
        let walked = walkable_part(&grid, Road { line }, 1.5);
        assert_eq!(walked.line, [[2.5, 5.5], [4.5, 5.5]], "but not its side");
    }
}
