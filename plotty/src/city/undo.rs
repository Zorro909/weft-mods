//! What a build replaced, so `/city undo` can always put it back: every cell of its area by the
//! names of what is in it (ids shift when mods come or go), as runs of one name in the order
//! `Area::blocks` walks the area.

use serde::{Deserialize, Serialize};
use weft_sdk::{World, abi, name_of};
use weftkit::{Area, BlockState, Pos, TypedWorld};

use super::grid::{CellXZ, Footprint};
use crate::tables::{BUILDS, CELLS, CITIES, PLOTS};

/// One build: where it stood and what was there before.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Build {
    pub dim: u64,
    pub area: Area,
    /// The cells it took into its city, given back when it is undone.
    pub cells: Vec<CellXZ>,
    /// The plot it made, if it made one.
    pub plot: Option<u32>,
    /// The city it founded, if it founded one.
    pub founded: Option<u32>,
    pub before: Blocks,
}

/// Between the parts of one cell's name: a slab and the water in it, a flower and the snow on it.
const PARTS: char = ' ';

/// Block states of an area as runs: `names`, and (index into `names`, how many) in walking order.
#[derive(Serialize, Deserialize, Clone, Default, PartialEq, Debug)]
pub struct Blocks {
    pub names: Vec<String>,
    pub runs: Vec<(u32, u32)>,
}

impl Blocks {
    /// Adds the next block's name.
    pub fn push(&mut self, name: &str) {
        let index = match self.names.iter().position(|known| known == name) {
            Some(index) => index as u32,
            None => {
                self.names.push(name.to_string());
                self.names.len() as u32 - 1
            }
        };
        match self.runs.last_mut() {
            Some((last, count)) if *last == index => *count += 1,
            _ => self.runs.push((index, 1)),
        }
    }

    /// Every block's name, in the order they were pushed.
    pub fn names_in_order(&self) -> impl Iterator<Item = &str> {
        self.runs.iter().flat_map(|(index, count)| {
            let name = self
                .names
                .get(*index as usize)
                .map_or("base:air", String::as_str);
            std::iter::repeat_n(name, *count as usize)
        })
    }
}

/// Why an area can't be saved.
pub enum Unsaveable {
    /// Part of it is not loaded where the command runs.
    NotLoaded(Pos),
    /// A block holds things (a chest, a furnace, a machine), which only block states can't keep.
    Holds(Pos),
}

/// The block states of `area`, refusing an area with a block that holds things.
pub fn save(world: &World, area: Area) -> Result<Blocks, Unsaveable> {
    let mut blocks = Blocks::default();
    for pos in area.blocks() {
        let state = world.block_loaded(pos).ok_or(Unsaveable::NotLoaded(pos))?;
        if world.block_entity(pos) != 0 {
            return Err(Unsaveable::Holds(pos));
        }
        let parts: Vec<&str> = world
            .parts(pos)
            .into_iter()
            .map(|part| name_of(abi::entry_kind::BLOCK, part))
            .collect();
        blocks.push(&cell_name(name_of(abi::entry_kind::BLOCK, state), &parts));
    }
    Ok(blocks)
}

/// The name a cell is saved by: `main`, the block in it, or every part where it has more.
fn cell_name(main: &str, parts: &[&str]) -> String {
    if parts.len() > 1 {
        parts.join(&PARTS.to_string())
    } else {
        main.to_string()
    }
}

/// Puts back what `save` saved of `area`; a block whose mod is gone becomes air.
pub fn restore(world: &World, area: Area, blocks: &Blocks) {
    for (pos, name) in area.blocks().zip(blocks.names_in_order()) {
        let mut parts = name
            .split(PARTS)
            .map(|part| BlockState::named(part).unwrap_or(BlockState::AIR));
        // A block set over the same block keeps the rest of its cell, which would come twice.
        if world.parts(pos).len() > 1 {
            world.set_block_typed(pos, BlockState::AIR);
        }
        world.set_block_typed(pos, parts.next().unwrap_or(BlockState::AIR));
        parts.for_each(|part| {
            world.part_add_typed(pos, part);
        });
    }
}

/// Takes back the last build over `cell` of dimension `dim`: puts back what it replaced and
/// forgets the cells, plot and city it made. Refused while a player owns the plot, and for a
/// city's founding plaza while the city has other cells.
pub fn undo_at(world: &World, dim: u64, cell: CellXZ) -> Result<(), &'static str> {
    let here = CELLS.get(&(dim, cell)).ok_or("plotty.city.no_city_here")?;
    let id = *here.builds.last().ok_or("plotty.city.nothing_to_undo")?;
    let build = BUILDS.get(&id).ok_or("plotty.city.nothing_to_undo")?;
    let plot = build.plot.or(here.plot).and_then(|plot| PLOTS.get(&plot));
    if plot.is_some_and(|plot| plot.owner.is_some()) {
        return Err("plotty.city.undo_claimed");
    }
    if build
        .founded
        .is_some_and(|city| has_other_cells(city, &build.cells))
    {
        return Err("plotty.city.undo_plaza_last");
    }
    if market_rented(dim, &build) {
        return Err("plotty.city.undo_rented");
    }
    // What was built there since, chests and stalls among them, would be lost.
    match save(world, build.area) {
        Err(Unsaveable::NotLoaded(_)) => return Err("plotty.city.undo_not_loaded"),
        Err(Unsaveable::Holds(_)) => return Err("plotty.city.undo_holds"),
        Ok(_) => {}
    }
    restore(world, build.area, &build.before);
    forget(dim, id, &build, here.plot);
    Ok(())
}

/// Whether build `build` made a market square with a place still rented.
fn market_rented(dim: u64, build: &Build) -> bool {
    build
        .cells
        .first()
        .is_some_and(|corner| crate::market::any_rented(dim, *corner))
}

/// Removes what build `id` wrote down: its own cells, or its number on the plot it stood on.
fn forget(dim: u64, id: u32, build: &Build, plot_under: Option<u32>) {
    if build.cells.is_empty() {
        let footprint = plot_under
            .and_then(|plot| PLOTS.get(&plot))
            .map(|plot| plot.footprint);
        for cell in footprint.into_iter().flat_map(Footprint::cells) {
            if let Some(mut entry) = CELLS.get(&(dim, cell)) {
                entry.builds.retain(|build| *build != id);
                CELLS.set(&(dim, cell), entry);
            }
        }
    }
    build
        .cells
        .iter()
        .for_each(|cell| CELLS.remove(&(dim, *cell)));
    build.plot.iter().for_each(|plot| PLOTS.remove(plot));
    if let Some(corner) = build.cells.first() {
        crate::market::forget_rents(dim, *corner);
    }
    build.founded.iter().for_each(|city| CITIES.remove(city));
    BUILDS.remove(&id);
}

/// Whether city `city` has cells besides `own`.
pub fn has_other_cells(city: u32, own: &[CellXZ]) -> bool {
    let mut other = false;
    CELLS.for_each(|(_, cell), entry| other |= entry.city == city && !own.contains(&cell));
    other
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_give_back_every_block_in_order() {
        let walked = [
            "base:stone",
            "base:stone",
            "base:air",
            "base:stone",
            "base:air",
            "base:air",
        ];
        let mut blocks = Blocks::default();
        walked.iter().for_each(|name| blocks.push(name));
        assert_eq!(blocks.names.len(), 2, "two kinds");
        assert_eq!(blocks.runs.len(), 4, "four runs");
        assert_eq!(
            blocks.names_in_order().collect::<Vec<_>>(),
            walked,
            "the same order back"
        );
    }

    #[test]
    fn a_cell_keeps_every_part() {
        let name = cell_name("base:oak_slab", &["base:oak_slab", "base:water"]);
        assert_eq!(
            name.split(PARTS).collect::<Vec<_>>(),
            ["base:oak_slab", "base:water"],
            "the water too"
        );
        assert_eq!(cell_name("base:air", &[]), "base:air", "air has no parts");
    }

    #[test]
    fn a_name_lost_from_the_list_reads_as_air() {
        let blocks = Blocks {
            names: vec![],
            runs: vec![(3, 2)],
        };
        assert_eq!(
            blocks.names_in_order().collect::<Vec<_>>(),
            ["base:air"; 2],
            "air, not a panic"
        );
    }
}
