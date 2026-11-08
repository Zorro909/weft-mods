//! A sketch's features drawn onto a grid of cells, one per column of the ground. Where two
//! overlap, the one that ranks higher wins: a street's paving over a path's gravel, a square
//! over a street's kerbs and gutter.

use crate::parse::{Feature, Sketch};

/// How far from a street's middle its gutter reaches.
const GUTTER: f32 = 0.72;

/// What one column of the ground becomes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Ground,
    Hedge,
    Furrow,
    /// Wheat of this age on farmland.
    Crop(u8),
    Rut,
    Path,
    Worn,
    Kerb,
    Paving,
    Patch,
    Gutter,
    Square,
    SquarePatch,
}

impl Cell {
    fn rank(self) -> u8 {
        match self {
            Cell::Ground => 0,
            Cell::Hedge | Cell::Furrow | Cell::Crop(_) => 1,
            Cell::Rut => 2,
            Cell::Path | Cell::Worn => 3,
            Cell::Kerb => 4,
            Cell::Paving | Cell::Patch => 5,
            Cell::Gutter => 6,
            Cell::Square | Cell::SquarePatch => 7,
        }
    }

    /// Its mark on the text map.
    pub fn glyph(self) -> char {
        match self {
            Cell::Ground => ' ',
            Cell::Hedge => 'h',
            Cell::Furrow | Cell::Crop(_) => '%',
            Cell::Rut => 't',
            Cell::Path | Cell::Worn => '.',
            Cell::Kerb => ':',
            Cell::Paving | Cell::Patch => '=',
            Cell::Gutter => '~',
            Cell::Square | Cell::SquarePatch => '+',
        }
    }
}

/// The cells from `origin` (x, z), `size` across.
pub struct Ground {
    pub origin: [i32; 2],
    pub size: [usize; 2],
    cells: Vec<Cell>,
}

impl Ground {
    /// Every feature of `sketch` drawn, on a grid just big enough for them.
    pub fn of(sketch: &Sketch) -> Ground {
        let shapes: Vec<Shape> = sketch.features.iter().map(Shape::of).collect();
        let (low, high) = bounds(&shapes);
        let size = [(high[0] - low[0]) as usize, (high[1] - low[1]) as usize];
        let mut ground = Ground {
            origin: low,
            size,
            cells: vec![Cell::Ground; size[0] * size[1]],
        };
        for shape in &shapes {
            ground.draw(shape);
        }
        ground
    }

    pub fn get(&self, [x, z]: [usize; 2]) -> Cell {
        self.cells[z * self.size[0] + x]
    }

    fn draw(&mut self, shape: &Shape) {
        let (low, high) = shape.bounds();
        for z in low[1]..high[1] {
            for x in low[0]..high[0] {
                let (gx, gz) = ((x - self.origin[0]) as usize, (z - self.origin[1]) as usize);
                let Some(cell) = shape.cell(x, z) else {
                    continue;
                };
                let here = &mut self.cells[gz * self.size[0] + gx];
                if cell.rank() > here.rank() {
                    *here = cell;
                }
            }
        }
    }
}

/// A feature with its line smoothed into curves.
struct Shape<'a> {
    feature: &'a Feature,
    points: Vec<[f32; 2]>,
    /// How far from its line or outline it reaches.
    reach: f32,
}

impl<'a> Shape<'a> {
    fn of(feature: &'a Feature) -> Shape<'a> {
        let (points, reach) = match feature {
            Feature::Path { width, line } => (smooth(line), width / 2.0 + 1.0),
            Feature::Track { line } => (smooth(line), 2.0),
            Feature::Street { width, line } => (smooth(line), width / 2.0 + 1.5),
            Feature::Hedge { line } => (smooth(line), 1.0),
            Feature::Square { area } | Feature::Field { area } | Feature::Garden { area } => {
                (area.clone(), 0.0)
            }
        };
        Shape {
            feature,
            points,
            reach,
        }
    }

    fn bounds(&self) -> ([i32; 2], [i32; 2]) {
        let low = [0, 1].map(|i| {
            let least = self.points.iter().map(|p| p[i]).fold(f32::MAX, f32::min);
            (least - self.reach).floor() as i32 - 1
        });
        let high = [0, 1].map(|i| {
            let most = self.points.iter().map(|p| p[i]).fold(f32::MIN, f32::max);
            (most + self.reach).ceil() as i32 + 1
        });
        (low, high)
    }

    /// What it makes of column (`x`, `z`), `None` where it doesn't reach.
    fn cell(&self, x: i32, z: i32) -> Option<Cell> {
        let at = [x as f32 + 0.5, z as f32 + 0.5];
        let roll = noise(x, z);
        match self.feature {
            Feature::Path { width, .. } => {
                let d = distance(&self.points, at);
                let ragged = d <= width / 2.0 + 0.7 && roll < 0.3;
                let on = d <= width / 2.0 || ragged;
                on.then_some(if roll > 0.9 { Cell::Worn } else { Cell::Path })
            }
            Feature::Track { .. } => {
                let d = distance(&self.points, at);
                ((d - 1.1).abs() <= 0.55).then_some(Cell::Rut)
            }
            Feature::Street { width, .. } => {
                let d = distance(&self.points, at);
                match d {
                    // Past half a cell's diagonal, so a gutter at an angle runs on unbroken.
                    d if d <= GUTTER => Some(Cell::Gutter),
                    d if d <= width / 2.0 => Some(if roll < 0.07 {
                        Cell::Patch
                    } else {
                        Cell::Paving
                    }),
                    d if d <= width / 2.0 + 1.0 => Some(Cell::Kerb),
                    _ => None,
                }
            }
            Feature::Hedge { .. } => (distance(&self.points, at) <= 0.6).then_some(Cell::Hedge),
            Feature::Square { area } => inside(area, at).then_some(if roll < 0.06 {
                Cell::SquarePatch
            } else {
                Cell::Square
            }),
            Feature::Field { area } => inside(area, at).then(|| strip(x, 4, &[3, 5, 7, 7])),
            Feature::Garden { area } => inside(area, at).then(|| strip(x, 2, &[2, 4, 5])),
        }
    }
}

/// Strips `width` blocks wide along z, a furrow between each, each strip's wheat of one age.
fn strip(x: i32, width: i32, ages: &[u8]) -> Cell {
    let index = x.div_euclid(width + 1);
    match x.rem_euclid(width + 1) == width {
        true => Cell::Furrow,
        false => Cell::Crop(ages[(noise(index, 7) * ages.len() as f32) as usize % ages.len()]),
    }
}

/// The lowest and highest corners over every shape.
fn bounds(shapes: &[Shape]) -> ([i32; 2], [i32; 2]) {
    let all: Vec<([i32; 2], [i32; 2])> = shapes.iter().map(Shape::bounds).collect();
    let low = [0, 1].map(|i| all.iter().map(|b| b.0[i]).min().unwrap_or(0));
    let high = [0, 1].map(|i| all.iter().map(|b| b.1[i]).max().unwrap_or(1));
    (low, high)
}

/// `line` with its corners cut three times over, so it bends in curves (Chaikin's way).
fn smooth(line: &[[f32; 2]]) -> Vec<[f32; 2]> {
    let mut points = line.to_vec();
    for _ in 0..3 {
        let mut cut = vec![points[0]];
        for pair in points.windows(2) {
            let [a, b] = [pair[0], pair[1]];
            cut.push([0.75 * a[0] + 0.25 * b[0], 0.75 * a[1] + 0.25 * b[1]]);
            cut.push([0.25 * a[0] + 0.75 * b[0], 0.25 * a[1] + 0.75 * b[1]]);
        }
        cut.push(points[points.len() - 1]);
        points = cut;
    }
    points
}

/// How far `at` is from the nearest point of `line`.
fn distance(line: &[[f32; 2]], at: [f32; 2]) -> f32 {
    if line.len() == 1 {
        return ((at[0] - line[0][0]).powi(2) + (at[1] - line[0][1]).powi(2)).sqrt();
    }
    line.windows(2)
        .map(|pair| {
            let [a, b] = [pair[0], pair[1]];
            let along = [b[0] - a[0], b[1] - a[1]];
            let length = along[0] * along[0] + along[1] * along[1];
            let t =
                (((at[0] - a[0]) * along[0] + (at[1] - a[1]) * along[1]) / length).clamp(0.0, 1.0);
            let near = [a[0] + t * along[0], a[1] + t * along[1]];
            ((at[0] - near[0]).powi(2) + (at[1] - near[1]).powi(2)).sqrt()
        })
        .fold(f32::MAX, f32::min)
}

/// Whether `at` is inside the polygon with corners `area`.
fn inside(area: &[[f32; 2]], at: [f32; 2]) -> bool {
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

/// A number from 0 to 1 that looks random but is always the same for a column.
fn noise(x: i32, z: i32) -> f32 {
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
    fn a_square_holds_its_inside_only() {
        let area = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        assert!(inside(&area, [5.0, 5.0]), "the middle");
        assert!(!inside(&area, [11.0, 5.0]), "east of it");
    }

    #[test]
    fn a_gutter_at_an_angle_is_unbroken() {
        let line = [[0.0, 0.0], [20.0, 20.0]];
        let gutter = |x: i32, z: i32| distance(&line, [x as f32 + 0.5, z as f32 + 0.5]) <= GUTTER;
        for i in 2..18 {
            assert!(gutter(i, i), "on the line at {i}");
            assert!(gutter(i + 1, i), "joined to the next one along x at {i}");
        }
    }

    #[test]
    fn a_smoothed_line_keeps_its_ends() {
        let line = smooth(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]);
        assert_eq!(line[0], [0.0, 0.0], "the start");
        assert_eq!(line[line.len() - 1], [10.0, 10.0], "the end");
        assert!(distance(&line, [10.0, 0.0]) > 1.0, "the corner is cut");
    }
}
