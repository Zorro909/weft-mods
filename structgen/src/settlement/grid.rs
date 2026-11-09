//! A settlement's ground seen from above, a cell per column: what each column is for, so roads,
//! buildings and fields keep out of each other, and how high its ground is.

use structgen_api::{Biomes, Plan};

use super::shape::{Point, add, distance, inside, scale};
use crate::worldgen::SEA;

/// What one column is for.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Free,
    /// A village green.
    Green,
    /// Wheat of this age on farmland, or bare earth between the strips.
    Crop(u8),
    Furrow,
    Lane,
    Kerb,
    Street,
    /// A town's market square, or paving up to a building.
    Square,
    Building,
    /// Water, no ground, land of another kind (a forest), or outside the region: nothing goes
    /// here.
    Blocked,
}

impl Cell {
    /// Where two things want a column, the one that ranks higher has it.
    fn rank(self) -> u8 {
        match self {
            Cell::Free => 0,
            Cell::Green | Cell::Crop(_) | Cell::Furrow => 1,
            Cell::Lane => 2,
            Cell::Kerb => 3,
            Cell::Street => 4,
            Cell::Square => 5,
            Cell::Building => 6,
            Cell::Blocked => 7,
        }
    }

    pub fn is_road(self) -> bool {
        matches!(self, Cell::Lane | Cell::Kerb | Cell::Street | Cell::Square)
    }
}

pub struct Grid {
    /// The north-west column.
    origin: [i32; 2],
    side: i32,
    cells: Vec<Cell>,
    heights: Vec<i32>,
}

impl Grid {
    /// The columns within `reach` of `middle`, read from the ground; only `land` is free.
    pub fn new(p: &Plan, middle: [i32; 2], reach: i32, land: &Biomes) -> Grid {
        let (origin, side) = ([middle[0] - reach, middle[1] - reach], 2 * reach + 1);
        let region = p.region();
        let mut grid = Grid {
            origin,
            side,
            cells: Vec::with_capacity((side * side) as usize),
            heights: Vec::with_capacity((side * side) as usize),
        };
        for z in origin[1]..origin[1] + side {
            for x in origin[0]..origin[0] + side {
                let ground = p
                    .ground(x, z)
                    .filter(|g| g.height >= SEA && region.contains(x, z) && land.contains(g.biome));
                let usable = ground.is_some();
                grid.cells
                    .push(if usable { Cell::Free } else { Cell::Blocked });
                grid.heights.push(ground.map_or(SEA, |g| g.height));
            }
        }
        grid
    }

    fn index(&self, [x, z]: [i32; 2]) -> Option<usize> {
        let [dx, dz] = [x - self.origin[0], z - self.origin[1]];
        let within = (0..self.side).contains(&dx) && (0..self.side).contains(&dz);
        within.then_some((dz * self.side + dx) as usize)
    }

    pub fn get(&self, at: [i32; 2]) -> Cell {
        self.index(at).map_or(Cell::Blocked, |i| self.cells[i])
    }

    /// The height of the ground's top block.
    pub fn height(&self, at: [i32; 2]) -> Option<i32> {
        let i = self.index(at)?;
        (self.cells[i] != Cell::Blocked).then_some(self.heights[i])
    }

    /// The height at the end of a walk of `blocks` blocks from `from` along `way`, if every
    /// column on the way has ground and no building, and none is more than a block above or
    /// below the last.
    pub fn walkable(&self, from: Point, way: Point, blocks: i32) -> Option<i32> {
        let ground = |at: Point| {
            let at = column(at);
            self.height(at).filter(|_| self.get(at) != Cell::Building)
        };
        let mut last = ground(from)?;
        for k in 1..=blocks {
            let next = ground(add(from, scale(way, k as f32)))?;
            if (next - last).abs() > 1 {
                return None;
            }
            last = next;
        }
        Some(last)
    }

    /// Puts `cell` at `at` unless what is there ranks higher.
    pub fn put(&mut self, at: [i32; 2], cell: Cell) {
        if let Some(i) = self.index(at)
            && cell.rank() > self.cells[i].rank()
        {
            self.cells[i] = cell;
        }
    }

    /// Whether every cell from `from` to `to` (both included) passes `test`.
    pub fn all(&self, from: [i32; 2], to: [i32; 2], test: impl Fn(Cell) -> bool) -> bool {
        (from[1]..=to[1]).all(|z| (from[0]..=to[0]).all(|x| test(self.get([x, z]))))
    }

    /// Every column with what it is for.
    pub fn columns(&self) -> impl Iterator<Item = ([i32; 2], Cell)> + '_ {
        (0..self.cells.len() as i32).map(|i| {
            let at = [
                self.origin[0] + i % self.side,
                self.origin[1] + i / self.side,
            ];
            (at, self.cells[i as usize])
        })
    }

    /// Puts `cell` on every column whose middle is inside `area`.
    pub fn put_area(&mut self, area: &[Point], cell: Cell) {
        let (from, to) = bounds(area, 0.0);
        for z in from[1]..=to[1] {
            for x in from[0]..=to[0] {
                if inside(area, middle([x, z])) {
                    self.put([x, z], cell);
                }
            }
        }
    }

    /// A road `width` wide along `line`: a lane, or a street with kerbs along both sides.
    pub fn put_road(&mut self, line: &[Point], width: f32, cell: Cell) {
        let half = width / 2.0;
        let (from, to) = bounds(line, half + 1.0);
        for z in from[1]..=to[1] {
            for x in from[0]..=to[0] {
                let d = distance(line, middle([x, z]));
                match cell {
                    Cell::Street if d <= half - 1.0 => self.put([x, z], Cell::Street),
                    Cell::Street if d <= half => self.put([x, z], Cell::Kerb),
                    _ if d <= half => self.put([x, z], cell),
                    _ => {}
                }
            }
        }
    }

    /// Paves the bare ground between a town's streets and the buildings on them, so no grass is
    /// left in front of a house: every free column next to a building and near a road.
    pub fn pave_frontages(&mut self) {
        let near = |grid: &Grid, at: [i32; 2], reach: i32, test: fn(Cell) -> bool| {
            let from = [at[0] - reach, at[1] - reach];
            !grid.all(from, [at[0] + reach, at[1] + reach], |c| !test(c))
        };
        let paved: Vec<[i32; 2]> = (self.columns())
            .filter(|&(at, cell)| {
                cell == Cell::Free
                    && near(self, at, 2, |c| c == Cell::Building)
                    && near(self, at, 3, Cell::is_road)
            })
            .map(|(at, _)| at)
            .collect();
        for at in paved {
            self.put(at, Cell::Square);
        }
    }
}

/// The column a point is in.
pub fn column(at: Point) -> [i32; 2] {
    [at[0].floor() as i32, at[1].floor() as i32]
}

/// The middle of column `at`.
pub fn middle([x, z]: [i32; 2]) -> Point {
    [x as f32 + 0.5, z as f32 + 0.5]
}

/// The columns from the lowest to the highest corner of `points`, `margin` wider all round.
fn bounds(points: &[Point], margin: f32) -> ([i32; 2], [i32; 2]) {
    let low = |i: usize| points.iter().map(|p| p[i]).fold(f32::MAX, f32::min) - margin;
    let high = |i: usize| points.iter().map(|p| p[i]).fold(f32::MIN, f32::max) + margin;
    (
        [low(0).floor() as i32, low(1).floor() as i32],
        [high(0).ceil() as i32, high(1).ceil() as i32],
    )
}

/// A row of columns from x 0 east, at these heights, `None` for water; for tests.
#[cfg(test)]
pub fn row(heights: &[Option<i32>]) -> Grid {
    let side = heights.len() as i32;
    let mut grid = Grid {
        origin: [0, 0],
        side,
        cells: vec![Cell::Blocked; (side * side) as usize],
        heights: vec![SEA; (side * side) as usize],
    };
    for (x, height) in heights.iter().enumerate() {
        if let Some(height) = height {
            (grid.cells[x], grid.heights[x]) = (Cell::Free, *height);
        }
    }
    grid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_walk_climbs_a_block_at_a_time_and_stops_at_water_or_a_ledge() {
        let east = [1.0, 0.0];
        let gentle = row(&[Some(64), Some(65), Some(66), Some(65)]);
        assert_eq!(gentle.walkable([0.5, 0.5], east, 3), Some(65));
        let ledge = row(&[Some(64), Some(64), Some(66), Some(66)]);
        assert_eq!(
            ledge.walkable([0.5, 0.5], east, 3),
            None,
            "two blocks up at once"
        );
        let stream = row(&[Some(64), None, Some(64), Some(64)]);
        assert_eq!(
            stream.walkable([0.5, 0.5], east, 3),
            None,
            "water between dry ends"
        );
        let mut house = row(&[Some(64); 4]);
        house.put([2, 0], Cell::Building);
        assert_eq!(
            house.walkable([0.5, 0.5], east, 3),
            None,
            "no way through a house"
        );
    }
}
