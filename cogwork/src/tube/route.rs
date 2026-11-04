//! Where an item goes from a tube's hub: any arm that can take it but the one it came by, as
//! each kind of tube allows. Among several, one at random, so items spread over branches.

use weft_sdk::*;

use super::{DIRECTING_TUBE, NONE, kind};
use crate::lib::{BlockState, Pos};

pub enum Route {
    /// On through this face.
    Out(u32),
    /// Nowhere to go for now: it waits at the hub.
    Wait,
    /// A void tube: gone.
    Destroy,
    /// No arms at all: it falls out.
    Spill,
}

pub fn route(w: &World, id: u64, pos: Pos, tube_kind: u32, from: u32, stack: ItemStack) -> Route {
    if tube_kind == kind::VOID {
        return Route::Destroy;
    }
    let arms: Vec<u32> = (0..6)
        .filter(|&f| super::connects_to_tube(w, pos, f))
        .collect();
    if arms.is_empty() {
        return Route::Spill;
    }
    let allowed = |f: u32| f != from && kind_allows(w, id, pos, tube_kind, f, stack);
    let mut ways: Vec<u32> = arms
        .iter()
        .copied()
        .filter(|&f| allowed(f) && takes(w, pos, f, tube_kind, stack))
        .collect();
    if tube_kind == kind::SORTING && ways.is_empty() {
        ways = super::sorting::unfiltered(w, id)
            .into_iter()
            .filter(|&f| f != from && arms.contains(&f) && takes(w, pos, f, tube_kind, stack))
            .collect();
    }
    if let Some(&face) = pick(w, &ways) {
        return Route::Out(face);
    }
    // Back where it came from, if that still leads somewhere.
    if from != NONE && arms.contains(&from) && takes(w, pos, from, tube_kind, stack) {
        return Route::Out(from);
    }
    Route::Wait
}

fn pick<'a>(w: &World, ways: &'a [u32]) -> Option<&'a u32> {
    if ways.is_empty() {
        None
    } else {
        ways.get(w.random_below(ways.len() as u32) as usize)
    }
}

/// What a kind of tube lets through `face` on its own terms.
fn kind_allows(w: &World, id: u64, pos: Pos, tube_kind: u32, face: u32, stack: ItemStack) -> bool {
    match tube_kind {
        kind::DIRECTING => {
            let tube = BlockState::at(w, pos);
            tube.is(&DIRECTING_TUBE) && tube.prop("out") == Some(Dir::from_face(face).name())
        }
        kind::SORTING => super::sorting::passes(w, id, face, stack),
        _ => true,
    }
}

/// Whether the neighbor through `face` takes the item: any tube (a full one makes it wait at
/// the face), or a machine with room. Timber tubes only feed tubes, never back into machines.
fn takes(w: &World, pos: Pos, face: u32, tube_kind: u32, stack: ItemStack) -> bool {
    let other = pos.neighbor(face);
    if super::is_tube(w, other) {
        return true;
    }
    tube_kind != kind::TIMBER && crate::store::accepts(w, other, stack)
}
