//! Settlements laid out afresh from the seed and the lie of the land. Each has a heart, a
//! village green or a town's market square, shaped at random and stretched along a main road
//! that runs out of it both ways; branches leave that road, and every road winds along the
//! ground (`roads.rs`). Buildings stand along both sides of the roads and round the heart, each
//! turned to face it (`frontage.rs`); gardens, a few small fields and trees fill in. What a kind
//! of settlement holds is data, `Settlement`; this is the planner they share.

mod draw;
mod frontage;
mod grid;
mod roads;
mod shape;

use std::f32::consts::TAU;

pub use frontage::Faces;
use frontage::{Front, Rules, Site, around, facing, sides, site, stand, width_along};
use grid::{Cell, Grid};
use roads::{Road, column, unit};
use shape::{Point, add, heading, length_of, scale, smooth, sub};
use structgen_api::{Biomes, Fit, Foundation, Materials, Plan, PlanError, Turn};
use weft_sdk::*;

use crate::styles;
use crate::worldgen::{OPEN, SEA};

static OPEN_LAND: Biomes = Biomes::named(OPEN);

/// A village's lane is this wide; a town's street, kerbs included, `STREET`.
const LANE: f32 = 3.0;
const STREET: f32 = 7.0;
/// Blocks a road goes in one step: `roads::STEP`, again for the planner's sums.
const STEP: i32 = 7;

/// A village of lanes round a green, or a town of streets round a market square.
#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Village,
    Town,
}

/// One kind of settlement.
pub struct Settlement {
    /// One per region of this many blocks square, at most.
    pub region: u32,
    pub chance: f32,
    pub kind: Kind,
    /// Steps of the main road out of the heart, each way, and of a branch.
    pub steps: [usize; 2],
    /// How many branches leave the main road, at least and at most.
    pub branches: [i32; 2],
    /// How far the heart reaches along the main road and across it, in blocks.
    pub heart: [f32; 2],
    /// Blocks between a road and a building's front, at least and at most, and between two
    /// buildings.
    pub setback: [i32; 2],
    pub gap: i32,
    /// In the heart: a village's chapel and well, a town's fountain.
    pub centre: &'static [&'static Template],
    /// Each takes the first spot along a front of its kind: the town hall facing the square,
    /// the tavern on a road.
    pub landmarks: &'static [(&'static Template, Faces)],
    /// Homes along the fronts, this many at most: `houses` near the heart, `farms` further out.
    pub homes: usize,
    pub houses: &'static [&'static Template],
    pub farms: &'static [&'static Template],
    /// Past the end of the main road: a mill.
    pub outskirts: &'static [&'static Template],
    /// Small strip fields at the ends of the roads.
    pub fields: usize,
    pub trees: &'static [&'static Template],
    /// The roofs its buildings get, one picked for each.
    pub roofs: &'static [&'static str],
    /// The frames and infills it is built with, one of each picked for the whole settlement.
    pub frames: &'static [&'static str],
    pub infills: &'static [&'static str],
}

/// Lays out one settlement in the region, if the seed and the ground there want one.
pub fn plan(p: &mut Plan, s: &Settlement) -> Result<(), PlanError> {
    if !p.random().chance(s.chance) {
        return Ok(());
    }
    let reach = s.steps[0] as i32 * STEP + s.heart[0] as i32 + 20;
    let middle = p.spot(reach);
    let mut grid = Grid::new(p, middle, reach);
    let Some(sites) = lay_out(p, &mut grid, s, middle) else {
        return Ok(());
    };
    let look = Look::pick(p, s);
    for site in &sites {
        let materials = look.for_building(p);
        p.structure(site.template, site.corner, site.turn)?
            .materials(&materials)
            .foundation(Foundation::Ground)
            .clear_above();
    }
    draw::draw(p, &grid)
}

/// Where everything stands, marked on `grid`: `None` when the ground leaves too little room
/// for a settlement worth the name.
fn lay_out(p: &mut Plan, grid: &mut Grid, s: &Settlement, middle: [i32; 2]) -> Option<Vec<Site>> {
    let random = p.random();
    let angle = unit(random) * TAU;
    let m = [middle[0] as f32 + 0.5, middle[1] as f32 + 0.5];
    let heart = roads::heart(random, m, angle, s.heart);
    let (width, road_cell, heart_cell) = match s.kind {
        Kind::Village => (LANE, Cell::Lane, Cell::Green),
        Kind::Town => (STREET, Cell::Street, Cell::Square),
    };
    grid.put_area(&heart, heart_cell);
    let mut roads: Vec<Road> = roads::main_road(grid, random, &heart, m, angle, s.steps[0]).into();
    roads::put(grid, &roads, width, road_cell);
    if s.kind == Kind::Village {
        let mut ring = heart.clone();
        ring.push(heart[0]);
        grid.put_road(&smooth(&ring), LANE, Cell::Lane);
    }
    for _ in 0..random.between(s.branches[0], s.branches[1]) {
        let from = &roads[random.below(2) as usize];
        let index = random.between(2, from.line.len() as i32 - 3).max(0) as usize;
        let branch = roads::branch(grid, random, from, index, s.steps[1]);
        roads::put(grid, std::slice::from_ref(&branch), width, road_cell);
        roads.push(branch);
    }

    let rules = Rules {
        setback: s.setback,
        gap: s.gap,
        fit: Fit::new().biome(&OPEN_LAND).max_slope(4).min_height(SEA),
    };
    let mut sites = centre(p, grid, s, m, angle);
    let heart_reach = if s.kind == Kind::Village {
        LANE / 2.0
    } else {
        0.0
    };
    let mut fronts = vec![around(&heart, heart_reach)];
    fronts.extend(roads.iter().flat_map(|road| sides(road, width / 2.0)));
    let homes = line_up(
        p,
        grid,
        s,
        &rules,
        &fronts,
        m,
        reach_of(&roads, m),
        &mut sites,
    );
    if homes < s.homes / 2 {
        return None;
    }
    sites.extend(outskirts(p, grid, s, &rules, &roads[..2]));
    fields(grid, &roads, s.fields);
    sites.extend(trees(p, grid, s, &rules, m));
    if s.kind == Kind::Town {
        grid.pave_frontages();
    }
    Some(sites)
}

/// The heart's own buildings, in it along the main road: the first in the middle.
fn centre(p: &mut Plan, grid: &mut Grid, s: &Settlement, m: Point, angle: f32) -> Vec<Site> {
    let rules = Rules {
        setback: [0, 0],
        gap: 2,
        fit: Fit::new().max_slope(3).min_height(SEA),
    };
    let along = heading(angle);
    let mut sites = Vec::new();
    for (i, &template) in s.centre.iter().enumerate() {
        let turn = facing(scale(along, if i % 2 == 0 { 1.0 } else { -1.0 }));
        let placed = [0.0, 10.0, -10.0, 16.0, -16.0]
            .into_iter()
            .find_map(|offset| {
                let size = p.size(template, turn)?;
                let [x, z] = column(add(m, scale(along, offset)));
                stand(
                    p,
                    grid,
                    &rules,
                    template,
                    [x - size[0] / 2, z - size[2] / 2],
                    turn,
                )
            });
        sites.extend(placed);
    }
    sites
}

/// Walks every front, heart first, putting a building wherever there is room: each landmark at
/// the first spot along a front of its kind, then homes until there are `s.homes`. Gives back
/// how many homes stand.
#[allow(clippy::too_many_arguments)]
fn line_up(
    p: &mut Plan,
    grid: &mut Grid,
    s: &Settlement,
    rules: &Rules,
    fronts: &[Front],
    m: Point,
    reach: f32,
    sites: &mut Vec<Site>,
) -> usize {
    let mut landmarks = s.landmarks.to_vec();
    let mut homes = 0;
    for front in fronts {
        let mut i = 0;
        while i < front.points.len() && (homes < s.homes || !landmarks.is_empty()) {
            let wanted = landmarks
                .iter()
                .position(|&(_, faces)| faces == front.faces);
            let far = length_of(sub(front.points[i].0, m)) > reach * 0.6;
            let template = match wanted {
                Some(k) => landmarks[k].0,
                None if homes < s.homes => one_of(
                    p,
                    if far && !s.farms.is_empty() {
                        s.farms
                    } else {
                        s.houses
                    },
                ),
                None => break,
            };
            let Some(placed) = site(p, grid, rules, template, front, i) else {
                i += 2;
                continue;
            };
            match wanted {
                Some(k) => drop(landmarks.remove(k)),
                None => homes += 1,
            }
            if s.kind == Kind::Village {
                garden(grid, &placed);
            }
            i += width_along(&placed) + s.gap as usize + p.random().between(0, s.gap) as usize;
            sites.push(placed);
        }
    }
    homes
}

/// How far the furthest road reaches from the heart's middle.
fn reach_of(roads: &[Road], m: Point) -> f32 {
    let points = roads.iter().flat_map(|road| road.line.iter());
    points.map(|&at| length_of(sub(at, m))).fold(0.0, f32::max)
}

/// The outskirts' buildings, each past the end of a road, its front toward it.
fn outskirts(
    p: &mut Plan,
    grid: &mut Grid,
    s: &Settlement,
    rules: &Rules,
    roads: &[Road],
) -> Vec<Site> {
    let mut sites = Vec::new();
    for (&template, road) in s.outskirts.iter().zip(roads) {
        let Some((end, way)) = end_of(road) else {
            continue;
        };
        let turn = facing(way);
        let Some(size) = p.size(template, turn) else {
            continue;
        };
        let [x, z] = column(add(
            end,
            scale(way, 6.0 + size[0].max(size[2]) as f32 / 2.0),
        ));
        sites.extend(stand(
            p,
            grid,
            rules,
            template,
            [x - size[0] / 2, z - size[2] / 2],
            turn,
        ));
    }
    sites
}

/// Up to `count` small strip fields, each just past the end of a road, the branches' first.
fn fields(grid: &mut Grid, roads: &[Road], count: usize) {
    let ends = roads.iter().rev().filter_map(end_of);
    let mut placed = 0;
    for (end, way) in ends {
        if placed == count {
            break;
        }
        let along_x = way[0].abs() > way[1].abs();
        let [w, d] = if along_x { [14, 9] } else { [9, 14] };
        let [x, z] = column(add(end, scale(way, 4.0 + 7.0)));
        let corner = [x - w / 2, z - d / 2];
        let last = [corner[0] + w - 1, corner[1] + d - 1];
        if !grid.all(corner, last, |c| c == Cell::Free) {
            continue;
        }
        for cz in corner[1]..=last[1] {
            for cx in corner[0]..=last[0] {
                let line = if along_x {
                    cz - corner[1]
                } else {
                    cx - corner[0]
                };
                grid.put([cx, cz], strip(line, 5, &[1, 2, 3, 4, 7]));
            }
        }
        placed += 1;
    }
}

/// A kitchen garden behind a house: a few rows of young wheat, if the ground there is free.
fn garden(grid: &mut Grid, site: &Site) {
    let [x, _, z] = site.corner;
    let [sx, _, sz] = site.size;
    let (corner, last) = match site.turn {
        Turn::None => ([x + 1, z + sz + 1], [x + sx - 2, z + sz + 4]),
        Turn::Back => ([x + 1, z - 5], [x + sx - 2, z - 2]),
        Turn::Left => ([x + sx + 1, z + 1], [x + sx + 4, z + sz - 2]),
        Turn::Right => ([x - 5, z + 1], [x - 2, z + sz - 2]),
    };
    if !grid.all(corner, last, |c| c == Cell::Free) {
        return;
    }
    for gz in corner[1]..=last[1] {
        for gx in corner[0]..=last[0] {
            grid.put([gx, gz], strip(gx + gz, 2, &[2, 3, 4]));
        }
    }
}

/// Line `line` of a field: a furrow every `width + 1` lines, wheat of an age from `ages`
/// otherwise, the same all along a strip.
fn strip(line: i32, width: i32, ages: &[u8]) -> Cell {
    let index = line.div_euclid(width + 1);
    match line.rem_euclid(width + 1) == width {
        true => Cell::Furrow,
        false => {
            Cell::Crop(ages[(shape::noise(index, 7) * ages.len() as f32) as usize % ages.len()])
        }
    }
}

/// Trees on free ground near the heart: up to one of each of `s.trees`, a few tries each.
fn trees(p: &mut Plan, grid: &mut Grid, s: &Settlement, rules: &Rules, m: Point) -> Vec<Site> {
    let mut sites = Vec::new();
    for &template in s.trees {
        for _ in 0..8 {
            let reach = s.heart[0] as i32 + 18;
            let x = m[0] as i32 + p.random().between(-reach, reach);
            let z = m[1] as i32 + p.random().between(-reach, reach);
            if let Some(tree) = stand(p, grid, rules, template, [x, z], Turn::None) {
                sites.push(tree);
                break;
            }
        }
    }
    sites
}

/// A road's last point and the way it runs there.
fn end_of(road: &Road) -> Option<(Point, Point)> {
    let [.., before, end] = road.line[..] else {
        return None;
    };
    let way = sub(end, before);
    let length = length_of(way);
    (length > 0.0).then(|| (end, scale(way, 1.0 / length)))
}

fn one_of(p: &mut Plan, templates: &'static [&'static Template]) -> &'static Template {
    templates[p.random().below(templates.len() as u64) as usize]
}

/// What a settlement is built of: one frame and infill for all of it, roofs per building, and
/// in a town paving round every building.
struct Look {
    walls: Materials,
    roofs: &'static [&'static str],
}

impl Look {
    fn pick(p: &mut Plan, s: &Settlement) -> Look {
        let frame = styles::blocks(pick_name(p, s.frames));
        let infill = styles::blocks(pick_name(p, s.infills));
        let mut walls = styles::add(styles::add(Materials::new(), frame), infill);
        if s.kind == Kind::Town {
            walls = styles::add(walls, styles::blocks("cobble_ground"));
        }
        Look {
            walls,
            roofs: s.roofs,
        }
    }

    fn for_building(&self, p: &mut Plan) -> Materials {
        styles::add(self.walls.clone(), styles::blocks(pick_name(p, self.roofs)))
    }
}

/// One of `names` at random, `""` (the defaults) when there are none.
fn pick_name(p: &mut Plan, names: &[&'static str]) -> &'static str {
    match names.len() {
        0 => "",
        n => names[p.random().below(n as u64) as usize],
    }
}
