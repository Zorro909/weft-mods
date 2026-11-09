//! A sketch's features drawn onto a grid of cells, one per column of the ground. Where two
//! overlap, the one that ranks higher wins: a street's paving over a path's gravel, a square
//! over a street's kerbs and gutter.

use crate::parse::{Feature, Sketch};

/// How far from a street's middle its gutter reaches.
const GUTTER: f32 = 0.72;
/// How far a town street paves out toward the buildings along it.
const FRONTAGE: i32 = 4;

/// A building's footprint seen from above: its north-west corner and size.
pub type Rect = ([i32; 2], [i32; 2]);

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
    /// Every feature of `sketch` drawn, on a grid just big enough for them, and the street
    /// paved out to the `paved` buildings standing along it.
    pub fn of(sketch: &Sketch, paved: &[Rect]) -> Ground {
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
        ground.pave_frontages(paved);
        ground
    }

    pub fn get(&self, [x, z]: [usize; 2]) -> Cell {
        self.cells[z * self.size[0] + x]
    }

    /// Paves the ground between a street and the buildings on it, so no grass is left in
    /// front of a house: every bare cell near both.
    fn pave_frontages(&mut self, buildings: &[Rect]) {
        let [w, h] = self.size.map(|s| s as i32);
        let paved = |g: &Ground, x: i32, z: i32| {
            let inside = (0..w).contains(&x) && (0..h).contains(&z);
            inside && g.cells[(z * w + x) as usize].rank() >= Cell::Kerb.rank()
        };
        let mut frontage = Vec::new();
        for z in 0..h {
            for x in 0..w {
                let world = [x + self.origin[0], z + self.origin[1]];
                let bare = self.cells[(z * w + x) as usize] == Cell::Ground;
                let near_building = buildings.iter().any(|b| gap(b, world) <= FRONTAGE);
                let reach = -FRONTAGE..=FRONTAGE;
                let near_street = || {
                    reach
                        .clone()
                        .any(|dz| reach.clone().any(|dx| paved(self, x + dx, z + dz)))
                };
                if bare && near_building && near_street() {
                    frontage.push((z * w + x) as usize);
                }
            }
        }
        for i in frontage {
            self.cells[i] = Cell::Square;
        }
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
                on.then_some(if worn(x, z, 0.82) {
                    Cell::Worn
                } else {
                    Cell::Path
                })
            }
            Feature::Track { .. } => {
                let d = distance(&self.points, at);
                ((d - 1.1).abs() <= 0.55).then_some(Cell::Rut)
            }
            Feature::Street { width, .. } => {
                let (d, along) = nearest(&self.points, at);
                // Along an axis only: at an angle a gutter of slabs steps like a staircase.
                let square_on =
                    along[0].abs().min(along[1].abs()) * 2.5 < along[0].abs().max(along[1].abs());
                match d {
                    // Past half a cell's diagonal, so a gutter at a slight angle runs on unbroken.
                    d if d <= GUTTER && square_on => Some(Cell::Gutter),
                    d if d <= width / 2.0 => Some(if worn(x, z, 0.9) {
                        Cell::Patch
                    } else {
                        Cell::Paving
                    }),
                    d if d <= width / 2.0 + 1.0 => Some(Cell::Kerb),
                    _ => None,
                }
            }
            Feature::Hedge { .. } => (distance(&self.points, at) <= 0.6).then_some(Cell::Hedge),
            Feature::Square { area } => inside(area, at).then_some(if worn(x, z, 0.92) {
                Cell::SquarePatch
            } else {
                Cell::Square
            }),
            Feature::Field { area } => {
                inside(area, at).then(|| strip(area, at, 4, &[1, 2, 3, 4, 7]))
            }
            Feature::Garden { area } => inside(area, at).then(|| strip(area, at, 2, &[2, 3, 4])),
        }
    }
}

/// Strips `width` blocks wide along the longest side of `area`, a furrow between each, each
/// strip's wheat of one age.
fn strip(area: &[[f32; 2]], at: [f32; 2], width: i32, ages: &[u8]) -> Cell {
    let along = longest_side(area);
    let across = (at[0] * -along[1] + at[1] * along[0]).floor() as i32;
    let index = across.div_euclid(width + 1);
    match across.rem_euclid(width + 1) == width {
        true => Cell::Furrow,
        false => Cell::Crop(ages[(noise(index, 7) * ages.len() as f32) as usize % ages.len()]),
    }
}

/// The direction of the longest side of `area`, one block long.
fn longest_side(area: &[[f32; 2]]) -> [f32; 2] {
    let sides = (0..area.len()).map(|i| {
        let [a, b] = [area[i], area[(i + 1) % area.len()]];
        [b[0] - a[0], b[1] - a[1]]
    });
    let side = sides.fold([0.0, 1.0], |best: [f32; 2], s| {
        if s[0].hypot(s[1]) > best[0].hypot(best[1]) {
            s
        } else {
            best
        }
    });
    let length = side[0].hypot(side[1]);
    [side[0] / length, side[1] / length]
}

/// Whether column (`x`, `z`) is worn, in patches of a few blocks: rarer the higher `rarity`.
fn worn(x: i32, z: i32, rarity: f32) -> bool {
    noise(x.div_euclid(3), z.div_euclid(3) + 9000) > rarity && noise(x, z) > 0.3
}

/// How many blocks `at` is from `rect`, along x or z, whichever is further; 0 inside it.
fn gap((corner, size): &Rect, at: [i32; 2]) -> i32 {
    let off = |i: usize| {
        (corner[i] - at[i])
            .max(at[i] - (corner[i] + size[i] - 1))
            .max(0)
    };
    off(0).max(off(1))
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
    nearest(line, at).0
}

/// How far `at` is from the nearest point of `line`, and which way the line runs there.
fn nearest(line: &[[f32; 2]], at: [f32; 2]) -> (f32, [f32; 2]) {
    if line.len() == 1 {
        return (
            ((at[0] - line[0][0]).powi(2) + (at[1] - line[0][1]).powi(2)).sqrt(),
            [1.0, 0.0],
        );
    }
    line.windows(2)
        .map(|pair| {
            let [a, b] = [pair[0], pair[1]];
            let along = [b[0] - a[0], b[1] - a[1]];
            let length = along[0] * along[0] + along[1] * along[1];
            let t =
                (((at[0] - a[0]) * along[0] + (at[1] - a[1]) * along[1]) / length).clamp(0.0, 1.0);
            let near = [a[0] + t * along[0], a[1] + t * along[1]];
            let d = ((at[0] - near[0]).powi(2) + (at[1] - near[1]).powi(2)).sqrt();
            (d, along)
        })
        .fold((f32::MAX, [1.0, 0.0]), |best, here| {
            if here.0 < best.0 { here } else { best }
        })
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
    fn fields_are_striped_along_their_longest_side() {
        let long_east = [[0.0, 0.0], [30.0, 0.0], [30.0, 8.0], [0.0, 8.0]];
        let row = |z: f32| (0..30).map(move |x| strip(&long_east, [x as f32 + 0.5, z], 4, &[1, 7]));
        let first: Vec<Cell> = row(0.5).collect();
        assert!(
            first.iter().all(|c| *c == first[0]),
            "one strip all along a row"
        );
    }

    #[test]
    fn the_gap_to_a_footprint_counts_blocks_outside_it() {
        let house: Rect = ([10, 10], [5, 5]);
        assert_eq!(gap(&house, [12, 12]), 0, "inside");
        assert_eq!(gap(&house, [16, 12]), 2, "two east of its last column");
        assert_eq!(gap(&house, [7, 7]), 3, "off a corner");
    }

    #[test]
    fn a_smoothed_line_keeps_its_ends() {
        let line = smooth(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]]);
        assert_eq!(line[0], [0.0, 0.0], "the start");
        assert_eq!(line[line.len() - 1], [10.0, 10.0], "the end");
        assert!(distance(&line, [10.0, 0.0]) > 1.0, "the corner is cut");
    }
}
