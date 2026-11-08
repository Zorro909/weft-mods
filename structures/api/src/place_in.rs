//! `Structure::place_in`: puts a structure into the world now, turned, the way a command would.
//! All of it or nothing.

use std::collections::HashMap;
use std::fmt;

use weft_sdk::*;

use crate::format::Structure;
use crate::turn::{Turn, turn_state};

/// What `place_in` replaced: each cell's position and all it held (`World::parts`).
pub type Replaced = Vec<([i32; 3], Vec<u32>)>;

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

/// Puts back what `place_in` replaced. A cell that holds it still is left alone.
pub fn put_back(world: &World, replaced: &Replaced) {
    for (pos, parts) in replaced {
        let now = world.parts(*pos);
        if now != *parts {
            write_cell(world, *pos, &now, parts);
        }
    }
}

/// Makes the cell at `pos`, holding `now`, hold just `parts`. Its first block is replaced in
/// place, so a chest turned or put back keeps what is in it.
fn write_cell(world: &World, pos: [i32; 3], now: &[u32], parts: &[u32]) {
    for &extra in now.iter().skip(1) {
        world.part_remove(pos, extra);
    }
    world.set_block(pos, parts.first().copied().unwrap_or(0));
    for &part in parts.iter().skip(1) {
        world.part_add(pos, part);
    }
}

impl Structure {
    /// Puts it into `world` turned by `turn`, the lowest corner of the turned box at `corner`.
    /// Every block is set, air too. On an error nothing is. Gives back what it replaced, for
    /// `put_back`.
    pub fn place_in(
        &self,
        world: &World,
        corner: [i32; 3],
        turn: Turn,
    ) -> Result<Replaced, PlaceError> {
        let states = self.states(turn)?;
        let (size, corner) = (self.size(), BlockPos::from(corner));
        let blocks: Vec<([i32; 3], u32)> = self
            .blocks()
            .map(|(pos, name)| {
                let pos = corner + turn.pos(pos, size).into();
                (pos.to_array(), states[name])
            })
            .collect();
        let replaced: Option<Replaced> = blocks
            .iter()
            .map(|&(pos, _)| {
                world.block_loaded(pos)?;
                Some((pos, world.parts(pos)))
            })
            .collect();
        let replaced = replaced.ok_or(PlaceError::NotLoaded)?;
        for (&(pos, state), (_, had)) in blocks.iter().zip(&replaced) {
            write_cell(world, pos, had, &[state]);
        }
        Ok(replaced)
    }

    /// Each block name as a state, turned.
    fn states(&self, turn: Turn) -> Result<HashMap<&str, u32>, PlaceError> {
        self.palette
            .iter()
            .map(|name| match Block::lookup(name) {
                Some(state) => Ok((name.as_str(), turn_state(state, turn))),
                None => Err(PlaceError::UnknownBlock(name.clone())),
            })
            .collect()
    }
}
