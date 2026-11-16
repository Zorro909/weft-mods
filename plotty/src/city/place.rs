//! Putting a confirmed build into the world: save what is there (`undo.rs`), level the ground,
//! place the template, and write down the cells, the plot and the city it makes. Everything
//! happens in the one command callback, in the region the game master stands in.

use weft_sdk::{Arg, ServerCommand, Template, Transform, World};
use weftkit::{Area, BlockState, Pos, TypedWorld};

use super::build::{Plan, Target, What};
use super::grid::Footprint;
use super::structures::Kind;
use super::undo::{self, Build, Unsaveable};
use super::{Cell, City, Plot, next_id};
use crate::tables::{BUILDS, CELLS, CITIES, PLOTS};
use crate::{dims, plot};

/// Blocks under the ground a build saves and lays with dirt.
pub const BELOW: i32 = 4;
/// Blocks over the ground a build saves and clears.
pub const ABOVE: i32 = 32;

/// Builds `plan`; whether it was built.
pub fn build(cmd: &mut ServerCommand, plan: &Plan) -> bool {
    let world = cmd.world();
    if dims::key_of(&world) != plan.dim {
        cmd.reply("plotty.city.other_dimension", &[]);
        return false;
    }
    let area = plan
        .footprint
        .area(plan.ground_y - BELOW, plan.ground_y + ABOVE);
    let before = match undo::save(&world, area) {
        Ok(before) => before,
        Err(Unsaveable::NotLoaded(pos)) => return refuse(cmd, "plotty.city.not_loaded", pos),
        Err(Unsaveable::Holds(pos)) => return refuse(cmd, "plotty.city.holds", pos),
    };
    let built = match &plan.what {
        What::Kind(kind) => put_kind(&world, plan, *kind),
        What::Building { template, plot } => put_building(&world, plan, template, *plot),
    };
    if !built {
        undo::restore(&world, area, &before);
        cmd.reply("plotty.city.not_placed", &[]);
        return false;
    }
    record(&world, plan, area, before);
    cmd.reply("plotty.city.built", &[]);
    true
}

/// Registers `plan`'s cells as they stand, changing no block: for a town that is already there.
pub fn mark(plan: &Plan) {
    take_cells(plan, None);
}

/// Levels the ground and places one of Plotty's structures.
fn put_kind(world: &World, plan: &Plan, kind: Kind) -> bool {
    level(world, plan.footprint, plan.ground_y);
    let corner = plan.footprint.corner.corner(plan.ground_y);
    world
        .place_template(kind.template(), corner, Transform::NONE, None)
        .is_some()
}

/// Places a structgen building in the middle of a plot, its front toward the street.
fn put_building(world: &World, plan: &Plan, template: &str, plot: u32) -> bool {
    let Some(plot) = PLOTS.get(&plot) else {
        return false;
    };
    let side = plot::post::street_side(plan.dim, plot.footprint);
    let turned = super::buildings::turned(template, side);
    let handle = Template::generated(turned);
    let Some([width, _, depth]) = super::buildings::size_of(handle) else {
        return false;
    };
    let blocks = plot.footprint.blocks();
    let corner = plot.footprint.corner.corner(plan.ground_y);
    let at = corner
        .add_x((blocks - width) / 2)
        .add_z((blocks - depth) / 2);
    world
        .place_template(handle, at, Transform::NONE, None)
        .is_some()
}

/// Clears the air over `footprint` and lays dirt under grass to the city's ground.
fn level(world: &World, footprint: Footprint, ground_y: i32) {
    let dirt = BlockState::named("base:dirt").unwrap_or(BlockState::AIR);
    let grass = BlockState::named("base:grass").unwrap_or(dirt);
    for pos in footprint
        .area(ground_y - BELOW + 1, ground_y + ABOVE)
        .blocks()
    {
        let state = match pos.y.cmp(&ground_y) {
            std::cmp::Ordering::Less => dirt,
            std::cmp::Ordering::Equal => grass,
            std::cmp::Ordering::Greater => BlockState::AIR,
        };
        world.set_block_typed(pos, state);
    }
}

/// Writes down a build that stands: its cells, plot and city, and what it replaced.
fn record(world: &World, plan: &Plan, area: Area, before: undo::Blocks) {
    let id = next_id(|counters| &mut counters.build);
    let mut build = Build {
        dim: plan.dim,
        area,
        cells: Vec::new(),
        plot: None,
        founded: None,
        before,
    };
    match &plan.what {
        What::Kind(_) => {
            let Some((city, plot)) = take_cells(plan, Some(id)) else {
                return;
            };
            build.cells = plan.footprint.cells().collect();
            build.plot = plot;
            build.founded = matches!(plan.target, Target::New(_)).then_some(city);
            if let Some(plot) = plot {
                plot::post::set(world, plot, plan.ground_y);
            }
        }
        What::Building { plot, .. } => {
            let footprint = PLOTS.get(plot).map(|plot| plot.footprint);
            for cell in footprint.into_iter().flat_map(Footprint::cells) {
                let key = (plan.dim, cell);
                if let Some(mut cell) = CELLS.get(&key) {
                    cell.builds.push(id);
                    CELLS.set(&key, cell);
                }
            }
        }
    }
    BUILDS.set(&id, build);
}

/// Gives `plan`'s cells to its city, founding it first if it is new; the city and the plot made.
fn take_cells(plan: &Plan, build: Option<u32>) -> Option<(u32, Option<u32>)> {
    let What::Kind(kind) = plan.what else {
        return None;
    };
    let city = match &plan.target {
        Target::Existing(city) => *city,
        Target::New(name) => {
            let id = next_id(|counters| &mut counters.city);
            CITIES.set(
                &id,
                City {
                    name: name.clone(),
                    dim: plan.dim,
                    ground_y: plan.ground_y,
                },
            );
            id
        }
    };
    let plot = kind.plot_size().map(|size| {
        let id = next_id(|counters| &mut counters.plot);
        let plot = Plot {
            city,
            dim: plan.dim,
            footprint: plan.footprint,
            size,
            owner: None,
            trusted: Vec::new(),
            given: false,
        };
        PLOTS.set(&id, plot);
        id
    });
    let kind_of_cells = kind.cell_kind();
    for cell in plan.footprint.cells() {
        let builds = build.into_iter().collect();
        CELLS.set(
            &(plan.dim, cell),
            Cell {
                city,
                kind: kind_of_cells,
                corner: plan.footprint.corner,
                plot,
                builds,
            },
        );
    }
    Some((city, plot))
}

fn refuse(cmd: &mut ServerCommand, key: &str, pos: Pos) -> bool {
    cmd.reply(
        key,
        &[
            Arg::Int(pos.x.into()),
            Arg::Int(pos.y.into()),
            Arg::Int(pos.z.into()),
        ],
    );
    false
}
