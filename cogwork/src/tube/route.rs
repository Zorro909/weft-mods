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
    let takes = |dir| takes(world, pos, dir, kind, stack);
    let allowed = |dir| kind_allows(world, id, pos, kind, dir, stack);
    // A sorting tube's sides without a filter take what no filter names.
    let spare = || match kind {
        Kind::Sorting => super::sorting::unfiltered(world, id),
        _ => Vec::new(),
    };
    let ways = onward(&arms, from, takes, allowed, spare);
    if let Some(dir) = pick(world, &ways) {
        return Route::Out(dir);
    }
    // Back where it came from, if that still leads somewhere.
    match back(&arms, from, takes) {
        Some(back) => Route::Out(back),
        None => Route::Wait,
    }
}

/// The `arms` an item that came by `from` may go on by: those that take it (`takes`) among the
/// ones its tube sends it to (`allowed`), or among the `spare` ones when there are none of
/// those. Never the arm it came by.
fn onward(
    arms: &[Dir],
    from: Option<Dir>,
    takes: impl Fn(Dir) -> bool,
    allowed: impl Fn(Dir) -> bool,
    spare: impl FnOnce() -> Vec<Dir>,
) -> Vec<Dir> {
    let open = |dir: &Dir| Some(*dir) != from && takes(*dir);
    let sent: Vec<Dir> = arms
        .iter()
        .copied()
        .filter(|&dir| allowed(dir))
        .filter(open)
        .collect();
    if !sent.is_empty() {
        return sent;
    }
    let spare = spare();
    arms.iter()
        .copied()
        .filter(|dir| spare.contains(dir))
        .filter(open)
        .collect()
}

/// The arm an item came by, if it is still there and takes the item back.
fn back(arms: &[Dir], from: Option<Dir>, takes: impl Fn(Dir) -> bool) -> Option<Dir> {
    from.filter(|&back| arms.contains(&back) && takes(back))
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

#[cfg(test)]
mod tests {
    use super::*;

    const ARMS: [Dir; 3] = [Dir::West, Dir::East, Dir::Up];

    fn none() -> Vec<Dir> {
        Vec::new()
    }

    #[test]
    fn an_item_goes_on_by_any_arm_but_the_one_it_came_by() {
        let ways = onward(&ARMS, Some(Dir::West), |_| true, |_| true, none);
        assert_eq!(ways, [Dir::East, Dir::Up], "not back west");
        let put_in = onward(&ARMS, None, |_| true, |_| true, none);
        assert_eq!(put_in, ARMS, "put in at the hub: any arm");
    }

    #[test]
    fn it_goes_only_where_the_tube_sends_it_and_it_is_taken() {
        let ways = onward(&ARMS, None, |dir| dir != Dir::East, |_| true, none);
        assert_eq!(ways, [Dir::West, Dir::Up], "east takes nothing");
        let ways = onward(&ARMS, None, |_| true, |dir| dir == Dir::Up, none);
        assert_eq!(ways, [Dir::Up], "the tube sends it up");
    }

    #[test]
    fn the_spare_arms_take_what_the_tube_sends_nowhere() {
        let spare = || vec![Dir::East, Dir::North];
        let ways = onward(&ARMS, None, |_| true, |_| false, spare);
        assert_eq!(ways, [Dir::East], "the spare arm it has");
        let sent = onward(&ARMS, None, |_| true, |dir| dir == Dir::Up, spare);
        assert_eq!(sent, [Dir::Up], "sent somewhere: not by a spare arm");
        let came_by = onward(&ARMS, Some(Dir::East), |_| true, |_| false, spare);
        assert!(came_by.is_empty(), "not back by a spare arm either");
    }

    #[test]
    fn it_turns_back_only_into_an_arm_that_takes_it() {
        let west = Some(Dir::West);
        assert_eq!(back(&ARMS, west, |_| true), west, "back west");
        assert_eq!(back(&ARMS, west, |_| false), None, "west is full");
        assert_eq!(back(&ARMS, Some(Dir::North), |_| true), None, "no arm");
        assert_eq!(back(&ARMS, None, |_| true), None, "it came by none");
    }
}
