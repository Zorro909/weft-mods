//! Cubes cogwork keeps loaded with nobody near: those of its cuboid loaders, and of its
//! excavators and levelers while they work. The engine tells mods neither which cubes are kept
//! nor how many, so each ticket is noted here too: for the cuboid lens to show, and for loaders
//! to count against the world's limit, since the engine refuses a ticket past it with only a
//! warning in the server log.

mod cubes;
mod lens;
mod loader;
mod machines;
mod tickets;

pub use cubes::{CubePos, Cubes};
#[cfg(test)]
pub use machines::kept_again;
pub use machines::{keep_machine, release_machine};
use twox_hash::XxHash3_64;
use weft_sdk::*;

use crate::lib::Pos;

/// Which ticket of a block is meant: a block may hold more than one.
#[derive(Clone, Copy)]
enum Role {
    /// A loader's cubes, or a machine's own and those around it.
    Home,
    /// The cubes a machine works in.
    Work,
    /// The `n`th cube joining those two, when its work is far above or below them.
    Link(u32),
}

impl Role {
    fn number(self) -> i32 {
        match self {
            Role::Home => 0,
            Role::Work => 1,
            Role::Link(nth) => 2 + nth as i32,
        }
    }
}

pub fn register(r: &mut Registry) {
    loader::register(r);
    lens::register(r);
}

/// The key of the ticket in `role` of the block at `pos`: the same for that block across
/// restarts, so its ticket can be let go when it breaks.
fn key(world: &World, pos: Pos, role: Role) -> u64 {
    let numbers = [world.dimension() as i32, pos.x, pos.y, pos.z, role.number()];
    let bytes: Vec<u8> = numbers
        .iter()
        .flat_map(|number| number.to_le_bytes())
        .collect();
    XxHash3_64::oneshot(&bytes)
}

/// A block that keeps cubes loaded was broken: its tickets go now, even its own and its work's
/// restored with the world that cogwork has not noted again yet.
pub fn let_go(world: &World, change: &BlockChange) {
    let pos = Pos::from(change.pos);
    tickets::release_any(world, key(world, pos, Role::Home));
    tickets::release_any(world, key(world, pos, Role::Work));
    machines::release_links(world, pos, 0);
}
