//! Cities: squares of cells a game master built or marked, each a plaza, a road, a market or
//! part of a plot. A cell is found by its position alone (`grid.rs`), so asking what a block
//! belongs to is one table read. Building, with its undo, is `build.rs`; the commands are
//! `commands.rs`.

mod build;
pub mod buildings;
mod commands;
pub mod grid;
mod notice;
mod place;
pub mod structures;
pub mod undo;

use serde::{Deserialize, Serialize};
use weft_sdk::Registry;
use weftkit::{PlayerKey, Pos};

use self::grid::{CellXZ, Footprint};
use crate::rules::Zone;
use crate::tables::{CELLS, CITIES, COUNTERS, PLOTS};

/// A city: its name, the dimension it stands in and the height of its ground.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct City {
    pub name: String,
    pub dim: u64,
    /// The block layer streets and plots are levelled to.
    pub ground_y: i32,
}

/// What a cell is used for. Kept by its place in this list: new kinds go last.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellKind {
    Plaza,
    Road,
    Plot,
    Market,
}

/// A cell of a city.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Cell {
    pub city: u32,
    pub kind: CellKind,
    pub plot: Option<u32>,
    /// The builds over it, oldest first; `/city undo` takes back the last.
    pub builds: Vec<u32>,
}

/// How big a plot is.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum PlotSize {
    Small,
    Normal,
    Large,
}

/// A plot: where it is, and who it belongs to.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Plot {
    pub city: u32,
    pub dim: u64,
    pub footprint: Footprint,
    pub size: PlotSize,
    pub owner: Option<PlayerKey>,
    pub trusted: Vec<PlayerKey>,
    /// A moderator gave it: it doesn't count against its owner's allowance.
    pub given: bool,
}

impl Plot {
    /// Whether `key` may build on it: its owner or someone they trust.
    pub fn lets_build(&self, key: &PlayerKey) -> bool {
        self.owner.as_ref() == Some(key) || self.trusted.contains(key)
    }
}

/// The next free numbers for cities, plots and builds.
#[derive(Serialize, Deserialize, Clone, Copy, Default, PartialEq, Debug)]
pub struct Counters {
    pub city: u32,
    pub plot: u32,
    pub build: u32,
}

pub fn register(r: &mut Registry) {
    structures::register(r);
    commands::register(r);
    notice::register(r);
}

/// The cell holding the block `pos` of dimension `dim`, where a city has it.
pub fn cell_at(dim: u64, pos: Pos) -> Option<Cell> {
    CELLS.get(&(dim, CellXZ::of(pos)))
}

/// What the block `pos` of dimension `dim` is part of, for `viewer`; `None` outside cities.
pub fn zone(dim: u64, pos: Pos, viewer: Option<&PlayerKey>) -> Option<Zone> {
    let cell = cell_at(dim, pos)?;
    let Some(plot) = cell.plot.and_then(|id| PLOTS.get(&id)) else {
        return Some(Zone::Public);
    };
    Some(match (&plot.owner, viewer) {
        (None, _) => Zone::FreePlot,
        (Some(_), Some(key)) if plot.lets_build(key) => Zone::OwnPlot,
        (Some(_), _) => Zone::OthersPlot,
    })
}

/// The next number of a kind, taken: `pick` names which counter.
pub fn next_id(pick: fn(&mut Counters) -> &mut u32) -> u32 {
    let mut taken = 0;
    COUNTERS.update(&(), |counters| {
        let counter = pick(counters);
        *counter += 1;
        taken = *counter;
    });
    taken
}

/// The city called `name`, any case, with its number.
pub fn city_named(name: &str) -> Option<(u32, City)> {
    let mut found = None;
    CITIES.for_each(|id, city| {
        if city.name.eq_ignore_ascii_case(name) {
            found = Some((id, city));
        }
    });
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(text: &str) -> PlayerKey {
        PlayerKey::new(text).expect("a test key")
    }

    #[test]
    fn owners_and_trusted_players_build() {
        let plot = Plot {
            city: 1,
            dim: 0,
            footprint: Footprint {
                corner: CellXZ::new(0, 0),
                side: 2,
            },
            size: PlotSize::Normal,
            owner: Some(key("ana")),
            trusted: vec![key("bo")],
            given: false,
        };
        assert!(plot.lets_build(&key("ana")), "the owner");
        assert!(plot.lets_build(&key("bo")), "a trusted friend");
        assert!(!plot.lets_build(&key("cy")), "nobody else");
    }
}
