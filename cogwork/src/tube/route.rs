//! Where an item goes from a tube's hub: any arm that can take it but the one it came by, as
//! each kind of tube allows. Among several, one at random, so items spread over branches.

use weft_sdk::*;

use super::{DIRECTING_TUBE, kind};
use crate::lib::{BlockState, Pos};

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

/// `from` is the side the item came in by, `None` for one put in at the hub.
pub fn route(
    w: &World,
    id: u64,
    pos: Pos,
    tube_kind: u32,
    from: Option<Dir>,
    stack: ItemStack,
) -> Route {
    if tube_kind == kind::VOID {
        return Route::Destroy;
    }
    let arms: Vec<Dir> = Dir::ALL
        .into_iter()
        .filter(|&d| super::connects_to_tube(w, pos, d))
        .collect();
    if arms.is_empty() {
        return Route::Spill;
    }
    let allowed = |d: Dir| Some(d) != from && kind_allows(w, id, pos, tube_kind, d, stack);
    let mut ways: Vec<Dir> = arms
        .iter()
        .copied()
        .filter(|&d| allowed(d) && takes(w, pos, d, tube_kind, stack))
        .collect();
    if tube_kind == kind::SORTING && ways.is_empty() {
        ways = super::sorting::unfiltered(w, id)
            .into_iter()
            .filter(|&d| Some(d) != from && arms.contains(&d) && takes(w, pos, d, tube_kind, stack))
            .collect();
    }
    if let Some(&d) = pick(w, &ways) {
        return Route::Out(d);
    }
    // Back where it came from, if that still leads somewhere.
    if let Some(back) = from
        && arms.contains(&back)
        && takes(w, pos, back, tube_kind, stack)
    {
        return Route::Out(back);
    }
    Route::Wait
}

fn pick<'a>(w: &World, ways: &'a [Dir]) -> Option<&'a Dir> {
    if ways.is_empty() {
        None
    } else {
        ways.get(w.random_below(ways.len() as u32) as usize)
    }
}

/// What a kind of tube lets through toward `d` on its own terms.
fn kind_allows(w: &World, id: u64, pos: Pos, tube_kind: u32, d: Dir, stack: ItemStack) -> bool {
    match tube_kind {
        kind::DIRECTING => {
            let tube = BlockState::at(w, pos);
            tube.is(&DIRECTING_TUBE) && tube.prop("out") == Some(d.name())
        }
        kind::SORTING => super::sorting::passes(w, id, d, stack),
        _ => true,
    }
}

/// Whether the neighbor toward `d` takes the item: any tube (a full one makes it wait at the
/// face), or a machine with room. Timber tubes only feed tubes, never back into machines.
fn takes(w: &World, pos: Pos, d: Dir, tube_kind: u32, stack: ItemStack) -> bool {
    let other = pos.toward(d);
    if super::is_tube(w, other) {
        return true;
    }
    tube_kind != kind::TIMBER && crate::store::accepts(w, other, stack)
}
