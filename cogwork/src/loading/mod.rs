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
    key_in(world.dimension(), pos, role)
}

/// The key of that ticket in the dimension numbered `dimension`.
fn key_in(dimension: u32, pos: Pos, role: Role) -> u64 {
    let numbers = [dimension as i32, pos.x, pos.y, pos.z, role.number()];
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ticket_has_a_key_of_its_own() {
        let pos = Pos::new(3, -20, 7);
        let roles = [Role::Home, Role::Work, Role::Link(0), Role::Link(1)];
        let mut keys: Vec<u64> = roles.map(|role| key_in(0, pos, role)).to_vec();
        keys.push(key_in(1, pos, Role::Home));
        keys.push(key_in(0, pos.add_y(1), Role::Home));
        // The numbers of a block swapped around are another block.
        keys.push(key_in(0, Pos::new(7, -20, 3), Role::Home));
        let mut distinct = keys.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), keys.len(), "no two alike: {keys:?}");
        assert_eq!(key_in(0, pos, Role::Work), keys[1], "the same each time");
    }
}
