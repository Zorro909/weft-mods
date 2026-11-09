//! What excavators and levelers keep loaded while they work: their own cubes, those they work
//! in, and those that join the two.

use weft_sdk::*;

use super::tickets::{self, Keeper};
use super::{CubePos, Cubes, Role, key};
use crate::lib::Pos;

/// Cubes along each side of a section. A machine reaches only its own region: the loaded cubes
/// joined to its own through sections that touch, a section counting once a cube of it is
/// loaded. Two cubes at most this far apart lie in touching sections.
const SECTION: i32 = 4;

/// Keeps what a working machine at `pos` needs loaded: its own cube and the eight around it,
/// the cubes it works in, `work`, when they reach outside them, and the cubes that join them.
///
/// A machine's `work` should not let go of a cube it comes back to later: unloaded, a cube is
/// written to disk at once, and read back from it when wanted again.
pub fn keep_machine(world: &World, pos: Pos, work: Option<Cubes>) {
    let home = Cubes::around(pos, 1);
    tickets::keep(world, key(world, pos, Role::Home), home, Keeper::Machine);
    let work = work.filter(|cubes| !home.contains(cubes));
    keep_or_release(world, key(world, pos, Role::Work), work);
    let links = work.map(|cubes| links(pos, cubes)).unwrap_or_default();
    for (i, &cube) in links.iter().enumerate() {
        let link = Some(Cubes::between(cube, cube));
        keep_or_release(world, key(world, pos, Role::Link(i as u32)), link);
    }
    release_links(world, pos, links.len());
}

/// Lets go of what the machine at `pos` kept loaded: it is done.
pub fn release_machine(world: &World, pos: Pos) {
    tickets::release(world, key(world, pos, Role::Home));
    tickets::release(world, key(world, pos, Role::Work));
    release_links(world, pos, 0);
}

/// Lets go of the links of the machine at `pos` from the `from`th on, up to the first cogwork
/// doesn't hold. Any after that, no longer renewed, runs out on its own.
pub(super) fn release_links(world: &World, pos: Pos, from: usize) {
    let mut i = from as u32;
    while tickets::release(world, key(world, pos, Role::Link(i))) {
        i += 1;
    }
}

/// The cubes that join the machine at `pos` to its `work` above or below it: one in every
/// `SECTION` straight up or down from its own. Sideways it needs none, as a machine's area
/// always starts beside it, within a cube of its own.
fn links(pos: Pos, work: Cubes) -> Vec<CubePos> {
    let own = CubePos::of(pos);
    let (edge, step) = if work.lo().y > own.y {
        (work.lo().y, SECTION)
    } else {
        (work.hi().y, -SECTION)
    };
    let mut links = Vec::new();
    let mut last = own.y;
    while (edge - last).abs() > SECTION {
        last += step;
        links.push(own.with_y(last));
    }
    links
}

fn keep_or_release(world: &World, key: u64, cubes: Option<Cubes>) {
    match cubes {
        Some(cubes) => drop(tickets::keep(world, key, cubes, Keeper::Machine)),
        None => drop(tickets::release(world, key)),
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

    fn heights(cubes: &[CubePos]) -> Vec<i32> {
        cubes.iter().map(|cube| cube.y).collect()
    }

    #[test]
    fn work_far_below_is_joined_by_a_cube_in_every_four() {
        let pos = Pos::new(5, 320, 5);
        let below = Cubes::around(Pos::new(30, 60, 30), 2);
        let joining = links(pos, below);
        assert_eq!(heights(&joining), [16, 12, 8, 4], "from cube 20 down to 3");
        assert!(
            joining.iter().all(|cube| cube.x == 0 && cube.z == 0),
            "its own column"
        );
        let above = Cubes::around(Pos::new(30, 400, 30), 0);
        assert_eq!(heights(&links(pos, above)), [24], "up to cube 25");
        let near = Cubes::around(Pos::new(30, 256, 30), 2);
        assert_eq!(links(pos, near), [], "four cubes down: touching");
    }

    #[test]
    fn each_cube_of_the_chain_lies_in_a_section_touching_the_next() {
        let section = |y: i32| y.div_euclid(SECTION);
        let pos = Pos::new(0, 600, 0);
        for work_y in -64..1200 {
            let work = Cubes::around(Pos::new(0, work_y, 0), 0);
            let mut chain = vec![CubePos::of(pos).y];
            chain.extend(heights(&links(pos, work)));
            chain.push(work.lo().y);
            let touching = chain
                .windows(2)
                .all(|pair| (section(pair[0]) - section(pair[1])).abs() <= 1);
            assert!(touching, "{chain:?} for work at {work_y}");
        }
    }

    #[test]
    fn links_only_grow_as_work_goes_down() {
        let pos = Pos::new(0, 300, 0);
        let mut held = Vec::new();
        for work_y in (-64..300).rev() {
            let now = links(pos, Cubes::around(Pos::new(0, work_y, 0), 0));
            assert_eq!(now[..held.len()], held[..], "nothing let go at {work_y}");
            held = now;
        }
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
