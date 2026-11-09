//! Where an item goes from a tube's hub: any arm that can take it but the one it came by, as
//! each kind of tube allows. Among several, one at random, so items spread over branches.

use weft_sdk::*;

use super::{DIRECTING_TUBE, Kind};
use crate::lib::{Pos, TypedWorld};

pub enum Route {
    /// On through this side.
    Out(Dir),
    /// Nowhere to go for now: it waits at the hub.
    Wait,
    /// A void tube: gone.
    Destroy,
    /// No arms at all: it falls out.
    Spill,
}

/// Where `stack` goes from the hub of the tube `id` at `pos`. `from` is the side it came in by,
/// `None` for one put in at the hub.
pub fn route(
    world: &World,
    id: u64,
    pos: Pos,
    kind: Kind,
    from: Option<Dir>,
    stack: ItemStack,
) -> Route {
    if kind == Kind::Void {
        return Route::Destroy;
    }
    let arms: Vec<Dir> = Dir::ALL
        .into_iter()
        .filter(|&dir| super::connects_to_tube(world, pos, dir))
        .collect();
    if arms.is_empty() {
        return Route::Spill;
    }
    let onward = |dir: &Dir| Some(*dir) != from && takes(world, pos, *dir, kind, stack);
    let mut ways: Vec<Dir> = arms
        .iter()
        .copied()
        .filter(|&dir| kind_allows(world, id, pos, kind, dir, stack))
        .filter(onward)
        .collect();
    if kind == Kind::Sorting && ways.is_empty() {
        let unnamed = super::sorting::unfiltered(world, id);
        ways = arms
            .iter()
            .copied()
            .filter(|dir| unnamed.contains(dir))
            .filter(onward)
            .collect();
    }
    if let Some(dir) = pick(world, &ways) {
        return Route::Out(dir);
    }
    // Back where it came from, if that still leads somewhere.
    match from.filter(|&back| arms.contains(&back) && takes(world, pos, back, kind, stack)) {
        Some(back) => Route::Out(back),
        None => Route::Wait,
    }
}

/// One of `ways` at random.
fn pick(world: &World, ways: &[Dir]) -> Option<Dir> {
    if ways.is_empty() {
        return None;
    }
    ways.get(world.random_below(ways.len() as u32) as usize)
        .copied()
}

/// What a kind of tube lets through toward `dir` on its own terms.
fn kind_allows(world: &World, id: u64, pos: Pos, kind: Kind, dir: Dir, stack: ItemStack) -> bool {
    match kind {
        Kind::Directing => {
            let tube = world.block_typed(pos);
            tube.is(&DIRECTING_TUBE) && tube.prop("out") == Some(dir.name())
        }
        Kind::Sorting => super::sorting::passes(world, id, dir, stack),
        Kind::Stone | Kind::Timber | Kind::Gilded | Kind::Void => true,
    }
}

/// Whether the neighbor toward `dir` takes the item: any tube (a full one makes it wait at the
/// face), or a machine with room. Timber tubes only feed tubes, never back into machines.
fn takes(world: &World, pos: Pos, dir: Dir, kind: Kind, stack: ItemStack) -> bool {
    let other = pos.toward(dir);
    if super::is_tube(world, other) {
        return true;
    }
    kind != Kind::Timber && crate::store::accepts(world, other, stack)
}
