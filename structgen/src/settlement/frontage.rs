//! Buildings standing along a road or round the heart, each turned to face it: a walk down the
//! front, a building wherever the ground and its neighbours leave room, then on past it.

use structgen_api::{Fit, Plan, Turn};
use weft_sdk::Template;

use super::grid::{Cell, Grid, column};
use super::roads::Road;
use super::shape::{Point, add, inside, scale, walk};

mod door;

pub use door::{corner_for, facing};
use door::{doorstep, outside};

/// What faces a front: a heart's edge, or a road.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Faces {
    Heart,
    Road,
}

/// One side of a road, or the edge of the heart: points a block apart, each with the way out
/// from it toward where buildings stand.
pub struct Front {
    pub points: Vec<(Point, Point)>,
    /// How far the road reaches out from the line, in blocks.
    pub reach: f32,
    pub faces: Faces,
}

/// A building decided on: where it stands and how it is turned.
pub struct Site {
    pub template: &'static Template,
    pub corner: [i32; 3],
    pub turn: Turn,
    pub size: [i32; 3],
}

/// How buildings line up along a front.
#[derive(Clone, Copy)]
pub struct Rules {
    /// Blocks between a road's edge and a building's front, at least and at most.
    pub setback: [i32; 2],
    /// Blocks kept free between two buildings.
    pub gap: i32,
    pub fit: Fit,
    /// What the way from the road to a building's door becomes: a path, or paving.
    pub path: Cell,
}

/// Every front buildings may face: round the heart, starting `start` points round it, then one
/// side of every road, then the other, so homes handed out a front at a time reach every road
/// before a road gets its second side.
pub fn fronts(heart: &[Point], ring: f32, roads: &[Road], width: f32, start: usize) -> Vec<Front> {
    let mut round = around(heart, ring);
    let turn = start % round.points.len().max(1);
    round.points.rotate_left(turn);
    let mut fronts = vec![round];
    for sign in [1.0, -1.0] {
        fronts.extend(roads.iter().map(|road| side(road, width / 2.0, sign)));
    }
    fronts
}

/// One side of `road`: the left going along it for `sign` 1, the right for -1.
fn side(road: &Road, reach: f32, sign: f32) -> Front {
    let points = walk(&road.line).into_iter();
    Front {
        points: points
            .map(|(at, way)| (at, scale([-way[1], way[0]], sign)))
            .collect(),
        reach,
        faces: Faces::Road,
    }
}

/// The outside of the closed outline `area`, its way out pointing away from it.
fn around(area: &[Point], reach: f32) -> Front {
    let mut closed = area.to_vec();
    closed.push(area[0]);
    let points = walk(&closed)
        .into_iter()
        .map(|(at, way)| {
            let out = [-way[1], way[0]];
            let out = if inside(area, add(at, scale(out, 2.0))) {
                scale(out, -1.0)
            } else {
                out
            };
            (at, out)
        })
        .collect();
    Front {
        points,
        reach,
        faces: Faces::Heart,
    }
}

/// The first building `pick` chooses that has room along `front` from point `from` on, and
/// the point past it where the next may start.
pub fn next_site(
    p: &mut Plan,
    grid: &mut Grid,
    rules: &Rules,
    front: &Front,
    from: usize,
    pick: &mut dyn FnMut(&mut Plan, Point) -> &'static Template,
) -> Option<(Site, usize)> {
    let mut i = from;
    while i < front.points.len() {
        let template = pick(p, front.points[i].0);
        if let Some(placed) = site(p, grid, rules, template, front, i) {
            let gap = rules.gap + p.random().between(0, rules.gap);
            let next = i + width_along(&placed) + gap as usize;
            return Some((placed, next));
        }
        i += 2;
    }
    None
}

/// Where `template` would stand at point `index` of `front`, if the ground and the buildings
/// already there leave it room and a walkable way leads from the road to the column outside
/// its door; marked on `grid` with that way when it does.
fn site(
    p: &mut Plan,
    grid: &mut Grid,
    rules: &Rules,
    template: &'static Template,
    front: &Front,
    index: usize,
) -> Option<Site> {
    let (at, out) = front.points[index];
    let turn = facing(out);
    let size = p.size(template, turn)?;
    let setback = p.random().between(rules.setback[0], rules.setback[1]);
    let length = (front.reach + setback as f32 + 0.5) as i32;
    let face = column(add(at, scale(out, length as f32)));
    let door = outside(face, turn);
    let doorstep = doorstep(grid, at, out, length, door)?;
    let placed = stand(
        p,
        grid,
        rules,
        template,
        corner_for(face, size, turn),
        turn,
        Some(doorstep),
    )?;
    let way = (0..length).map(|k| column(add(at, scale(out, k as f32))));
    for way in way.chain([door]) {
        if grid.get(way) == Cell::Free {
            grid.put(way, rules.path);
        }
    }
    Some(placed)
}

/// `template` with its north-west corner at `corner`, if it stands on free columns, no closer
/// than the gap to another building, on ground that meets the rules' fit, and, given a
/// `doorstep` height, with its floor within a block of it.
pub fn stand(
    p: &mut Plan,
    grid: &mut Grid,
    rules: &Rules,
    template: &'static Template,
    corner: [i32; 2],
    turn: Turn,
    doorstep: Option<i32>,
) -> Option<Site> {
    let size = p.size(template, turn)?;
    let last = [corner[0] + size[0] - 1, corner[1] + size[2] - 1];
    let g = rules.gap;
    let room = grid.all(corner, last, |c| {
        matches!(c, Cell::Free | Cell::Green | Cell::Square)
    }) && grid.all(
        [corner[0] - g, corner[1] - g],
        [last[0] + g, last[1] + g],
        |c| c != Cell::Building,
    );
    if !room {
        return None;
    }
    let at = p.fit(template, corner, turn, &rules.fit)?;
    if doorstep.is_some_and(|y| (y - at[1]).abs() > 1) {
        return None;
    }
    for z in corner[1]..=last[1] {
        for x in corner[0]..=last[0] {
            grid.put([x, z], Cell::Building);
        }
    }
    Some(Site {
        template,
        corner: at,
        turn,
        size,
    })
}

/// `template` round the column `middle` is in; see `stand`.
pub fn stand_round(
    p: &mut Plan,
    grid: &mut Grid,
    rules: &Rules,
    template: &'static Template,
    middle: Point,
    turn: Turn,
) -> Option<Site> {
    let [sx, _, sz] = p.size(template, turn)?;
    let [x, z] = column(middle);
    stand(
        p,
        grid,
        rules,
        template,
        [x - sx / 2, z - sz / 2],
        turn,
        None,
    )
}

/// How many points along a front a building takes: its width along the road.
fn width_along(site: &Site) -> usize {
    match site.turn {
        Turn::None | Turn::Back => site.size[0] as usize,
        Turn::Left | Turn::Right => site.size[2] as usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_road_gets_a_side_before_any_gets_its_second() {
        let heart = [[-4.0, -4.0], [4.0, -4.0], [4.0, 4.0], [-4.0, 4.0]];
        let east = Road {
            line: vec![[5.0, 0.0], [20.0, 0.0]],
        };
        let south = Road {
            line: vec![[0.0, 5.0], [0.0, 20.0]],
        };
        let fronts = fronts(&heart, 0.0, &[east, south], 3.0, 5);
        let first = |f: &Front| f.points[0];
        assert_eq!(fronts.len(), 5);
        assert_eq!(fronts[0].faces, Faces::Heart);
        assert_eq!(
            first(&fronts[1]),
            ([5.0, 0.0], [0.0, 1.0]),
            "east road, south side"
        );
        assert_eq!(first(&fronts[2]).1, [-1.0, 0.0], "south road, west side");
        assert_eq!(first(&fronts[3]).1, [0.0, -1.0], "east road, north side");
        assert_ne!(
            fronts[0].points[0].0, heart[0],
            "the heart's walk starts further round"
        );
    }
}
