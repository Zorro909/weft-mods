//! Settlements laid out afresh from the seed and the lie of the land. Each has a heart, a
//! village green or a town's market square, shaped at random and stretched along a main road
//! that runs out of it both ways; branches leave that road, and every road winds along the
//! ground (`roads.rs`). Buildings stand along both sides of the roads and round the heart, each
//! turned to face it (`frontage.rs`); gardens, a few small fields and trees fill in. What a kind
//! of settlement holds is data, `Settlement`; this is the planner they share.

mod buildings;
mod draw;
mod frontage;
mod grid;
mod planting;
mod roads;
mod shape;

use std::f32::consts::TAU;

pub use frontage::Faces;
use frontage::{Rules, Site, fronts};
use grid::{Cell, Grid};
use roads::Road;
use shape::{Point, length_of, sub};
use structgen_api::{Biomes, Fit, Foundation, Materials, Plan, PlanError, Random};
use weft_sdk::*;

use crate::styles;
use crate::worldgen::{OPEN, SEA};

static OPEN_LAND: Biomes = Biomes::named(OPEN);

/// A village's lane is this wide; a town's street, kerbs included, `STREET`.
const LANE: f32 = 3.0;
const STREET: f32 = 7.0;
/// How many spots in a region a settlement tries.
const SPOTS: usize = 3;

/// A village of lanes round a green, or a town of streets round a market square.
#[derive(Clone, Copy, PartialEq)]
pub enum Kind {
    Village,
    Town,
}

impl Kind {
    /// How wide its roads are.
    fn width(self) -> f32 {
        match self {
            Kind::Village => LANE,
            Kind::Town => STREET,
        }
    }

    /// What its roads, and the ways to its doors, are on the grid.
    fn road(self) -> Cell {
        match self {
            Kind::Village => Cell::Lane,
            Kind::Town => Cell::Street,
        }
    }
}

/// One kind of settlement.
pub struct Settlement {
    /// One per region of this many blocks square, at most, with this chance.
    pub region: u32,
    pub chance: f32,
    /// Where two kinds would overlap, the one of higher priority stands: a town over a village
    /// over a hamlet over a lone building (0).
    pub priority: i32,
    pub kind: Kind,
    /// Steps of the main road out of the heart, each way; a branch takes about half as many.
    pub steps: usize,
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
    /// A spot where fewer than four in five of the homes and landmarks fit gets no settlement.
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

/// Lays out one settlement in the region, if the seed wants one there: at the first of a few
/// spots where the ground leaves room for it.
pub fn plan(p: &mut Plan, s: &Settlement) -> Result<(), PlanError> {
    if !p.random().chance(s.chance) {
        return Ok(());
    }
    let reach = (s.steps as f32 * roads::STEP + s.heart[0]) as i32 + 20;
    for _ in 0..SPOTS {
        let middle = p.spot(reach);
        let open = p.ground(middle[0], middle[1]);
        if !open.is_some_and(|g| g.height >= SEA && OPEN_LAND.contains(g.biome)) {
            continue;
        }
        let mut grid = Grid::new(p, middle, reach, &OPEN_LAND);
        if let Some(sites) = lay_out(p, &mut grid, s, grid::middle(middle)) {
            return build(p, s, &grid, &sites);
        }
    }
    Ok(())
}

/// Puts up the buildings in a settlement's materials, a town's paved round (but not its
/// trees), then lays its roads, squares and fields.
fn build(p: &mut Plan, s: &Settlement, grid: &Grid, sites: &[Site]) -> Result<(), PlanError> {
    let walls = walls(p, s);
    for site in sites {
        let roof = styles::blocks(one_of(p, s.roofs).unwrap_or(""));
        let mut materials = styles::add(walls.clone(), roof);
        let tree = s.trees.iter().any(|&t| std::ptr::eq(t, site.template));
        if s.kind == Kind::Town && !tree {
            materials = styles::add(materials, styles::blocks("cobble_ground"));
        }
        p.structure(site.template, site.corner, site.turn)?
            .materials(&materials)
            .foundation(Foundation::Ground)
            .clear_above();
    }
    draw::draw(p, grid)
}

/// Where everything stands round the middle `m`, marked on `grid`: `None` when the ground leaves
/// too little room for a settlement worth the name.
fn lay_out(p: &mut Plan, grid: &mut Grid, s: &Settlement, m: Point) -> Option<Vec<Site>> {
    let angle = unit(p.random()) * TAU;
    let heart = roads::heart(p.random(), m, angle, s.heart);
    // A village's green has a lane round it.
    let ring = if s.kind == Kind::Village {
        LANE / 2.0
    } else {
        0.0
    };
    if !grid.even(&heart, ring) {
        return None;
    }
    let roads = roads::lay(p.random(), grid, s, &heart, m, angle);
    let mut sites = buildings::centre(p, grid, s, m, angle)?;
    let start = p.random().below(1 << 16) as usize;
    let fronts = fronts(&heart, ring, &roads, s.kind.width(), start);
    let rules = rules(s);
    let farmland = reach_of(&roads, m) * 0.6;
    sites.extend(buildings::along(p, grid, s, &rules, &fronts, m, farmland)?);
    // Before the mill goes up, so the road out to it fades like the rest.
    if s.kind == Kind::Town {
        grid.fade_streets(12);
    }
    let from_road = s.kind.width() / 2.0 + 1.0;
    sites.extend(
        s.mill
            .and_then(|mill| buildings::outskirts(p, grid, &rules, mill, &roads[..2], from_road)),
    );
    planting::fields(grid, &roads, s.fields);
    // Before the trees, so paving goes round buildings only.
    if s.kind == Kind::Town {
        grid.pave_frontages();
    }
    let trees = Rules { gap: 3, ..rules };
    sites.extend(planting::trees(
        p,
        grid,
        s.trees,
        &trees,
        m,
        s.heart[0] as i32 + 18,
    ));
    Some(sites)
}

/// How a settlement's buildings line up along its roads; the way to each door is a lane in a
/// village, paving in a town.
fn rules(s: &Settlement) -> Rules {
    Rules {
        setback: s.setback,
        gap: s.gap,
        fit: Fit::new().biome(&OPEN_LAND).max_slope(4).min_height(SEA),
        path: s.kind.road(),
    }
}

/// How far the furthest road reaches from the heart's middle.
fn reach_of(roads: &[Road], m: Point) -> f32 {
    let points = roads.iter().flat_map(|road| road.line.iter());
    points.map(|&at| length_of(sub(at, m))).fold(0.0, f32::max)
}

/// A number from 0 to 1.
fn unit(random: &mut Random) -> f32 {
    random.below(1 << 16) as f32 / 65536.0
}

/// One of `items` at random, `None` when there are none.
fn one_of<T: Copy>(p: &mut Plan, items: &[T]) -> Option<T> {
    match items.len() {
        0 => None,
        n => Some(items[p.random().below(n as u64) as usize]),
    }
}

/// The frame and infill all of a settlement's buildings share.
fn walls(p: &mut Plan, s: &Settlement) -> Materials {
    let frame = styles::blocks(one_of(p, s.frames).unwrap_or(""));
    let infill = styles::blocks(one_of(p, s.infills).unwrap_or(""));
    styles::add(styles::add(Materials::new(), frame), infill)
}
