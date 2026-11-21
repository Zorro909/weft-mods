//! `/city`: founding and growing cities, which game masters do, and what anyone may ask of them.

use weft_sdk::{Arg, ServerCommand, World, permission};
use weftkit::{PlayerKey, Pos};

use super::build::{self, Plan, Target, What};
use super::grid::{CellXZ, Footprint};
use super::structures::{self, Kind};
use super::{CellKind, buildings, cell_at, city_named, undo};
use crate::tables::{self, CELLS, CITIES, PLOTS};
use crate::{dims, market, plot};

const USAGE: &str = "found <name:word> [how:word(build,mark)] \
                     | build <kind:word(plaza,road,small,normal,large,market)> | building <name:word> \
                     | buildings | confirm | cancel | undo \
                     | mark <kind:word(plaza,road,small,normal,large,market)> | unmark \
                     | info | list | rename <name:word> | delete <name:word>";

/// Who runs a city command, and where.
struct Here {
    world: World,
    key: PlayerKey,
    dim: u64,
    pos: Pos,
}

pub fn register(r: &mut weft_sdk::Registry) {
    r.command("city", USAGE, run)
        .permission(permission::EVERYONE);
}

fn run(cmd: &mut ServerCommand) {
    let Some(here) = here(cmd) else {
        return cmd.reply("plotty.only_players", &[]);
    };
    let for_anyone = cmd.given("info") || cmd.given("list") || cmd.given("buildings");
    if !for_anyone && here.world.permission(cmd.player()) < permission::GAME_MASTER {
        return cmd.reply("plotty.game_masters_only", &[]);
    }
    let _changing = tables::changing();
    if cmd.given("found") {
        found(cmd, &here);
    } else if cmd.given("build") || cmd.given("mark") {
        grow(cmd, &here, cmd.given("mark"));
    } else if cmd.given("building") {
        building(cmd, &here);
    } else if cmd.given("buildings") {
        list_buildings(cmd);
    } else if cmd.given("confirm") {
        build::confirm(cmd, &here.key);
    } else if cmd.given("cancel") {
        build::cancel(cmd, &here.key);
    } else if cmd.given("undo") {
        undo(cmd, &here);
    } else if cmd.given("unmark") {
        unmark(cmd, &here);
    } else if cmd.given("list") {
        list(cmd);
    } else if cmd.given("rename") {
        rename(cmd, &here);
    } else if cmd.given("delete") {
        delete(cmd);
    } else {
        info(cmd, &here);
    }
}

fn here(cmd: &ServerCommand) -> Option<Here> {
    let world = cmd.world();
    let key = PlayerKey::of(&world, cmd.player())?;
    let pos = Pos::from(world.entity(cmd.player())?.pos.block());
    Some(Here {
        world,
        key,
        dim: dims::key_of(&world),
        pos,
    })
}

/// A new city with its plaza around the player, on the ground they stand on.
fn found(cmd: &mut ServerCommand, here: &Here) {
    let name = cmd.text("name").unwrap_or_default().to_string();
    if city_named(&name).is_some() {
        return cmd.reply("plotty.city.name_taken", &[Arg::Text(&name)]);
    }
    let footprint = match build::footprint_for(here.dim, here.pos, Kind::Plaza.side(), true) {
        Ok((footprint, _)) => footprint,
        Err(problem) => return cmd.reply(problem, &[]),
    };
    let what = What::Kind(Kind::Plaza);
    let plan = Plan {
        what,
        target: Target::New(name),
        dim: here.dim,
        footprint,
        ground_y: here.pos.y - 1,
    };
    if cmd.text("how") == Some("mark") {
        return build::mark(cmd, &plan);
    }
    build::propose(cmd, here.key, plan);
}

/// Cells of a kind next to a city: built after a preview, or only `mark`ed.
fn grow(cmd: &mut ServerCommand, here: &Here, mark: bool) {
    let Some(kind) = cmd.text("kind").and_then(Kind::named) else {
        return;
    };
    let (footprint, city) = match build::footprint_for(here.dim, here.pos, kind.side(), false) {
        Ok((footprint, Some(city))) => (footprint, city),
        Ok((_, None)) => return,
        Err(problem) => return cmd.reply(problem, &[]),
    };
    let Some(ground_y) = build::ground_of(city) else {
        return;
    };
    let plan = Plan {
        what: What::Kind(kind),
        target: Target::Existing(city),
        dim: here.dim,
        footprint,
        ground_y,
    };
    if mark {
        return build::mark(cmd, &plan);
    }
    build::propose(cmd, here.key, plan);
}

/// A structgen building on the free plot the player stands on.
fn building(cmd: &mut ServerCommand, here: &Here) {
    let name = cmd.text("name").unwrap_or_default().to_string();
    let Some((id, plot)) = plot::at(here.dim, here.pos) else {
        return cmd.reply("plotty.plot.none_here", &[]);
    };
    if plot.owner.is_some() {
        return cmd.reply("plotty.city.building_on_claimed", &[]);
    }
    let side = plot::post::street_side(here.dim, plot.footprint);
    let template = weft_sdk::Template::generated(buildings::turned(&name, side));
    let Some(size) = buildings::size_of(template) else {
        return cmd.reply("plotty.city.no_building", &[Arg::Text(&name)]);
    };
    if !buildings::fits(size, plot.footprint.blocks()) {
        return cmd.reply("plotty.city.building_too_big", &[Arg::Text(&name)]);
    }
    let Some(ground_y) = build::ground_of(plot.city) else {
        return;
    };
    let what = What::Building {
        template: name,
        plot: id,
    };
    let plan = Plan {
        what,
        target: Target::Existing(plot.city),
        dim: here.dim,
        footprint: plot.footprint,
        ground_y,
    };
    build::propose(cmd, here.key, plan);
}

fn list_buildings(cmd: &mut ServerCommand) {
    let known: Vec<&str> = buildings::BUILDINGS
        .into_iter()
        .filter(|name| {
            buildings::size_of(weft_sdk::Template::generated(format!("structgen:{name}"))).is_some()
        })
        .collect();
    if known.is_empty() {
        return cmd.reply("plotty.city.no_structgen", &[]);
    }
    cmd.reply("plotty.city.buildings", &[Arg::Text(&known.join(", "))]);
}

fn undo(cmd: &mut ServerCommand, here: &Here) {
    match undo::undo_at(&here.world, here.dim, CellXZ::of(here.pos)) {
        Ok(()) => cmd.reply("plotty.city.undone", &[]),
        Err(problem) => cmd.reply(problem, &[]),
    }
}

/// Takes a marked cell out of its city, with the rest of the plot, plaza or market it is part of;
/// built ones need `/city undo`. Refused as `/city undo` refuses; a city left without cells is
/// forgotten.
fn unmark(cmd: &mut ServerCommand, here: &Here) {
    let Some(cell) = cell_at(here.dim, here.pos) else {
        return cmd.reply("plotty.city.no_city_here", &[]);
    };
    if !cell.builds.is_empty() {
        return cmd.reply("plotty.city.unmark_built", &[]);
    }
    let plot = cell
        .plot
        .and_then(|id| PLOTS.get(&id).map(|plot| (id, plot)));
    let footprint = match &plot {
        Some((_, plot)) => plot.footprint,
        None => Footprint {
            corner: cell.corner,
            side: structures::side_of(cell.kind),
        },
    };
    let cells: Vec<CellXZ> = footprint.cells().collect();
    let refused = if plot.as_ref().is_some_and(|(_, plot)| plot.owner.is_some()) {
        Some("plotty.city.undo_claimed")
    } else if cell.kind == CellKind::Plaza && undo::has_other_cells(cell.city, &cells) {
        Some("plotty.city.undo_plaza_last")
    } else if cell.kind == CellKind::Market && market::any_rented(here.dim, cell.corner) {
        Some("plotty.city.undo_rented")
    } else {
        None
    };
    if let Some(refused) = refused {
        return cmd.reply(refused, &[]);
    }
    cells.iter().for_each(|at| CELLS.remove(&(here.dim, *at)));
    plot.iter().for_each(|(id, _)| PLOTS.remove(id));
    market::forget_rents(here.dim, cell.corner);
    if !undo::has_other_cells(cell.city, &[]) {
        CITIES.remove(&cell.city);
    }
    cmd.reply("plotty.city.unmarked", &[]);
}

/// The city here: its name and how many of its plots are free and taken.
fn info(cmd: &mut ServerCommand, here: &Here) {
    let Some(cell) = cell_at(here.dim, here.pos) else {
        return cmd.reply("plotty.city.no_city_here", &[]);
    };
    let name = CITIES
        .get(&cell.city)
        .map(|city| city.name)
        .unwrap_or_default();
    let (mut free, mut taken) = (0i64, 0i64);
    PLOTS.for_each(|_, plot| {
        if plot.city == cell.city {
            if plot.owner.is_some() {
                taken += 1
            } else {
                free += 1
            }
        }
    });
    let kind = match cell.kind {
        CellKind::Plaza => "plotty.kind.plaza",
        CellKind::Road => "plotty.kind.road",
        CellKind::Plot => "plotty.kind.plot",
        CellKind::Market => "plotty.kind.market",
    };
    cmd.reply(
        "plotty.city.info",
        &[
            Arg::Text(&name),
            Arg::Key(kind),
            Arg::Int(free),
            Arg::Int(taken),
        ],
    );
}

fn list(cmd: &mut ServerCommand) {
    let mut names = Vec::new();
    CITIES.for_each(|_, city| names.push(city.name));
    names.sort();
    cmd.reply(
        "plotty.city.list",
        &[Arg::Int(names.len() as i64), Arg::Text(&names.join(", "))],
    );
}

fn rename(cmd: &mut ServerCommand, here: &Here) {
    let name = cmd.text("name").unwrap_or_default().to_string();
    let Some(cell) = cell_at(here.dim, here.pos) else {
        return cmd.reply("plotty.city.no_city_here", &[]);
    };
    if city_named(&name).is_some() {
        return cmd.reply("plotty.city.name_taken", &[Arg::Text(&name)]);
    }
    let Some(mut city) = CITIES.get(&cell.city) else {
        return;
    };
    city.name = name.clone();
    CITIES.set(&cell.city, city);
    cmd.reply("plotty.city.renamed", &[Arg::Text(&name)]);
}

/// Forgets a city, its cells, plots and builds; its blocks stay as they stand.
fn delete(cmd: &mut ServerCommand) {
    let name = cmd.text("name").unwrap_or_default().to_string();
    let Some((id, _)) = city_named(&name) else {
        return cmd.reply("plotty.city.unknown", &[Arg::Text(&name)]);
    };
    let mut cells = Vec::new();
    CELLS.for_each(|key, cell| {
        if cell.city == id {
            cells.push((key, cell))
        }
    });
    for ((dim, at), cell) in cells {
        CELLS.remove(&(dim, at));
        cell.builds
            .iter()
            .for_each(|build| crate::tables::BUILDS.remove(build));
        if cell.kind == CellKind::Market {
            crate::market::forget_rents(dim, cell.corner);
        }
    }
    let mut plots = Vec::new();
    PLOTS.for_each(|plot, entry| {
        if entry.city == id {
            plots.push(plot)
        }
    });
    plots.iter().for_each(|plot| PLOTS.remove(plot));
    CITIES.remove(&id);
    cmd.reply("plotty.city.deleted", &[Arg::Text(&name)]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_usage_lists_every_kind() {
        let listed = format!("({})", super::super::structures::WORDS);
        assert_eq!(
            USAGE.matches(&listed).count(),
            2,
            "build and mark take the same kinds"
        );
    }
}
