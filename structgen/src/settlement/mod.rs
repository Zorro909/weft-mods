//! Settlements laid out afresh from the seed and the lie of the land. Each has a heart, a
//! village green or a town's market square, shaped at random and stretched along a main road
//! that runs out of it both ways; branches leave that road, and every road winds along the
//! ground (`roads.rs`). Buildings stand along both sides of the roads and round the heart, each
//! turned to face it (`frontage.rs`); gardens, a few small fields and trees fill in. What a kind
//! of settlement holds is data, `Settlement`; this is the planner they share.

mod draw;
mod frontage;
mod grid;
mod planting;
mod roads;
mod shape;

use std::f32::consts::TAU;

pub use frontage::Faces;
use frontage::{Front, Rules, Site, around, facing, next_site, sides, stand_round};
use grid::{Cell, Grid};
use roads::{Road, unit};
use shape::{Point, add, heading, length_of, scale, smooth, sub};
use structgen_api::{Biomes, Fit, Foundation, Materials, Plan, PlanError, Random};
use weft_sdk::*;

use crate::styles;
use crate::worldgen::{OPEN, SEA};

static OPEN_LAND: Biomes = Biomes::named(OPEN);

/// A village's lane is this wide; a town's street, kerbs included, `STREET`.
const LANE: f32 = 3.0;
const STREET: f32 = 7.0;

/// A village of lanes round a green, or a town of streets round a market square.
#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Village,
    Town,
}

/// The side of a region, in blocks: every kind of settlement shares them out, so no two stand
/// in one region.
pub const REGION: u32 = 512;

/// One kind of settlement.
pub struct Settlement {
    /// The regions it takes: those whose first random number, from 0 to 1, falls in this range.
    /// The kinds' ranges don't overlap.
    pub share: [f32; 2],
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
    /// Each takes the first spot with room along a front of its kind: the church facing the
    /// green, the smithy on a road.
    pub landmarks: &'static [(&'static Template, Faces)],
    /// Homes along the fronts, this many at most: `houses` near the heart, `farms` further out.
    pub homes: usize,
    pub houses: &'static [&'static Template],
    pub farms: &'static [&'static Template],
    /// Past one end of the main road.
    pub mill: Option<&'static Template>,
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
    let roll = unit(p.random());
    if !(s.share[0]..s.share[1]).contains(&roll) {
        return Ok(());
    }
    let reach = (s.steps[0] as f32 * roads::STEP + s.heart[0]) as i32 + 20;
    let middle = p.spot(reach);
    let mut grid = Grid::new(p, middle, reach);
    let Some(sites) = lay_out(p, &mut grid, s, middle) else {
        return Ok(());
    };
    let walls = walls(p, s);
    for site in &sites {
        let roof = styles::blocks(one_of(p, s.roofs).unwrap_or(""));
        p.structure(site.template, site.corner, site.turn)?
            .materials(&styles::add(walls.clone(), roof))
            .foundation(Foundation::Ground)
            .clear_above();
    }
    draw::draw(p, &grid)
}

/// Where everything stands, marked on `grid`: `None` when the ground leaves too little room
/// for a settlement worth the name.
fn lay_out(p: &mut Plan, grid: &mut Grid, s: &Settlement, middle: [i32; 2]) -> Option<Vec<Site>> {
    let angle = unit(p.random()) * TAU;
    let m = [middle[0] as f32 + 0.5, middle[1] as f32 + 0.5];
    let heart = roads::heart(p.random(), m, angle, s.heart);
    let roads = lay_roads(p.random(), grid, s, &heart, m, angle);
    // A village's green has a lane round it; a town's square is paved to its houses' doors.
    let (width, ring, path) = match s.kind {
        Kind::Village => (LANE, LANE / 2.0, Cell::Lane),
        Kind::Town => (STREET, 0.0, Cell::Square),
    };
    let rules = Rules {
        setback: s.setback,
        gap: s.gap,
        fit: Fit::new().biome(&OPEN_LAND).max_slope(4).min_height(SEA),
        path,
    };
    let mut sites = centre(p, grid, s, m, angle);
    let mut fronts = vec![around(&heart, ring)];
    fronts.extend(roads.iter().flat_map(|road| sides(road, width / 2.0)));
    let farmland = reach_of(&roads, m) * 0.6;
    let far = |at: Point| length_of(sub(at, m)) > farmland;
    let homes = line_up(p, grid, s, &rules, &fronts, &far, &mut sites);
    if homes < s.homes / 2 {
        return None;
    }
    sites.extend(
        s.mill
            .and_then(|mill| outskirts(p, grid, &rules, mill, &roads[..2])),
    );
    planting::fields(grid, &roads, s.fields);
    let reach = s.heart[0] as i32 + 18;
    sites.extend(planting::trees(p, grid, s.trees, &rules, m, reach));
    if s.kind == Kind::Town {
        grid.pave_frontages();
    }
    Some(sites)
}

/// The heart and the roads out of it, marked on `grid`: the main road both ways, then the
/// branches, each leaving it a third of the way along or further.
fn lay_roads(
    random: &mut Random,
    grid: &mut Grid,
    s: &Settlement,
    heart: &[Point],
    m: Point,
    angle: f32,
) -> Vec<Road> {
    let (width, road, middle) = match s.kind {
        Kind::Village => (LANE, Cell::Lane, Cell::Green),
        Kind::Town => (STREET, Cell::Street, Cell::Square),
    };
    grid.put_area(heart, middle);
    let mut roads: Vec<Road> = roads::main_road(grid, random, heart, m, angle, s.steps[0]).into();
    for main in &roads {
        grid.put_road(&main.line, width, road);
    }
    if s.kind == Kind::Village {
        let mut ring = heart.to_vec();
        ring.push(heart[0]);
        grid.put_road(&smooth(&ring), LANE, Cell::Lane);
    }
    for _ in 0..random.between(s.branches[0], s.branches[1]) {
        let from = &roads[random.below(2) as usize];
        let n = from.line.len() as i32;
        let index = random.between(n / 3, (n - 3).max(n / 3)) as usize;
        let branch = roads::branch(grid, random, from, index, s.steps[1]);
        grid.put_road(&branch.line, width, road);
        roads.push(branch);
    }
    roads
}

/// The heart's own buildings, in it along the main road: the first in the middle.
fn centre(p: &mut Plan, grid: &mut Grid, s: &Settlement, m: Point, angle: f32) -> Vec<Site> {
    let rules = Rules {
        setback: [0, 0],
        gap: 2,
        fit: Fit::new().max_slope(3).min_height(SEA),
        path: Cell::Free,
    };
    let along = heading(angle);
    let mut sites = Vec::new();
    for (i, &template) in s.centre.iter().enumerate() {
        let turn = facing(scale(along, if i % 2 == 0 { 1.0 } else { -1.0 }));
        let placed = [0.0, 10.0, -10.0, 16.0, -16.0]
            .into_iter()
            .find_map(|offset| {
                stand_round(
                    p,
                    grid,
                    &rules,
                    template,
                    add(m, scale(along, offset)),
                    turn,
                )
            });
        sites.extend(placed);
    }
    sites
}

/// Puts buildings along the fronts, heart first: each landmark at the first spot with room
/// along a front of its kind, then homes, one to each front in turn, until there are
/// `s.homes` or no front has room left. Gives back how many homes stand.
fn line_up(
    p: &mut Plan,
    grid: &mut Grid,
    s: &Settlement,
    rules: &Rules,
    fronts: &[Front],
    far: &dyn Fn(Point) -> bool,
    sites: &mut Vec<Site>,
) -> usize {
    let mut next = vec![0; fronts.len()];
    for &(template, faces) in s.landmarks {
        let mut ours = fronts
            .iter()
            .zip(&mut next)
            .filter(|(front, _)| front.faces == faces);
        let found = ours.find_map(|(front, from)| {
            let placed = next_site(p, grid, rules, front, *from, &mut |_, _| template)?;
            Some((placed, from))
        });
        if let Some(((placed, after), from)) = found {
            *from = after;
            sites.push(placed);
        }
    }
    let mut pick = |p: &mut Plan, at: Point| match far(at) && !s.farms.is_empty() {
        true => one_of(p, s.farms).expect("farms are there"),
        false => one_of(p, s.houses).expect("a settlement has houses"),
    };
    let mut homes = 0;
    while homes < s.homes
        && next
            .iter()
            .zip(fronts)
            .any(|(&i, front)| i < front.points.len())
    {
        for (front, from) in fronts.iter().zip(&mut next) {
            if homes == s.homes {
                break;
            }
            let Some((placed, after)) = next_site(p, grid, rules, front, *from, &mut pick) else {
                *from = front.points.len();
                continue;
            };
            if s.kind == Kind::Village {
                planting::garden(grid, &placed);
            }
            (*from, homes) = (after, homes + 1);
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

/// `template` past one end or the other of the main road, its front toward it.
fn outskirts(
    p: &mut Plan,
    grid: &mut Grid,
    rules: &Rules,
    template: &'static Template,
    main: &[Road],
) -> Option<Site> {
    main.iter().filter_map(Road::end).find_map(|(end, way)| {
        let turn = facing(way);
        let [sx, _, sz] = p.size(template, turn)?;
        let middle = add(end, scale(way, 6.0 + sx.max(sz) as f32 / 2.0));
        stand_round(p, grid, rules, template, middle, turn)
    })
}

/// One of `items` at random, `None` when there are none.
fn one_of<T: Copy>(p: &mut Plan, items: &[T]) -> Option<T> {
    match items.len() {
        0 => None,
        n => Some(items[p.random().below(n as u64) as usize]),
    }
}

/// The frame and infill all of a settlement's buildings share, and in a town paving round each.
fn walls(p: &mut Plan, s: &Settlement) -> Materials {
    let frame = styles::blocks(one_of(p, s.frames).unwrap_or(""));
    let infill = styles::blocks(one_of(p, s.infills).unwrap_or(""));
    let walls = styles::add(styles::add(Materials::new(), frame), infill);
    match s.kind {
        Kind::Village => walls,
        Kind::Town => styles::add(walls, styles::blocks("cobble_ground")),
    }
}
