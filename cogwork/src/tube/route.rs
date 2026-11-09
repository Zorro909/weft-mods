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
    w: &World,
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
        .filter(|&d| super::connects_to_tube(w, pos, d))
        .collect();
    if arms.is_empty() {
        return Route::Spill;
    }
    let onward = |d: &Dir| Some(*d) != from && takes(w, pos, *d, kind, stack);
    let mut ways: Vec<Dir> = arms
        .iter()
        .copied()
        .filter(|&d| kind_allows(w, id, pos, kind, d, stack))
        .filter(onward)
        .collect();
    if kind == Kind::Sorting && ways.is_empty() {
        let unnamed = super::sorting::unfiltered(w, id);
        ways = arms
            .iter()
            .copied()
            .filter(|d| unnamed.contains(d))
            .filter(onward)
            .collect();
    }
    if let Some(d) = pick(w, &ways) {
        return Route::Out(d);
    }
    // Back where it came from, if that still leads somewhere.
    match from.filter(|&back| arms.contains(&back) && takes(w, pos, back, kind, stack)) {
        Some(back) => Route::Out(back),
        None => Route::Wait,
    }
}

/// One of `ways` at random.
fn pick(w: &World, ways: &[Dir]) -> Option<Dir> {
    if ways.is_empty() {
        return None;
    }
    ways.get(w.random_below(ways.len() as u32) as usize)
        .copied()
}

/// What a kind of tube lets through toward `d` on its own terms.
fn kind_allows(w: &World, id: u64, pos: Pos, kind: Kind, d: Dir, stack: ItemStack) -> bool {
    match kind {
        Kind::Directing => {
            let tube = w.block_typed(pos);
            tube.is(&DIRECTING_TUBE) && tube.prop("out") == Some(d.name())
        }
        Kind::Sorting => super::sorting::passes(w, id, d, stack),
        Kind::Stone | Kind::Timber | Kind::Gilded | Kind::Void => true,
    }
}

/// Whether the neighbor toward `d` takes the item: any tube (a full one makes it wait at the
/// face), or a machine with room. Timber tubes only feed tubes, never back into machines.
fn takes(w: &World, pos: Pos, d: Dir, kind: Kind, stack: ItemStack) -> bool {
    let other = pos.toward(d);
    if super::is_tube(w, other) {
        return true;
    }
    kind != Kind::Timber && crate::store::accepts(w, other, stack)
}
