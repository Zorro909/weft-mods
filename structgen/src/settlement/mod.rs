//! Settlements laid out afresh from the seed and the lie of the land. Each has a heart, a
//! village green or a town's market square, shaped at random and stretched along a main road
//! that runs out of it both ways; branches leave that road, and every road winds along the
//! ground (`roads.rs`). Buildings stand along both sides of the roads and round the heart, each
//! turned to face it (`frontage.rs`); gardens, a few small fields and trees fill in. What a kind
//! of settlement holds is data, `Settlement`; this is the planner they share.

mod biomes;
mod buildings;
mod draw;
mod fit;
mod frontage;
mod grid;
mod plan;
mod planting;
mod random;
mod roads;
mod shape;

use std::f32::consts::TAU;

use biomes::Biomes;
use fit::Fit;
pub use frontage::Faces;
use frontage::{Rules, Site, fronts};
use grid::{Cell, Grid};
use plan::Plan;
use random::Random;
use roads::Road;
use shape::{Point, length_of, sub};
use weft_sdk::*;
use weftkit::Pos;

use crate::styles;

/// Open land to build on: plains and snowy plains, without the trees of forests, which would
/// stand cut through a building.
pub const OPEN: &str = "#animals !#forest !#mountain";
static OPEN_LAND: Biomes = Biomes::named(OPEN);
/// Base's sea level: lakes and the sea fill the ground below it, so buildings stand at least
/// this high.
const SEA: i32 = 63;
/// How many spots a settlement tries, and how far from its start the others are at most.
const SPOTS: usize = 3;
pub const SHIFT: i32 = 48;
/// What one or two buildings of every settlement keep in their chests.
pub static CHESTS: LootTable = LootTable::named("settlement_chest");

/// A village's lane is this wide; a town's street, kerbs included, `STREET`.
const LANE: f32 = 3.0;
const STREET: f32 = 7.0;

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

    /// How many blocks its heart may rise from its lowest column to its highest: a paved square
    /// shows every step.
    fn rise(self) -> i32 {
        match self {
            Kind::Village => 6,
            Kind::Town => 3,
        }
    }

    /// What its heart is on the grid: a green or a square.
    fn heart(self) -> Cell {
        match self {
            Kind::Village => Cell::Green,
            Kind::Town => Cell::Square,
        }
    }

    /// How far round its heart a road runs: a village's green has a lane round it.
    fn ring(self) -> f32 {
        match self {
            Kind::Village => LANE / 2.0,
            Kind::Town => 0.0,
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
    /// One per square of this many blocks, at most, with this chance.
    pub spacing: i32,
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

impl Settlement {
    /// How far from its middle a settlement reaches: its roads, and the buildings and fields
    /// along them.
    pub fn reach(&self) -> i32 {
        (self.steps as f32 * roads::STEP + self.heart[0]) as i32 + 20
    }

    /// Whether a settlement with its middle at `middle` keeps to the square of `start`, so it
    /// overlaps none of its kind from the squares around.
    fn keeps_to_its_square(&self, start: Pos, [x, z]: [i32; 2]) -> bool {
        let inside = |start: i32, v: i32| {
            let square = start.div_euclid(self.spacing) * self.spacing;
            (square + self.reach()..square + self.spacing - self.reach()).contains(&v)
        };
        inside(start.x, x) && inside(start.z, z)
    }
}

/// Lays out the settlement of `start`, if the ground leaves room for one: with its middle at
/// the start, or else at one of a few spots up to `SHIFT` from it.
pub fn plan(start: &mut weft_sdk::Plan, s: &Settlement) {
    let mut p = Plan::new(start);
    let at = p.start();
    for spot in 0..SPOTS {
        let middle = match spot {
            0 => [at.x, at.z],
            _ => [
                at.x + p.random().between(-SHIFT, SHIFT),
                at.z + p.random().between(-SHIFT, SHIFT),
            ],
        };
        let open = p.ground(middle[0], middle[1]);
        let open = open.is_some_and(|g| g.height >= SEA && OPEN_LAND.contains(g.biome));
        if !open || !s.keeps_to_its_square(at, middle) {
            continue;
        }
        let mut grid = Grid::new(&p, middle, s.reach(), &OPEN_LAND);
        if let Some(sites) = lay_out(&mut p, &mut grid, s, grid::middle(middle)) {
            return build(&mut p, s, &grid, &sites);
        }
    }
}

/// Puts up the buildings in a settlement's styles, a town's paved round (but not its trees or
/// its mill out in the fields), stocks the chests of one or two, then lays its roads, squares
/// and fields.
fn build(p: &mut Plan, s: &Settlement, grid: &Grid, sites: &[Site]) {
    let walls = [one_of(p, s.frames), one_of(p, s.infills)].map(Option::unwrap_or_default);
    for site in sites {
        let roof = one_of(p, s.roofs).unwrap_or_default();
        let is = |t: &Template| std::ptr::eq(t, site.template);
        let outside = s.trees.iter().any(|&t| is(t)) || s.mill.is_some_and(is);
        let paved = s.kind == Kind::Town && !outside;
        let palette = styles::palette(&[roof, walls[0], walls[1]]);
        p.building(site.template, site.corner, site.turn, palette, paved);
    }
    stock_chests(p, sites);
    draw::draw(p, grid)
}

/// Fills the chests of one or two of `sites` from `CHESTS`.
fn stock_chests(p: &mut Plan, sites: &[Site]) {
    let with_chests: Vec<(&Site, Vec<Pos>)> = (sites.iter())
        .map(|site| (site, crate::buildings::chests(site.template, site.turn)))
        .filter(|(_, chests)| !chests.is_empty())
        .collect();
    let n = with_chests.len() as u64;
    if n == 0 {
        return;
    }
    let first = p.random().below(n);
    // Half the time a second, another one.
    let second =
        (n > 1 && p.random().chance(0.5)).then(|| (first + 1 + p.random().below(n - 1)) % n);
    for i in [Some(first), second].into_iter().flatten() {
        let (site, chests) = &with_chests[i as usize];
        for &chest in chests {
            p.loot(site.corner + chest, &CHESTS);
        }
    }
}

/// Where everything stands round the middle `m`, marked on `grid`: `None` when the ground leaves
/// too little room for a settlement worth the name.
fn lay_out(p: &mut Plan, grid: &mut Grid, s: &Settlement, m: Point) -> Option<Vec<Site>> {
    let angle = unit(p.random()) * TAU;
    let heart = roads::heart(p.random(), m, angle, s.heart);
    let ring = s.kind.ring();
    if !grid.even(&heart, ring, s.kind.rise()) {
        return None;
    }
    let roads = roads::lay(p.random(), grid, s, &heart, m, angle);
    let mut sites = buildings::centre(p, grid, s, m, angle)?;
    let start = p.random().below(1 << 16) as usize;
    let fronts = fronts(&heart, ring, &roads, s.kind.width(), start);
    let rules = rules(s);
    let farmland = reach_of(&roads, m) * 0.6;
    sites.extend(buildings::along(p, grid, s, &rules, &fronts, m, farmland)?);
    // Before the mill goes up and the trees are planted, so the road out to the mill fades like
    // the rest and the paving goes round the town's own buildings only.
    if s.kind == Kind::Town {
        grid.fade_streets(12);
        grid.pave_frontages();
    }
    let from_road = s.kind.width() / 2.0 + 1.0;
    sites.extend(
        s.mill
            .and_then(|mill| buildings::outskirts(p, grid, &rules, mill, &roads[..2], from_road)),
    );
    planting::fields(grid, &roads, s.fields);
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
