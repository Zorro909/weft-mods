//! Pistons (spec §9.1): what a push moves and breaks, which pistons get in each other's way, and
//! the phases of a move.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Cell, Device, Dir, Kind, Pos, Push, View, step};

/// Blocks one piston moves at most.
pub const PUSH_LIMIT: usize = 12;

/// Where a piston's move stands. A move takes 2 ticks; `since` counts the ticks done.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PistonPhase {
    Retracted,
    Extending { since: u8 },
    Extended,
    Retracting { since: u8, pull: bool },
}

/// What a step asks of the world.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    None,
    /// The move is done: pushed blocks land, and a sticky piston's pulled block with `pull`.
    Land { pull: bool },
    /// Power went while extending: pushed blocks land now, nothing is grabbed.
    Snap,
    /// Retraction starts, grabbing the block in front with `pull`.
    StartRetract { pull: bool },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PushPlan {
    Blocked,
    Move {
        /// The cell the head goes into.
        head: Pos,
        /// The blocks that move one step, the far end first.
        moved: Vec<Pos>,
        /// The block at the end of the line that breaks.
        broken: Vec<Pos>,
        /// The air cell the far end moves into: none when nothing moves or the line ends in a
        /// break.
        reserve: Option<Pos>,
    },
}

/// What a push does with the cell in its way.
enum Rule {
    End,
    Move,
    Break,
    Blocked,
}

fn rule(c: &Cell, resuming: bool) -> Rule {
    match c {
        Cell::Air => Rule::End,
        Cell::Block { push: Push::Reserved, .. } if resuming => Rule::End,
        Cell::Unloaded => Rule::Blocked,
        Cell::Block { push: Push::Move, .. } => Rule::Move,
        Cell::Block { push: Push::Break, .. } => Rule::Break,
        Cell::Block { push: Push::Never | Push::Reserved, .. } => Rule::Blocked,
        // Wire needs the block under it.
        Cell::Wire(_) => Rule::Break,
        Cell::Device(Device { kind: Some(k), .. }) => match k {
            // The mod moves its own devices with what they have pending: piston doors and flying
            // machines need that.
            Kind::PowerBlock | Kind::Lamp { .. } | Kind::Repeater { .. } | Kind::Comparator { .. } | Kind::Piston { phase: PistonPhase::Retracted, .. } => Rule::Move,
            Kind::Piston { .. } => Rule::Blocked,
            // What hangs on a block or lies on it breaks, like dust.
            Kind::Lever { .. } | Kind::Button { .. } | Kind::Plate { .. } | Kind::Torch { .. } => Rule::Break,
        },
        // Other mods' devices: their data can't be moved yet.
        Cell::Device(Device { kind: None, .. }) => Rule::Blocked,
    }
}

/// What the piston at `piston` pushing to `facing` moves, from the head cell on.
pub fn push_plan(v: &impl View, piston: Pos, facing: Dir) -> PushPlan {
    plan(v, piston, facing, false)
}

/// `push_plan` for a piston landing a move it started before a restart: the cells it reserved
/// then are free to it.
pub fn resume_plan(v: &impl View, piston: Pos, facing: Dir) -> PushPlan {
    plan(v, piston, facing, true)
}

fn plan(v: &impl View, piston: Pos, facing: Dir, resuming: bool) -> PushPlan {
    let head = step(piston, facing);
    let mut moved = Vec::new();
    let mut at = head;
    loop {
        match rule(&v.cell(at), resuming) {
            Rule::End => {
                let reserve = (!moved.is_empty()).then_some(at);
                moved.reverse();
                return PushPlan::Move { head, moved, broken: Vec::new(), reserve };
            }
            Rule::Break => {
                moved.reverse();
                return PushPlan::Move { head, moved, broken: vec![at], reserve: None };
            }
            Rule::Blocked => return PushPlan::Blocked,
            Rule::Move => {
                if moved.len() == PUSH_LIMIT {
                    return PushPlan::Blocked;
                }
                moved.push(at);
                at = step(at, facing);
            }
        }
    }
}

/// The block a sticky piston pulls back: the one in front of its head, if it moves.
pub fn pull_plan(v: &impl View, piston: Pos, facing: Dir) -> Option<Pos> {
    let at = step(step(piston, facing), facing);
    matches!(rule(&v.cell(at), false), Rule::Move).then_some(at)
}

/// The pistons whose plans touch a cell another plan touches (the piston itself, its head cell,
/// what moves, where it moves, what breaks): every one of them counts as blocked this tick, so
/// nothing depends on which goes first. A pull is a plan moving one block into the head cell.
pub fn resolve_conflicts(plans: &BTreeMap<Pos, PushPlan>) -> BTreeSet<Pos> {
    let mut by_cell: BTreeMap<Pos, BTreeSet<Pos>> = BTreeMap::new();
    for (piston, plan) in plans {
        let PushPlan::Move { head, moved, broken, reserve } = plan else { continue };
        for c in [*piston, *head].iter().chain(moved).chain(broken).chain(reserve) {
            by_cell.entry(*c).or_default().insert(*piston);
        }
    }
    by_cell.into_values().filter(|s| s.len() > 1).flatten().collect()
}

/// One tick of a piston: its next phase and what the world has to do. `powered` is read from
/// every side but the one it faces; `can_push` is whether its push plan isn't blocked.
pub fn piston_step(phase: PistonPhase, powered: bool, sticky: bool, can_push: bool) -> (PistonPhase, Action) {
    use PistonPhase::*;
    match phase {
        Retracted if powered && can_push => (Extending { since: 0 }, Action::None),
        Retracted => (Retracted, Action::None),
        // The landing tick lands whatever the power does; losing it before snaps.
        Extending { since } if since + 1 >= 2 => (Extended, Action::Land { pull: false }),
        Extending { .. } if !powered => (Retracting { since: 0, pull: false }, Action::Snap),
        Extending { since } => (Extending { since: since + 1 }, Action::None),
        Extended if !powered => (Retracting { since: 0, pull: sticky }, Action::StartRetract { pull: sticky }),
        Extended => (Extended, Action::None),
        // Power coming back finishes the retraction first.
        Retracting { since, pull } if since + 1 >= 2 => (Retracted, Action::Land { pull }),
        Retracting { since, pull } => (Retracting { since: since + 1, pull }, Action::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testgrid::{MapGrid, rotate_pos};
    use crate::{Range, device};

    const P: Pos = [0, 0, 0];

    fn put(g: &mut MapGrid, p: Pos, k: Kind) {
        g.put(p, Cell::Device(device(k, Range::from_index(0))));
    }

    fn line(n: i32) -> MapGrid {
        let mut g = MapGrid::default();
        put(&mut g, P, Kind::Piston { facing: Dir::East, sticky: false, phase: PistonPhase::Retracted });
        for x in 1..=n {
            g.solid([x, 0, 0]);
        }
        g
    }

    #[test]
    fn push_limit_12() {
        let plan = push_plan(&line(12), P, Dir::East);
        let moved: Vec<Pos> = (1..=12).rev().map(|x| [x, 0, 0]).collect();
        assert_eq!(plan, PushPlan::Move { head: [1, 0, 0], moved, broken: vec![], reserve: Some([13, 0, 0]) });
        assert_eq!(push_plan(&line(13), P, Dir::East), PushPlan::Blocked);
        // Nothing in front: only the head moves out.
        assert_eq!(push_plan(&line(0), P, Dir::East), PushPlan::Move { head: [1, 0, 0], moved: vec![], broken: vec![], reserve: None });
    }

    #[test]
    fn unpushable_blocks() {
        let mut g = line(4);
        g.put([3, 0, 0], Cell::Block { solid: true, push: Push::Never });
        assert_eq!(push_plan(&g, P, Dir::East), PushPlan::Blocked);
        // Extended and moving pistons, and other mods' devices, don't move either.
        for phase in [PistonPhase::Extended, PistonPhase::Extending { since: 1 }, PistonPhase::Retracting { since: 0, pull: false }] {
            let mut g = line(4);
            put(&mut g, [2, 0, 0], Kind::Piston { facing: Dir::Up, sticky: true, phase });
            assert_eq!(push_plan(&g, P, Dir::East), PushPlan::Blocked, "{phase:?}");
        }
        let mut g = line(4);
        let mut other = device(Kind::PowerBlock, Range::from_index(0));
        other.kind = None;
        g.put([2, 0, 0], Cell::Device(other));
        assert_eq!(push_plan(&g, P, Dir::East), PushPlan::Blocked);
        // A retracted piston, a lamp, a power block and a repeater move.
        let mut g = line(4);
        put(&mut g, [1, 0, 0], Kind::Piston { facing: Dir::Up, sticky: true, phase: PistonPhase::Retracted });
        put(&mut g, [2, 0, 0], Kind::Lamp { lit: true });
        put(&mut g, [3, 0, 0], Kind::PowerBlock);
        put(&mut g, [4, 0, 0], Kind::Repeater { facing: Dir::East, delay: 1, powered: false, locked: false });
        assert!(matches!(push_plan(&g, P, Dir::East), PushPlan::Move { ref moved, .. } if moved.len() == 4));
        // Reserved cells block, except for the piston resuming its own move.
        let mut g = line(2);
        g.put([3, 0, 0], Cell::Block { solid: false, push: Push::Reserved });
        assert_eq!(push_plan(&g, P, Dir::East), PushPlan::Blocked);
        assert_eq!(resume_plan(&g, P, Dir::East), PushPlan::Move { head: [1, 0, 0], moved: vec![[2, 0, 0], [1, 0, 0]], broken: vec![], reserve: Some([3, 0, 0]) });
    }

    #[test]
    fn break_non_solid_ends_line() {
        let mut g = line(4);
        g.put([3, 0, 0], Cell::block(false));
        assert_eq!(push_plan(&g, P, Dir::East), PushPlan::Move { head: [1, 0, 0], moved: vec![[2, 0, 0], [1, 0, 0]], broken: vec![[3, 0, 0]], reserve: None });
        // Dust and devices that need support break too.
        for c in [Cell::Wire(crate::Wire::dust(0)), Cell::Device(device(Kind::Lever { on: true, attached: Dir::Down }, Range::from_index(0))), Cell::Device(device(Kind::Torch { lit: true, attached: Dir::West }, Range::from_index(0)))] {
            let mut g = line(0);
            g.put([1, 0, 0], c.clone());
            assert_eq!(push_plan(&g, P, Dir::East), PushPlan::Move { head: [1, 0, 0], moved: vec![], broken: vec![[1, 0, 0]], reserve: None }, "{c:?}");
        }
    }

    #[test]
    fn unloaded_blocks_push() {
        let mut g = line(2);
        g.unloaded([3, 0, 0]);
        assert_eq!(push_plan(&g, P, Dir::East), PushPlan::Blocked);
        let mut g = line(0);
        g.unloaded([1, 0, 0]);
        assert_eq!(push_plan(&g, P, Dir::East), PushPlan::Blocked);
    }

    #[test]
    fn pull_takes_only_movable_blocks() {
        let mut g = line(0);
        assert_eq!(pull_plan(&g, P, Dir::East), None);
        g.solid([2, 0, 0]);
        assert_eq!(pull_plan(&g, P, Dir::East), Some([2, 0, 0]));
        g.put([2, 0, 0], Cell::block(false));
        assert_eq!(pull_plan(&g, P, Dir::East), None);
        g.put([2, 0, 0], Cell::Block { solid: true, push: Push::Never });
        assert_eq!(pull_plan(&g, P, Dir::East), None);
        put(&mut g, [2, 0, 0], Kind::Lamp { lit: false });
        assert_eq!(pull_plan(&g, P, Dir::East), Some([2, 0, 0]));
    }

    fn plan_of(g: &MapGrid, p: Pos, facing: Dir) -> PushPlan {
        push_plan(g, p, facing)
    }

    #[test]
    fn conflict_both_blocked() {
        // Two heads into the same cell, and a third piston elsewhere.
        let mut g = MapGrid::default();
        let a = Kind::Piston { facing: Dir::East, sticky: false, phase: PistonPhase::Retracted };
        put(&mut g, [0, 0, 0], a);
        put(&mut g, [2, 0, 0], Kind::Piston { facing: Dir::West, sticky: false, phase: PistonPhase::Retracted });
        put(&mut g, [0, 0, 5], a);
        let plans: BTreeMap<Pos, PushPlan> = [([0, 0, 0], Dir::East), ([2, 0, 0], Dir::West), ([0, 0, 5], Dir::East)].into_iter().map(|(p, d)| (p, plan_of(&g, p, d))).collect();
        assert_eq!(resolve_conflicts(&plans), BTreeSet::from([[0, 0, 0], [2, 0, 0]]));
        // One pushing the other, which moves itself.
        let mut g = MapGrid::default();
        put(&mut g, [0, 0, 0], a);
        put(&mut g, [1, 0, 0], Kind::Piston { facing: Dir::Up, sticky: false, phase: PistonPhase::Retracted });
        let plans: BTreeMap<Pos, PushPlan> = [([0, 0, 0], Dir::East), ([1, 0, 0], Dir::Up)].into_iter().map(|(p, d)| (p, plan_of(&g, p, d))).collect();
        assert_eq!(resolve_conflicts(&plans), BTreeSet::from([[0, 0, 0], [1, 0, 0]]));
    }

    #[test]
    fn conflict_pull_vs_push() {
        // Sticky S pulls [2, 0, 0] west while P pushes it north.
        let mut g = MapGrid::default();
        put(&mut g, [0, 0, 0], Kind::Piston { facing: Dir::East, sticky: true, phase: PistonPhase::Extended });
        g.solid([2, 0, 0]);
        put(&mut g, [2, 0, 1], Kind::Piston { facing: Dir::North, sticky: false, phase: PistonPhase::Retracted });
        let pulled = pull_plan(&g, [0, 0, 0], Dir::East).expect("pulls");
        let mut plans = BTreeMap::new();
        plans.insert([0, 0, 0], PushPlan::Move { head: [1, 0, 0], moved: vec![pulled], broken: vec![], reserve: None });
        plans.insert([2, 0, 1], plan_of(&g, [2, 0, 1], Dir::North));
        assert_eq!(resolve_conflicts(&plans), BTreeSet::from([[0, 0, 0], [2, 0, 1]]));
    }

    /// Steps through `powered` per tick from `start`, collecting the actions.
    fn run(start: PistonPhase, sticky: bool, powered: &[bool]) -> (PistonPhase, Vec<Action>) {
        let mut phase = start;
        let mut actions = Vec::new();
        for &p in powered {
            let (next, a) = piston_step(phase, p, sticky, true);
            phase = next;
            actions.push(a);
        }
        (phase, actions)
    }

    #[test]
    fn extending_takes_two_ticks() {
        let (phase, actions) = run(PistonPhase::Retracted, false, &[true, true, true]);
        assert_eq!(phase, PistonPhase::Extended);
        assert_eq!(actions, [Action::None, Action::None, Action::Land { pull: false }]);
        // Blocked: stays retracted, extends once clear.
        assert_eq!(piston_step(PistonPhase::Retracted, true, false, false), (PistonPhase::Retracted, Action::None));
    }

    #[test]
    fn sticky_one_tick_drops() {
        let (phase, actions) = run(PistonPhase::Retracted, true, &[true, false, false, false]);
        assert_eq!(actions[1], Action::Snap);
        assert_eq!(phase, PistonPhase::Retracted);
        assert!(!actions.iter().any(|a| matches!(a, Action::Land { pull: true } | Action::StartRetract { pull: true })));
    }

    #[test]
    fn sticky_two_ticks_grabs() {
        let (phase, actions) = run(PistonPhase::Retracted, true, &[true, true, false, false, false, false]);
        assert_eq!(actions, [Action::None, Action::None, Action::Land { pull: false }, Action::StartRetract { pull: true }, Action::None, Action::Land { pull: true }]);
        assert_eq!(phase, PistonPhase::Retracted);
        // A normal piston never pulls.
        let (_, actions) = run(PistonPhase::Retracted, false, &[true, true, false, false, false, false]);
        assert_eq!(actions[3], Action::StartRetract { pull: false });
    }

    #[test]
    fn pull_without_push_needs_two_ticks() {
        // Into empty space: one tick of power snaps back without grabbing, two grab.
        let (_, one) = run(PistonPhase::Retracted, true, &[true, false, false, false]);
        assert!(one.contains(&Action::Snap));
        let (_, two) = run(PistonPhase::Retracted, true, &[true, true, false, false, false, false]);
        assert!(two.contains(&Action::StartRetract { pull: true }));
    }

    #[test]
    fn repower_while_retracting() {
        let (phase, actions) = run(PistonPhase::Extended, true, &[false, true, true, true]);
        assert_eq!(actions, [Action::StartRetract { pull: true }, Action::None, Action::Land { pull: true }, Action::None]);
        assert_eq!(phase, PistonPhase::Extending { since: 0 });
    }

    fn rotate_plan(p: &PushPlan) -> PushPlan {
        match p {
            PushPlan::Blocked => PushPlan::Blocked,
            PushPlan::Move { head, moved, broken, reserve } => PushPlan::Move {
                head: rotate_pos(*head),
                moved: moved.iter().map(|p| rotate_pos(*p)).collect(),
                broken: broken.iter().map(|p| rotate_pos(*p)).collect(),
                reserve: reserve.map(rotate_pos),
            },
        }
    }

    #[test]
    fn push_order_rotation_invariant() {
        let mut g = line(5);
        g.put([4, 0, 0], Cell::block(false));
        let mut g2 = line(3);
        g2.solid([1, 0, 1]);
        for g in [g, g2] {
            let mut grid = g.clone();
            let mut facing = Dir::East;
            let mut want = push_plan(&grid, P, facing);
            for _ in 0..4 {
                grid = grid.rotated();
                facing = facing.rotate_y();
                want = rotate_plan(&want);
                assert_eq!(push_plan(&grid, P, facing), want);
            }
        }
    }
}
