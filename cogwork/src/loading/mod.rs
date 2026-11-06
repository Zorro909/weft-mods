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
#[repr(u8)]
enum Role {
    /// A loader's cubes, or a machine's own and those around it.
    Home,
    /// The cubes a machine works in.
    Work,
    /// The cubes between those two, when its work is above or below them.
    Link,
}

const ROLES: [Role; 3] = [Role::Home, Role::Work, Role::Link];

pub fn register(r: &mut Registry) {
    loader::register(r);
    lens::register(r);
}

/// The key of the ticket in `role` of the block at `pos`: the same for that block across
/// restarts, so its ticket can be let go when it breaks.
fn key(w: &World, pos: Pos, role: Role) -> u64 {
    let numbers = [w.dimension() as i32, pos.x, pos.y, pos.z, role as i32];
    let bytes: Vec<u8> = numbers.iter().flat_map(|n| n.to_le_bytes()).collect();
    XxHash3_64::oneshot(&bytes)
}

/// A block that keeps cubes loaded was broken: its tickets go now, even ones restored with the
/// world that cogwork has not noted again yet.
pub fn let_go(w: &World, change: &BlockChange) {
    let pos = Pos::from(change.pos);
    for role in ROLES {
        tickets::release_any(w, key(w, pos, role));
    }
}
