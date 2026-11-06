//! What excavators and levelers keep loaded while they work: their own cubes, those they work
//! in, and those between.

use weft_sdk::*;

use super::tickets::{self, Keeper};
use super::{CubePos, Cubes, ROLES, Role, key};
use crate::lib::Pos;

/// Keeps what a working machine at `pos` needs loaded: its own cube and the eight around it,
/// and the cubes it works in, `work`, when they reach outside them.
///
/// A machine's `work` should not let go of a cube it comes back to later: unloaded, a cube is
/// written to disk at once, and read back from it when wanted again.
pub fn keep_machine(w: &World, pos: Pos, work: Option<Cubes>) {
    let home = Cubes::around(pos, 1);
    tickets::keep(w, key(w, pos, Role::Home), home, Keeper::Machine);
    let work = work.filter(|cubes| !home.contains(cubes));
    keep_or_release(w, key(w, pos, Role::Work), work);
    keep_or_release(w, key(w, pos, Role::Link), work.and_then(|c| link(pos, c)));
}

/// Lets go of what the machine at `pos` kept loaded: it is done.
pub fn release_machine(w: &World, pos: Pos) {
    for role in ROLES {
        tickets::release(w, key(w, pos, role));
    }
}

/// The column of cubes straight between the machine at `pos` and its `work`, if any. A machine
/// reaches only its own region, and a gap of 64 blocks or more with nothing loaded puts its
/// work in another one.
fn link(pos: Pos, work: Cubes) -> Option<Cubes> {
    let own = CubePos::of(pos);
    let (lo, hi) = if work.lo().y > own.y {
        (own.y + 1, work.lo().y - 1)
    } else {
        (work.hi().y + 1, own.y - 1)
    };
    (lo <= hi).then(|| Cubes::between(own.with_y(lo), own.with_y(hi)))
}

fn keep_or_release(w: &World, key: u64, cubes: Option<Cubes>) {
    match cubes {
        Some(cubes) => drop(tickets::keep(w, key, cubes, Keeper::Machine)),
        None => tickets::release(w, key),
    }
}

/// For machines' tests: the first cube that `kept`, the boxes a machine keeps one after another,
/// lets go of and later keeps again.
#[cfg(test)]
pub fn kept_again(kept: impl IntoIterator<Item = Cubes>) -> Option<CubePos> {
    use std::collections::HashSet;
    let (mut held, mut gone): (HashSet<CubePos>, HashSet<CubePos>) = Default::default();
    let mut last = None;
    for cubes in kept {
        if last == Some(cubes) {
            continue;
        }
        last = Some(cubes);
        if let Some(back) = cubes.each().find(|cube| gone.contains(cube)) {
            return Some(back);
        }
        let now: HashSet<CubePos> = cubes.each().collect();
        gone.extend(held.difference(&now));
        held = now;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_far_below_is_linked_to_the_machine_by_the_cubes_between() {
        let pos = Pos::new(5, 320, 5);
        let below = Cubes::around(Pos::new(30, 60, 30), 2);
        let column = link(pos, below).expect("cubes between");
        assert_eq!(
            column.lo(),
            CubePos::new(0, 4, 0),
            "from just above the work"
        );
        assert_eq!(
            column.hi(),
            CubePos::new(0, 19, 0),
            "to just below the machine"
        );
        let above = Cubes::around(Pos::new(30, 400, 30), 0);
        assert_eq!(
            link(pos, above).map(|c| c.lo().y),
            Some(21),
            "work above it"
        );
        let beside = Cubes::around(Pos::new(30, 330, 30), 2);
        assert_eq!(link(pos, beside), None, "at its height");
        let next_cube_down = Cubes::around(Pos::new(30, 310, 30), 2);
        assert_eq!(link(pos, next_cube_down), None, "joined already");
    }

    #[test]
    fn a_cube_let_go_and_kept_again_is_found() {
        let layer = |y| Cubes::spanning(Pos::new(0, y, 0), Pos::new(40, y, 40));
        let down = [70, 70, 60, 50, 40].map(layer);
        assert_eq!(kept_again(down), None, "one layer after another");
        let back = [70, 60, 70].map(layer);
        assert_eq!(kept_again(back), Some(CubePos::new(0, 4, 0)), "back up");
    }
}
