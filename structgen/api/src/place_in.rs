//! `Structure::place_in`: puts a structure into the world now, turned, the way a command would.
//! All of it or nothing.

use std::collections::HashMap;
use std::fmt;

use weft_sdk::*;
use weftkit::{BlockState, Pos};

use crate::format::Structure;
use crate::turn::{Turn, turn_state};

/// What `place_in` replaced: each cell's position and all it held (`World::parts`).
pub type Replaced = Vec<(Pos, Vec<BlockState>)>;

/// Why `place_in` placed nothing.
#[derive(Clone, Debug, PartialEq)]
pub enum PlaceError {
    /// A block the structure has and this game doesn't (`base:furnace[facing=east]`).
    UnknownBlock(String),
    /// Part of where it would go isn't loaded.
    NotLoaded,
}

impl fmt::Display for PlaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlaceError::UnknownBlock(name) => write!(f, "{name} is no block of this game"),
            PlaceError::NotLoaded => write!(f, "part of where it would go isn't loaded"),
        }
    }
}

/// Puts back what `place_in` replaced.
pub fn put_back(world: &World, replaced: &Replaced) {
    for (pos, parts) in replaced {
        write_cell(world, *pos, parts);
    }
}

/// Makes the cell at `pos` hold just `parts`, left alone if it does. Its first block is
/// replaced in place, so a chest turned or put back keeps what is in it.
fn write_cell(world: &World, pos: Pos, parts: &[BlockState]) {
    let now = parts_at(world, pos);
    if now == parts {
        return;
    }
    for extra in now.iter().skip(1) {
        world.part_remove(pos, extra.id());
    }
    let first = parts.first().copied().unwrap_or(BlockState::AIR);
    first.place(world, pos);
    for part in parts.iter().skip(1) {
        world.part_add(pos, part.id());
    }
}

/// Everything the cell at `pos` holds (`World::parts`).
fn parts_at(world: &World, pos: Pos) -> Vec<BlockState> {
    world.parts(pos).into_iter().map(BlockState::from).collect()
}

impl Structure {
    /// Puts it into `world` turned by `turn`, the lowest corner of the turned box at `corner`.
    /// Every block is set, air too. On an error nothing is. Gives back what it replaced, for
    /// `put_back`.
    pub fn place_in(&self, world: &World, corner: Pos, turn: Turn) -> Result<Replaced, PlaceError> {
        let states = self.states(turn)?;
        let size = self.size();
        let blocks: Vec<(Pos, BlockState)> = self
            .blocks()
            .map(|(pos, name)| (corner + turn.pos(pos, size), states[name]))
            .collect();
        let replaced: Option<Replaced> = blocks
            .iter()
            .map(|&(pos, _)| {
                BlockState::loaded_at(world, pos)?;
                Some((pos, parts_at(world, pos)))
            })
            .collect();
        let replaced = replaced.ok_or(PlaceError::NotLoaded)?;
        for &(pos, state) in &blocks {
            write_cell(world, pos, &[state]);
        }
        Ok(replaced)
    }

    /// Each block name as a state, turned.
    fn states(&self, turn: Turn) -> Result<HashMap<&str, BlockState>, PlaceError> {
        self.palette
            .iter()
            .map(|name| match BlockState::named(name) {
                Some(state) => Ok((name.as_str(), turn_state(state.id(), turn).into())),
                None => Err(PlaceError::UnknownBlock(name.clone())),
            })
            .collect()
    }
}
