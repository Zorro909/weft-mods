//! Buildings standing along a road or round the heart, each turned to face it: a walk down the
//! front, a building wherever the ground and its neighbours leave room, then on past it.

use structgen_api::{Fit, Plan, Turn};
use weft_sdk::Template;

use super::grid::{Cell, Grid, column};
use super::roads::Road;
use super::shape::{Point, add, inside, scale, walk};

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
pub struct Rules {
    /// Blocks between a road's edge and a building's front, at least and at most.
    pub setback: [i32; 2],
    /// Blocks kept free between two buildings.
    pub gap: i32,
    pub fit: Fit,
    /// What the way from the road to a building's door becomes: a path, or paving.
    pub path: Cell,
}

/// Both sides of `road`.
pub fn sides(road: &Road, reach: f32) -> [Front; 2] {
    let points = walk(&road.line);
    [1.0, -1.0].map(|side| Front {
        points: points
            .iter()
            .map(|&(at, way)| (at, scale([-way[1], way[0]], side)))
            .collect(),
        reach,
        faces: Faces::Road,
    })
}

/// The outside of the closed outline `area`, its way out pointing away from it.
pub fn around(area: &[Point], reach: f32) -> Front {
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

/// The turn that points a building's front, north as saved, back along `out` toward the road.
pub fn facing(out: Point) -> Turn {
    match (out[1].abs() >= out[0].abs(), out[0] > 0.0, out[1] > 0.0) {
        (true, _, true) => Turn::None,
        (true, _, false) => Turn::Back,
        (false, true, _) => Turn::Left,
        (false, false, _) => Turn::Right,
    }
}

/// The north-west corner of a building of `size` whose front, turned by `turn`, has its middle
/// at column `front`.
pub fn corner_for(front: [i32; 2], [sx, _, sz]: [i32; 3], turn: Turn) -> [i32; 2] {
    let [x, z] = front;
    match turn {
        Turn::None => [x - sx / 2, z],
        Turn::Back => [x - sx / 2, z - sz + 1],
        Turn::Right => [x - sx + 1, z - sz / 2],
        Turn::Left => [x, z - sz / 2],
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
/// already there leave it room and a walkable way leads from the road to its door; marked on
/// `grid` with that way when it does.
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
    let doorstep = grid.walkable(at, out, (length - 1).max(0))?;
    let face = column(add(at, scale(out, length as f32)));
    let placed = stand(
        p,
        grid,
        rules,
        template,
        corner_for(face, size, turn),
        turn,
        Some(doorstep),
    )?;
    for k in 0..length {
        let way = column(add(at, scale(out, k as f32)));
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
    fn a_building_turns_its_front_to_the_road() {
        assert_eq!(
            facing([0.0, 1.0]),
            Turn::None,
            "south of the road: front north"
        );
        assert_eq!(facing([0.0, -1.0]), Turn::Back, "north of it: front south");
        assert_eq!(facing([1.0, 0.2]), Turn::Left, "east of it: front west");
        assert_eq!(facing([-1.0, 0.0]), Turn::Right, "west of it: front east");
    }

    #[test]
    fn the_front_lands_on_the_column_asked_for() {
        let size = [7, 9, 5];
        assert_eq!(
            corner_for([10, 20], size, Turn::None),
            [7, 20],
            "front row is z 20"
        );
        assert_eq!(
            corner_for([10, 20], size, Turn::Back),
            [7, 16],
            "back row is z 16"
        );
        assert_eq!(
            corner_for([10, 20], size, Turn::Left),
            [10, 18],
            "front column is x 10"
        );
        assert_eq!(
            corner_for([10, 20], size, Turn::Right),
            [4, 18],
            "front column is x 10"
        );
    }
}
