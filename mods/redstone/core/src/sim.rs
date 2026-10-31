//! The tick (spec §6, §7): apply due device changes at once, solve the wire networks they touch,
//! write each changed wire once, then let receivers react for a later tick. Everything iterates in
//! position order, so the same build gives the same result wherever it is and in whatever order it
//! was changed. Circuits reaching into unloaded cubes freeze (§7.1).

use std::cell::{Cell as StdCell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::mem::discriminant;

use crate::devices::sides_of;
use crate::network::Recorder;
use crate::{Action, Cell, Device, Dir, Kind, PistonPhase, Pos, PushPlan, Range, Shape, SidePower, View, apply, cube_of, inputs, links, network, piston_step, pull_plan, push_plan, react, resolve_conflicts, resume_plan, shape, solve, step};

/// A device change counting down.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pending {
    pub next: Kind,
    pub remaining: u32,
}

/// The world the tick reads and writes. Writes must be visible to reads right away.
pub trait World: View {
    fn range(&self) -> Range;
    fn cube_loaded(&self, cube: [i32; 3]) -> bool;
    /// The comparator reading of what's behind a comparator at `p`.
    fn container(&self, p: Pos) -> Option<u8>;
    /// Which device is at `p`, as an id no other device in any world has, 0 if it has none. A
    /// pending change applies only to the device it started on.
    fn instance(&self, p: Pos) -> u64;
    /// A wire's strength and shape after this tick, for every wire of every network the tick
    /// solved. The world shows what it can (a brightness level) and skips writes that change
    /// nothing it shows.
    fn write_wire(&mut self, p: Pos, strength: u8, shape: Shape);
    fn write_device(&mut self, p: Pos, k: Kind);
    /// What reaches an extension receiver changed.
    fn notify(&mut self, p: Pos, inputs: [SidePower; 6]);
    /// Stores (or clears) a device's pending change so it survives unload and restart; `true` when
    /// it is frozen and not counting down. Unless frozen it applies `remaining` ticks after the
    /// tick it was saved in.
    fn save_pending(&mut self, p: Pos, pending: Option<(Pending, bool)>);
    /// Holds the air cell `p` free for a block moving into it: an invisible solid placeholder
    /// that reads as `Push::Reserved`. False if it can't be written there.
    fn reserve(&mut self, p: Pos) -> bool;
    /// A piston move lands: breaks `broken` with drops, then moves each of `moved` (the far end
    /// first) one step `to`, with what the mod keeps for its own devices. Entities in the cells
    /// the blocks move into, and in `head` (the piston head's cell), move the same step.
    fn shift(&mut self, to: Dir, moved: &[Pos], broken: &[Pos], head: Option<Pos>);
    /// Puts a piston head (its facing and whether it is sticky) into `p`, or with `None` takes
    /// away the head or placeholder there, and nothing else.
    fn head(&mut self, p: Pos, head: Option<(Dir, bool)>);
}

#[derive(Default)]
pub struct Sim {
    /// By position and `World::instance`: the same place in two dimensions are two devices.
    pending: BTreeMap<(Pos, u64), Entry>,
    dirty: BTreeSet<Pos>,
    /// Positions in a frozen circuit, retried when their countdown runs out.
    frozen_dirty: BTreeMap<Pos, u32>,
    /// What extension receivers were last told, only non-zero ones.
    last_inputs: BTreeMap<Pos, [SidePower; 6]>,
    /// Comparators reading a container, which react every tick: nothing says when a container
    /// changes.
    readers: BTreeSet<Pos>,
    /// Pistons that are moving, powered but blocked, or about to retract, keyed like `pending`.
    pistons: BTreeMap<(Pos, u64), PistonRun>,
}

struct Entry {
    pending: Pending,
    /// The cubes its circuit spans; `None` until the next tick works them out.
    cubes: Option<BTreeSet<[i32; 3]>>,
    frozen: bool,
    /// Scheduled from outside the tick: the first tick that sees it doesn't count it down.
    fresh: bool,
}

/// A piston's move, in phases of `piston_step`. Its pending change is the phase.
struct PistonRun {
    phase: PistonPhase,
    powered: bool,
    /// The cells the move under way touches: no other piston starts a move through them.
    busy: Vec<Pos>,
    /// The placeholders it put into the world.
    reserved: Vec<Pos>,
}

impl PistonRun {
    fn new(phase: PistonPhase, powered: bool) -> PistonRun {
        PistonRun { phase, powered, busy: Vec::new(), reserved: Vec::new() }
    }

    /// Nothing to do until its power changes.
    fn idle(&self) -> bool {
        matches!((self.phase, self.powered), (PistonPhase::Retracted, false) | (PistonPhase::Extended, true))
    }
}

impl Sim {
    /// Something at `p` changed: a device, a block, a wire.
    pub fn touch(&mut self, p: Pos) {
        self.dirty.insert(p);
    }

    /// Starts (or replaces) the pending change of the device at `p`. The next tick saves it, and
    /// it applies `remaining` ticks after that one.
    /// A piston's change is the phase it is in, and the tick continues its move from there.
    pub fn schedule(&mut self, p: Pos, pending: Pending, owner: u64) {
        if let Kind::Piston { phase, .. } = pending.next {
            // Its power is read again when something next to it changes.
            let powered = matches!(phase, PistonPhase::Extending { .. });
            self.pistons.insert((p, owner), PistonRun::new(phase, powered));
            return;
        }
        self.pending.insert((p, owner), Entry { pending, cubes: None, frozen: false, fresh: true });
    }

    /// The device at `p` is gone. Returns the placeholders a piston there had put into the
    /// world, which nothing will fill now.
    pub fn forget(&mut self, p: Pos) -> Vec<Pos> {
        let at: Vec<_> = self.pending.range((p, 0)..=(p, u64::MAX)).map(|(k, _)| *k).collect();
        for k in at {
            self.pending.remove(&k);
        }
        self.frozen_dirty.remove(&p);
        self.last_inputs.remove(&p);
        self.readers.remove(&p);
        let runs: Vec<_> = self.pistons.range((p, 0)..=(p, u64::MAX)).map(|(k, _)| *k).collect();
        runs.into_iter().filter_map(|k| self.pistons.remove(&k)).flat_map(|r| r.reserved).collect()
    }

    pub fn has_pending(&self, p: Pos, owner: u64) -> bool {
        self.pending.contains_key(&(p, owner)) || self.pistons.contains_key(&(p, owner))
    }

    pub fn tick(&mut self, w: &mut impl World) {
        let range = w.range();

        // 1. Only changes whose device is in this world count: every region and dimension ticks
        // the same Sim (G6), and a change belongs to the device instance it started on, so the
        // same place in another dimension is another device. One that was removed or replaced is
        // simply never seen again (`forget` cleans up). A change whose circuit reaches an
        // unloaded cube stops counting down.
        let mut due = Vec::new();
        for (&(p, owner), e) in &mut self.pending {
            let here = matches!(w.cell(p), Cell::Device(Device { kind: Some(k), .. }) if discriminant(&k) == discriminant(&e.pending.next))
                && w.instance(p) == owner;
            if !here {
                continue;
            }
            // New here: scheduled, restored or pushed by a piston.
            let located = e.cubes.is_none();
            let cubes = e.cubes.get_or_insert_with(|| reach(&*w, p));
            let frozen = !cubes.iter().all(|c| w.cube_loaded(*c));
            // A scheduled change starts counting in the tick after the first one that sees it.
            if !frozen && !e.fresh {
                e.pending.remaining = e.pending.remaining.saturating_sub(1);
            }
            e.fresh = false;
            if e.pending.remaining == 0 {
                due.push((p, owner));
            } else if located || !frozen || frozen != e.frozen {
                // Saved as counted down, like a fresh change: `remaining` ticks after this one.
                // Every tick it counts: a restart continues from what is left, however long
                // its cube was unloaded since.
                w.save_pending(p, Some((e.pending, frozen)));
            }
            e.frozen = frozen;
        }

        // 2. Apply every change due now, all at once.
        for (p, owner) in due {
            let e = self.pending.remove(&(p, owner)).expect("due entry");
            let next = match w.cell(p) {
                Cell::Device(Device { kind: Some(k), .. }) => apply(k, e.pending.next),
                _ => e.pending.next,
            };
            w.write_device(p, next);
            w.save_pending(p, None);
            self.dirty.insert(p);
        }
        self.step_pistons(w);
        // A frozen circuit's retry waits for a world that has redstone there.
        self.frozen_dirty.retain(|p, n| {
            *n = n.saturating_sub(1);
            if *n > 0 {
                return true;
            }
            if matches!(w.cell(*p), Cell::Wire(_) | Cell::Device(_)) {
                self.dirty.insert(*p);
                return false;
            }
            true
        });

        // 3 + 4. Solve each network next to a change once and hand every wire to the world. Wire
        // strength isn't stored, so every receiver such a network reaches is a candidate.
        let mut seeds = BTreeSet::new();
        for p in &self.dirty {
            around(&*w, *p, &mut seeds);
        }
        let mut solved = BTreeMap::new();
        let mut candidates = BTreeSet::new();
        for s in seeds {
            if solved.contains_key(&s) || !matches!(w.cell(s), Cell::Wire(_)) {
                continue;
            }
            let net = network(&*w, s);
            if net.incomplete {
                solved.extend(net.wires.iter().map(|p| (*p, None)));
                self.frozen_dirty.entry(s).or_insert(RETRY);
                continue;
            }
            candidates.extend(net.devices.iter().copied());
            for (p, strength) in solve(&*w, &net, range) {
                solved.insert(p, Some(strength));
                let Cell::Wire(wire) = w.cell(p) else { continue };
                let sh = shape(links(&*w, p, &wire));
                w.write_wire(p, strength, sh);
            }
        }

        // 5. Receivers whose inputs may have changed react, for a later tick.
        for p in &self.dirty {
            around(&*w, *p, &mut candidates);
        }
        candidates.extend(self.readers.iter().copied());
        let solved = RefCell::new(solved);
        for p in candidates {
            let Cell::Device(dev) = w.cell(p) else { continue };
            if !dev.receives {
                continue;
            }
            let (i, frozen) = {
                let v = Solved { w: &*w, strengths: &solved, range, frozen: StdCell::new(false) };
                let r = Recorder::new(&v);
                let i = inputs(&r, p);
                (i, r.unloaded.get() || v.frozen.get())
            };
            if frozen {
                self.frozen_dirty.entry(p).or_insert(RETRY);
                continue;
            }
            match dev.kind {
                Some(Kind::Piston { facing, phase, .. }) => {
                    // Not from the face it pushes with (spec §9.1).
                    let powered = Dir::ALL.iter().any(|d| *d != facing && i[d.index()].strength > 0);
                    let run = self.pistons.entry((p, w.instance(p))).or_insert_with(|| PistonRun::new(phase, powered));
                    run.powered = powered;
                }
                Some(k) => self.react(w, p, k, i, range),
                None => {
                    let last = self.last_inputs.get(&p).copied().unwrap_or_default();
                    if i != last {
                        w.notify(p, i);
                        if i == <[SidePower; 6]>::default() {
                            self.last_inputs.remove(&p);
                        } else {
                            self.last_inputs.insert(p, i);
                        }
                    }
                }
            }
        }

        // 6.
        self.dirty.clear();
        self.pistons.retain(|_, r| !r.idle());
    }

    /// Every piston in this world steps at once (spec §9.1): moves start where no other piston
    /// wants the same cells, and land at the end of their second tick.
    fn step_pistons(&mut self, w: &mut impl World) {
        let mut here = Vec::new();
        for &(p, owner) in self.pistons.keys() {
            if let Cell::Device(Device { kind: Some(Kind::Piston { facing, sticky, .. }), .. }) = w.cell(p)
                && w.instance(p) == owner
            {
                here.push((p, owner, facing, sticky));
            }
        }
        // What would start a move this tick: pushes, and sticky pulls.
        let mut busy = BTreeSet::new();
        let mut plans = BTreeMap::new();
        for &(p, owner, facing, sticky) in &here {
            let r = &self.pistons[&(p, owner)];
            busy.extend(r.busy.iter().copied());
            let plan = match r.phase {
                PistonPhase::Retracted if r.powered => push_plan(&*w, p, facing),
                PistonPhase::Extended if !r.powered && sticky => match pull_plan(&*w, p, facing) {
                    Some(b) => PushPlan::Move { head: step(p, facing), moved: vec![b], broken: Vec::new(), reserve: None },
                    None => continue,
                },
                _ => continue,
            };
            plans.insert(p, plan);
        }
        let mut blocked = resolve_conflicts(&plans);
        for (p, plan) in &plans {
            if let PushPlan::Move { head, moved, broken, reserve } = plan
                && [*p, *head].iter().chain(moved).chain(broken).chain(reserve).any(|c| busy.contains(c))
            {
                blocked.insert(*p);
            }
        }

        for (p, owner, facing, sticky) in here {
            // A piston landed earlier in this loop may have pushed this one.
            let Some(&PistonRun { phase, powered, .. }) = self.pistons.get(&(p, owner)) else { continue };
            let plan = plans.get(&p).filter(|_| !blocked.contains(&p));
            // A blocked pull waits extended, like a blocked push waits retracted.
            if phase == PistonPhase::Extended && plans.contains_key(&p) && plan.is_none() {
                continue;
            }
            // A sticky piston with nothing to grab when it starts back grabs nothing: a block
            // pushed in front of it meanwhile would otherwise be pulled or not depending on which
            // piston goes first.
            let grabs = sticky && plan.is_some();
            let (next, action) = piston_step(phase, powered, grabs, matches!(plan, Some(PushPlan::Move { .. })));
            let head = step(p, facing);
            let mut busy = Vec::new();
            let mut reserved = Vec::new();
            match (phase, action) {
                (PistonPhase::Retracted, _) => {
                    if let Some(PushPlan::Move { head, moved, broken, reserve }) = plan
                        && next != phase
                    {
                        reserved.extend(*reserve);
                        if moved.is_empty() && broken.is_empty() {
                            reserved.push(*head);
                        }
                        // A cell that reads as air but can't be written (past the top of the
                        // world) blocks the push.
                        if !reserved.iter().all(|c| w.reserve(*c)) {
                            for c in &reserved {
                                w.head(*c, None);
                            }
                            continue;
                        }
                        busy = [*head].iter().chain(moved).chain(broken).chain(reserve).copied().collect();
                    }
                }
                (PistonPhase::Extended, Action::StartRetract { pull: true }) => {
                    if let Some(PushPlan::Move { moved, .. }) = plan {
                        busy = [head].iter().chain(moved).copied().collect();
                    }
                }
                (PistonPhase::Extending { .. }, Action::Land { .. } | Action::Snap) => {
                    // What it reserved is free to it; anything else in the way (or unloaded)
                    // makes it wait.
                    let PushPlan::Move { moved, broken, .. } = resume_plan(&*w, p, facing) else { continue };
                    self.shift(w, facing, &moved, &broken, Some(head));
                    w.head(head, Some((facing, sticky)));
                }
                (PistonPhase::Retracting { .. }, Action::Land { pull }) => {
                    if !w.cube_loaded(cube_of(head)) {
                        continue;
                    }
                    let pulled = if pull { pull_plan(&*w, p, facing) } else { None };
                    w.head(head, None);
                    self.dirty.insert(head);
                    // Only into the head's cell: if the head was broken, whatever took its place
                    // stays.
                    if let Some(b) = pulled
                        && w.cell(head) == Cell::Air
                    {
                        self.shift(w, facing.opposite(), &[b], &[], None);
                    }
                }
                _ => {}
            }
            if next == phase {
                continue;
            }
            let Some(r) = self.pistons.get_mut(&(p, owner)) else { continue };
            r.phase = next;
            r.busy = busy;
            if !reserved.is_empty() || matches!(action, Action::Land { .. } | Action::Snap) {
                r.reserved = reserved;
            }
            let k = Kind::Piston { facing, sticky, phase: next };
            w.write_device(p, k);
            let moving = matches!(next, PistonPhase::Extending { .. } | PistonPhase::Retracting { .. });
            w.save_pending(p, moving.then_some((Pending { next: k, remaining: 1 }, false)));
            self.dirty.insert(p);
        }
    }

    /// Lands a piston move, taking along what the tick keeps for the devices that move: their
    /// pending changes still apply on time.
    fn shift(&mut self, w: &mut impl World, to: Dir, moved: &[Pos], broken: &[Pos], head: Option<Pos>) {
        let mut carried = Vec::new();
        for q in moved.iter().chain(broken) {
            let key = (*q, w.instance(*q));
            carried.push(self.pending.remove(&key));
            // A moved piston is a retracted one: its power is read again where it lands.
            self.pistons.remove(&key);
            self.frozen_dirty.remove(q);
            self.last_inputs.remove(q);
            self.readers.remove(q);
        }
        w.shift(to, moved, broken, head);
        for (q, e) in moved.iter().zip(carried) {
            let d = step(*q, to);
            if let Some(mut e) = e {
                e.cubes = None;
                self.pending.insert((d, w.instance(d)), e);
            }
            self.dirty.insert(*q);
            self.dirty.insert(d);
        }
        self.dirty.extend(broken.iter().copied());
        self.dirty.extend(head);
    }

    fn react(&mut self, w: &mut impl World, p: Pos, k: Kind, i: [SidePower; 6], range: Range) {
        let container = if matches!(k, Kind::Comparator { .. }) { w.container(p) } else { None };
        if container.is_some() {
            self.readers.insert(p);
        } else {
            self.readers.remove(&p);
        }
        let lockers = match k {
            Kind::Repeater { facing, .. } => side_lockers(&*w, p, facing),
            _ => false,
        };
        let owner = w.instance(p);
        if let Some(e) = self.pending.get(&(p, owner)) {
            // One pending change per device, except that locking or unlocking a repeater replaces
            // a change that would leave it in the wrong lock state.
            match e.pending.next {
                Kind::Repeater { locked, .. } if locked != lockers => {
                    self.pending.remove(&(p, owner));
                    w.save_pending(p, None);
                }
                _ => return,
            }
        }
        if let Some((delay, next)) = react(k, i, range, container, lockers) {
            let pending = Pending { next, remaining: delay.max(1) };
            let cubes = reach(&*w, p);
            let frozen = !cubes.iter().all(|c| w.cube_loaded(*c));
            w.save_pending(p, Some((pending, frozen)));
            self.pending.insert((p, owner), Entry { pending, cubes: Some(cubes), frozen, fresh: false });
        }
    }
}

/// Ticks between retries of a frozen circuit (spec §7.1).
const RETRY: u32 = 20;

/// The world with wire strengths filled in: those the tick solved, and any other network solved
/// when one of its wires is first read. A wire of an incomplete network reads as 0 and sets
/// `frozen`.
struct Solved<'a, W> {
    w: &'a W,
    strengths: &'a RefCell<BTreeMap<Pos, Option<u8>>>,
    range: Range,
    frozen: StdCell<bool>,
}

impl<W: View> View for Solved<'_, W> {
    fn cell(&self, p: Pos) -> Cell {
        let c = self.w.cell(p);
        let Cell::Wire(mut wire) = c else { return c };
        let known = self.strengths.borrow().get(&p).copied();
        let strength = known.unwrap_or_else(|| {
            let net = network(self.w, p);
            let solved = (!net.incomplete).then(|| solve(self.w, &net, self.range));
            let mut m = self.strengths.borrow_mut();
            for q in &net.wires {
                m.insert(*q, solved.as_ref().map(|s| s[q]));
            }
            solved.map(|s| s[&p])
        });
        match strength {
            Some(s) => wire.strength = s,
            None => self.frozen.set(true),
        }
        Cell::Wire(wire)
    }
}

/// Adds `p`, its neighbors, the neighbors of solid blocks next to it (they relay power) and the
/// positions a step away from it (wire climbing to or from it).
fn around(v: &impl View, p: Pos, out: &mut BTreeSet<Pos>) {
    out.insert(p);
    for d in Dir::ALL {
        let q = step(p, d);
        out.insert(q);
        if matches!(v.cell(q), Cell::Block { solid: true, .. }) {
            out.extend(Dir::ALL.map(|e| step(q, e)));
        }
    }
    for d in Dir::HORIZONTAL {
        let q = step(p, d);
        out.insert(step(q, Dir::Up));
        out.insert(step(q, Dir::Down));
    }
}

/// The cubes a device's circuit spans: everything its inputs read and the networks around it.
fn reach(v: &impl View, p: Pos) -> BTreeSet<[i32; 3]> {
    let r = Recorder::new(v);
    inputs(&r, p);
    let mut cubes = r.cubes.into_inner();
    cubes.insert(cube_of(p));
    let mut near = BTreeSet::new();
    around(v, p, &mut near);
    let mut seen = BTreeSet::new();
    for q in near {
        if !seen.contains(&q) && matches!(v.cell(q), Cell::Wire(_)) {
            let net = network(v, q);
            seen.extend(net.wires);
            cubes.extend(net.cubes);
        }
    }
    cubes
}

/// Whether a powered repeater or comparator points into a side of the repeater at `p`.
fn side_lockers(v: &impl View, p: Pos, facing: Dir) -> bool {
    sides_of(facing).iter().any(|d| match v.cell(step(p, *d)) {
        Cell::Device(Device { kind: Some(Kind::Repeater { facing: f, powered: true, .. }), .. }) => f == d.opposite(),
        Cell::Device(Device { kind: Some(Kind::Comparator { facing: f, output, .. }), .. }) => output > 0 && f == d.opposite(),
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testgrid::MapGrid;
    use crate::{Power, Push, Wire, device};

    const R15: Range = Range::from_index_const(0);

    #[derive(Clone, PartialEq, Debug)]
    enum Ev {
        Wire(Pos, u8),
        Device(Pos, Kind),
        Notify(Pos, [SidePower; 6]),
        Save(Pos, Option<(Pending, bool)>),
    }

    impl Ev {
        fn moved(&self, by: Pos) -> Ev {
            let m = |p: &Pos| [p[0] - by[0], p[1] - by[1], p[2] - by[2]];
            match self {
                Ev::Wire(p, s) => Ev::Wire(m(p), *s),
                Ev::Device(p, k) => Ev::Device(m(p), *k),
                Ev::Notify(p, i) => Ev::Notify(m(p), *i),
                Ev::Save(p, x) => Ev::Save(m(p), *x),
            }
        }
    }

    /// A grid seen the way the adapter sees Weft: built-in devices take their outputs from their
    /// kind and the current range, and wire strength isn't stored (every wire reads as 0). What
    /// the tick wrote is kept aside for the tests, with unloadable cubes and a log.
    struct TestWorld {
        g: MapGrid,
        range: Range,
        unloaded: BTreeSet<[i32; 3]>,
        written: BTreeMap<Pos, (u8, Shape)>,
        log: Vec<Ev>,
        /// Comparator readings by the comparator's position.
        containers: BTreeMap<Pos, u8>,
        /// Device instance ids (`World::instance`), 0 where not set.
        instances: BTreeMap<Pos, u64>,
        /// Cells that read as air but can't be written: past the top of the world.
        unwritable: BTreeSet<Pos>,
    }

    impl TestWorld {
        fn new(g: MapGrid) -> TestWorld {
            TestWorld { g, range: R15, unloaded: BTreeSet::new(), written: BTreeMap::new(), log: Vec::new(), containers: BTreeMap::new(), instances: BTreeMap::new(), unwritable: BTreeSet::new() }
        }
        fn put(&mut self, p: Pos, k: Kind) -> &mut Self {
            self.g.put(p, Cell::Device(device(k, self.range)));
            self
        }
        fn kind(&self, p: Pos) -> Kind {
            match self.g.cell(p) {
                Cell::Device(Device { kind: Some(k), .. }) => k,
                c => panic!("no device at {p:?}: {c:?}"),
            }
        }
        /// The strength the tick last wrote for the wire at `p`.
        fn strength(&self, p: Pos) -> u8 {
            assert!(matches!(self.g.cell(p), Cell::Wire(_)), "no wire at {p:?}");
            self.written.get(&p).map_or(0, |w| w.0)
        }
        fn strengths(&self) -> BTreeMap<Pos, u8> {
            self.written.iter().map(|(p, w)| (*p, w.0)).collect()
        }
        fn wire_writes(&self) -> Vec<(Pos, u8)> {
            self.log.iter().filter_map(|e| if let Ev::Wire(p, s) = e { Some((*p, *s)) } else { None }).collect()
        }
        fn device_writes(&self, at: Pos) -> usize {
            self.log.iter().filter(|e| matches!(e, Ev::Device(p, _) if *p == at)).count()
        }
        fn notifies(&self) -> Vec<(Pos, [SidePower; 6])> {
            self.log.iter().filter_map(|e| if let Ev::Notify(p, i) = e { Some((*p, *i)) } else { None }).collect()
        }
    }

    impl View for TestWorld {
        fn cell(&self, p: Pos) -> Cell {
            if self.unloaded.contains(&crate::cube_of(p)) {
                return Cell::Unloaded;
            }
            match self.g.cell(p) {
                Cell::Device(Device { kind: Some(k), .. }) => Cell::Device(device(k, self.range)),
                Cell::Wire(w) => Cell::Wire(Wire { strength: 0, ..w }),
                c => c,
            }
        }
    }

    impl World for TestWorld {
        fn range(&self) -> Range {
            self.range
        }
        fn cube_loaded(&self, cube: [i32; 3]) -> bool {
            !self.unloaded.contains(&cube)
        }
        fn container(&self, p: Pos) -> Option<u8> {
            self.containers.get(&p).copied()
        }
        fn instance(&self, p: Pos) -> u64 {
            self.instances.get(&p).copied().unwrap_or(0)
        }
        fn write_wire(&mut self, p: Pos, strength: u8, shape: Shape) {
            assert!(matches!(self.g.cell(p), Cell::Wire(_)), "write_wire on non-wire {p:?}");
            if self.written.insert(p, (strength, shape)) != Some((strength, shape)) {
                self.log.push(Ev::Wire(p, strength));
            }
        }
        fn write_device(&mut self, p: Pos, k: Kind) {
            self.g.put(p, Cell::Device(device(k, self.range)));
            self.log.push(Ev::Device(p, k));
        }
        fn notify(&mut self, p: Pos, inputs: [SidePower; 6]) {
            self.log.push(Ev::Notify(p, inputs));
        }
        fn save_pending(&mut self, p: Pos, pending: Option<(Pending, bool)>) {
            self.log.push(Ev::Save(p, pending));
        }
        fn reserve(&mut self, p: Pos) -> bool {
            if self.unwritable.contains(&p) {
                return false;
            }
            self.g.put(p, Cell::Block { solid: false, push: Push::Reserved });
            true
        }
        fn shift(&mut self, facing: Dir, moved: &[Pos], broken: &[Pos], _head: Option<Pos>) {
            for b in broken {
                self.g.air(*b);
            }
            for p in moved {
                let c = self.g.cell(*p);
                self.g.put(step(*p, facing), c);
                self.g.air(*p);
            }
        }
        fn head(&mut self, p: Pos, head: Option<(Dir, bool)>) {
            if head.is_some() {
                self.g.put(p, HEAD);
            } else if matches!(self.g.cell(p), HEAD | Cell::Block { push: Push::Reserved, .. }) {
                self.g.air(p);
            }
        }
    }

    /// How the test world shows a piston head.
    const HEAD: Cell = Cell::Block { solid: false, push: Push::Never };

    fn piston(facing: Dir, sticky: bool, phase: PistonPhase) -> Kind {
        Kind::Piston { facing, sticky, phase }
    }

    fn phase_at(w: &TestWorld, p: Pos) -> PistonPhase {
        match w.kind(p) {
            Kind::Piston { phase, .. } => phase,
            k => panic!("no piston at {p:?}: {k:?}"),
        }
    }

    fn lever(on: bool) -> Kind {
        Kind::Lever { on, attached: Dir::Down }
    }

    fn repeater(facing: Dir, delay: u8) -> Kind {
        Kind::Repeater { facing, delay, powered: false, locked: false }
    }

    fn powered(k: Kind) -> bool {
        matches!(k, Kind::Repeater { powered: true, .. })
    }

    fn ticks(sim: &mut Sim, w: &mut TestWorld, n: u32) {
        for _ in 0..n {
            sim.tick(w);
        }
    }

    /// lever [0,0,0] → dust [1..=5,0,0] → lamp [6,0,0]
    fn lever_line() -> TestWorld {
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], lever(false)).put([6, 0, 0], Kind::Lamp { lit: false });
        for x in 1..=5 {
            w.g.dust([x, 0, 0]);
        }
        w
    }

    #[test]
    fn lever_dust_lamp() {
        let mut w = lever_line();
        let mut sim = Sim::default();
        w.put([0, 0, 0], lever(true));
        sim.touch([0, 0, 0]);
        sim.tick(&mut w);
        assert_eq!(w.wire_writes(), (1..=5).map(|x| ([x, 0, 0], 16 - x as u8)).collect::<Vec<_>>());
        assert_eq!(w.kind([6, 0, 0]), Kind::Lamp { lit: false });
        assert_eq!(sim.pending[&([6, 0, 0], 0)].pending, Pending { next: Kind::Lamp { lit: true }, remaining: 1 });
        w.log.clear();
        sim.tick(&mut w);
        assert_eq!(w.kind([6, 0, 0]), Kind::Lamp { lit: true });
        assert!(w.wire_writes().is_empty());
        assert!(!sim.has_pending([6, 0, 0], 0));
    }

    #[test]
    fn torch_clock_one_tick() {
        // A torch on the side of a block powers dust on top of itself, which runs onto the block
        // and powers it, which turns the torch off.
        let mut w = TestWorld::new(MapGrid::default());
        w.g.solid([0, 0, 0]).dust([1, 1, 0]).dust([0, 1, 0]);
        w.put([1, 0, 0], Kind::Torch { lit: true, attached: Dir::West });
        let mut sim = Sim::default();
        sim.touch([1, 0, 0]);
        for n in 1..=10 {
            sim.tick(&mut w);
            assert_eq!(w.kind([1, 0, 0]), Kind::Torch { lit: n % 2 == 1, attached: Dir::West }, "after tick {n}");
            assert_eq!(w.strength([0, 1, 0]), if n % 2 == 1 { 14 } else { 0 }, "dust after tick {n}");
        }
    }

    #[test]
    fn turn_off_is_full_recompute() {
        let mut w = lever_line();
        let mut sim = Sim::default();
        w.put([0, 0, 0], lever(true));
        sim.touch([0, 0, 0]);
        ticks(&mut sim, &mut w, 3);
        w.put([0, 0, 0], lever(false));
        sim.touch([0, 0, 0]);
        sim.tick(&mut w);
        for x in 1..=5 {
            assert_eq!(w.strength([x, 0, 0]), 0);
        }
    }

    #[test]
    fn writes_once_per_tick() {
        let mut w = lever_line();
        w.put([6, 0, 0], lever(false));
        let mut sim = Sim::default();
        for on in [true, false, true] {
            w.put([0, 0, 0], lever(on)).put([6, 0, 0], lever(on));
            sim.touch([0, 0, 0]);
            sim.touch([6, 0, 0]);
            w.log.clear();
            sim.tick(&mut w);
            let writes = w.wire_writes();
            let unique: BTreeSet<Pos> = writes.iter().map(|(p, _)| *p).collect();
            assert_eq!(unique.len(), writes.len(), "{writes:?}");
            assert_eq!(writes.len(), 5);
        }
        assert_eq!((1..=5).map(|x| w.strength([x, 0, 0])).collect::<Vec<_>>(), [15, 14, 13, 14, 15]);
    }

    /// Two levers feeding one line with a torch and a lamp on it.
    fn two_lever_build(w: &mut TestWorld, o: Pos) -> [Pos; 2] {
        let at = |p: Pos| [o[0] + p[0], o[1] + p[1], o[2] + p[2]];
        w.put(at([0, 0, 0]), lever(true)).put(at([2, 0, 1]), lever(true)).put(at([5, 0, 0]), Kind::Lamp { lit: false });
        for x in 1..=4 {
            w.g.dust(at([x, 0, 0]));
        }
        w.g.solid(at([3, 0, -1]));
        w.put(at([3, 0, -2]), Kind::Torch { lit: true, attached: Dir::South });
        [at([0, 0, 0]), at([2, 0, 1])]
    }

    #[test]
    fn order_independent() {
        let (oa, ob) = ([0, 0, 0], [37, 5, -20]);
        let mut a = TestWorld::new(MapGrid::default());
        let mut b = TestWorld::new(MapGrid::default());
        let la = two_lever_build(&mut a, oa);
        let lb = two_lever_build(&mut b, ob);
        let (mut sa, mut sb) = (Sim::default(), Sim::default());
        sa.touch(la[0]);
        sa.touch(la[1]);
        sb.touch(lb[1]);
        sb.touch(lb[0]);
        ticks(&mut sa, &mut a, 10);
        ticks(&mut sb, &mut b, 10);
        assert!(!a.log.is_empty());
        let ra: Vec<Ev> = a.log.iter().map(|e| e.moved(oa)).collect();
        let rb: Vec<Ev> = b.log.iter().map(|e| e.moved(ob)).collect();
        assert_eq!(ra, rb);
    }

    #[test]
    fn orientation_independent() {
        // lever → dust corner → repeater (delay 2) → dust into a block → torch on its side → lamp
        let mut base = TestWorld::new(MapGrid::default());
        base.put([0, 0, 0], lever(true)).put([2, 0, 2], repeater(Dir::South, 2));
        base.g.dust([1, 0, 0]).dust([2, 0, 0]).dust([2, 0, 1]).dust([2, 0, 3]).solid([2, 0, 4]);
        base.put([3, 0, 4], Kind::Torch { lit: true, attached: Dir::West }).put([4, 0, 4], Kind::Lamp { lit: false });
        let all: Vec<Pos> = base.g.cells.keys().copied().collect();
        let start = base.g.clone();
        let run = |g: MapGrid, k: usize| {
            let mut w = TestWorld::new(g);
            let mut sim = Sim::default();
            for p in &all {
                sim.touch((0..k).fold(*p, |q, _| crate::testgrid::rotate_pos(q)));
            }
            ticks(&mut sim, &mut w, 20);
            let s = w.strengths();
            (w.g, s)
        };
        let (done, strengths) = run(start.clone(), 0);
        assert_eq!(done.cell([4, 0, 4]), Cell::Device(device(Kind::Lamp { lit: false }, R15)));
        assert!(matches!(done.cell([2, 0, 2]), Cell::Device(Device { kind: Some(Kind::Repeater { powered: true, .. }), .. })));
        assert_eq!(strengths[&[2, 0, 3]], 15);
        let (mut g, mut want) = (start, done);
        for k in 1..=3 {
            g = g.rotated();
            want = want.rotated();
            let want_strengths: BTreeMap<Pos, u8> = strengths.iter().map(|(p, s)| ((0..k).fold(*p, |q, _| crate::testgrid::rotate_pos(q)), *s)).collect();
            let (got, got_strengths) = run(g.clone(), k);
            assert_eq!(got.cells, want.cells, "after {k} turns");
            assert_eq!(got_strengths, want_strengths, "strengths after {k} turns");
        }
    }

    #[test]
    fn repeater_pulse_stretch() {
        let mut w = TestWorld::new(MapGrid::default());
        w.put([-1, 0, 0], lever(true)).put([0, 0, 0], repeater(Dir::East, 4));
        let mut sim = Sim::default();
        sim.touch([-1, 0, 0]);
        sim.tick(&mut w);
        w.put([-1, 0, 0], lever(false));
        sim.touch([-1, 0, 0]);
        let mut on = Vec::new();
        for n in 2..=15 {
            sim.tick(&mut w);
            if powered(w.kind([0, 0, 0])) {
                on.push(n);
            }
        }
        assert_eq!(on, [5, 6, 7, 8]);
    }

    #[test]
    fn repeater_lock_holds() {
        // R [0,0,0] facing east, fed by lever L behind it; S [0,0,-1] points into R's north side,
        // fed by lever L2.
        let (r, l, s, l2) = ([0, 0, 0], [-1, 0, 0], [0, 0, -1], [0, 0, -2]);
        let mut w = TestWorld::new(MapGrid::default());
        w.put(r, repeater(Dir::East, 3)).put(l, lever(true)).put(s, repeater(Dir::South, 1)).put(l2, lever(false));
        let mut sim = Sim::default();
        sim.touch(l);
        ticks(&mut sim, &mut w, 4);
        assert!(powered(w.kind(r)));
        w.put(l2, lever(true));
        sim.touch(l2);
        ticks(&mut sim, &mut w, 3);
        assert_eq!(w.kind(r), Kind::Repeater { facing: Dir::East, delay: 3, powered: true, locked: true });
        w.put(l, lever(false));
        sim.touch(l);
        ticks(&mut sim, &mut w, 10);
        assert_eq!(w.kind(r), Kind::Repeater { facing: Dir::East, delay: 3, powered: true, locked: true });
        w.put(l2, lever(false));
        sim.touch(l2);
        ticks(&mut sim, &mut w, 3);
        assert_eq!(w.kind(r), Kind::Repeater { facing: Dir::East, delay: 3, powered: true, locked: false });
        ticks(&mut sim, &mut w, 2);
        assert!(powered(w.kind(r)), "fresh delay of 3 after unlock");
        sim.tick(&mut w);
        assert!(!powered(w.kind(r)));
    }

    #[test]
    fn frozen_circuit_keeps_state_and_delays() {
        let mut w = TestWorld::new(MapGrid::default());
        w.put([-1, 0, 0], lever(true)).put([0, 0, 0], repeater(Dir::East, 3));
        let mut sim = Sim::default();
        sim.touch([-1, 0, 0]);
        sim.tick(&mut w);
        assert_eq!(sim.pending[&([0, 0, 0], 0)].pending.remaining, 3);
        sim.tick(&mut w);
        // The lever's cube goes: the repeater is still here, its circuit isn't whole.
        w.unloaded.insert([-1, 0, 0]);
        w.log.clear();
        ticks(&mut sim, &mut w, 10);
        assert_eq!(w.device_writes([0, 0, 0]), 0);
        let next = Kind::Repeater { facing: Dir::East, delay: 3, powered: true, locked: false };
        assert_eq!(w.log, [Ev::Save([0, 0, 0], Some((Pending { next, remaining: 2 }, true)))]);
        // Its own cube goes too: not seen at all, still not counting.
        w.unloaded.insert([0, 0, 0]);
        ticks(&mut sim, &mut w, 10);
        assert_eq!(sim.pending[&([0, 0, 0], 0)].pending.remaining, 2);
        w.unloaded.clear();
        sim.tick(&mut w);
        assert_eq!(w.device_writes([0, 0, 0]), 0);
        sim.tick(&mut w);
        assert!(powered(w.kind([0, 0, 0])));
    }

    #[test]
    fn saved_remaining_counts_from_the_next_tick() {
        // However a change was last saved (on reacting, on freezing, on thawing, after
        // `schedule`), it applies `remaining` ticks after that save: what a restored save relies on.
        let mut w = TestWorld::new(MapGrid::default());
        w.put([-1, 0, 0], lever(true)).put([0, 0, 0], repeater(Dir::East, 4));
        let mut sim = Sim::default();
        sim.touch([-1, 0, 0]);
        let (mut last, mut checked) = (None, 0);
        for t in 0..30u32 {
            match t {
                2 => drop(w.unloaded.insert([-1, 0, 0])),
                5 => drop(w.unloaded.remove(&[-1, 0, 0])),
                _ => {}
            }
            w.log.clear();
            sim.tick(&mut w);
            for e in &w.log {
                match e {
                    Ev::Save(_, Some((p, false))) => last = Some((t, t + p.remaining)),
                    Ev::Save(_, _) => last = None,
                    Ev::Device(..) => {
                        let (at, due) = last.take().expect("applied without a save");
                        assert_eq!(due, t, "saved in tick {at}");
                        checked += 1;
                    }
                    _ => {}
                }
            }
            if t == 12 {
                // As restored after a restart: scheduled fresh, saved by the next tick.
                w.put([0, 0, 0], repeater(Dir::East, 4));
                sim.schedule([0, 0, 0], Pending { next: Kind::Repeater { facing: Dir::East, delay: 4, powered: true, locked: false }, remaining: 3 }, 0);
            }
        }
        assert_eq!(checked, 2);
        assert!(powered(w.kind([0, 0, 0])));
    }

    #[test]
    fn scheduled_change_counts_from_the_first_tick_that_sees_it() {
        // A button pressed between ticks stays on for exactly its length.
        let mut w = TestWorld::new(MapGrid::default());
        let on = Kind::Button { on: true, attached: Dir::Down, wood: false };
        w.put([0, 0, 0], on);
        let mut sim = Sim::default();
        sim.schedule([0, 0, 0], Pending { next: Kind::Button { on: false, attached: Dir::Down, wood: false }, remaining: 3 }, 0);
        for n in 1..=3 {
            sim.tick(&mut w);
            assert_eq!(w.kind([0, 0, 0]), on, "after tick {n}");
        }
        sim.tick(&mut w);
        assert_eq!(w.kind([0, 0, 0]), Kind::Button { on: false, attached: Dir::Down, wood: false });
    }

    #[test]
    fn applied_change_keeps_what_players_set() {
        // A repeater's delay and a comparator's mode changed while a change counts down.
        let mut w = TestWorld::new(MapGrid::default());
        w.put([-1, 0, 0], lever(true)).put([0, 0, 0], repeater(Dir::East, 3));
        w.put([-1, 0, 2], lever(true)).put([0, 0, 2], Kind::Comparator { facing: Dir::East, subtract: false, output: 0 });
        let mut sim = Sim::default();
        sim.touch([-1, 0, 0]);
        sim.touch([-1, 0, 2]);
        sim.tick(&mut w);
        w.put([0, 0, 0], repeater(Dir::East, 4));
        w.put([0, 0, 2], Kind::Comparator { facing: Dir::East, subtract: true, output: 0 });
        ticks(&mut sim, &mut w, 3);
        assert_eq!(w.kind([0, 0, 0]), Kind::Repeater { facing: Dir::East, delay: 4, powered: true, locked: false });
        assert!(matches!(w.kind([0, 0, 2]), Kind::Comparator { subtract: true, output: 15, .. }));
    }

    #[test]
    fn comparator_follows_its_container() {
        // Nothing tells redstone a container changed: a comparator reading one checks every tick.
        let mut w = TestWorld::new(MapGrid::default());
        let c = [0, 0, 0];
        w.put(c, Kind::Comparator { facing: Dir::East, subtract: false, output: 0 });
        w.containers.insert(c, 0);
        let mut sim = Sim::default();
        sim.touch(c);
        ticks(&mut sim, &mut w, 2);
        w.containers.insert(c, 7);
        ticks(&mut sim, &mut w, 2);
        assert!(matches!(w.kind(c), Kind::Comparator { output: 7, .. }));
        w.containers.insert(c, 3);
        ticks(&mut sim, &mut w, 2);
        assert!(matches!(w.kind(c), Kind::Comparator { output: 3, .. }));
        // Gone: not read again.
        w.g.air(c);
        sim.forget(c);
        sim.tick(&mut w);
        assert!(sim.readers.is_empty());
    }

    #[test]
    fn same_place_in_two_worlds() {
        // Two dimensions with the same circuit at the same place share the Sim (G6): only the
        // world whose lever changed reacts, whichever of them ticks first.
        let (mut a, mut b) = (lever_line(), lever_line());
        a.instances.insert([6, 0, 0], 1);
        b.instances.insert([6, 0, 0], 2);
        let mut sim = Sim::default();
        a.put([0, 0, 0], lever(true));
        sim.tick(&mut b);
        sim.touch([0, 0, 0]);
        sim.tick(&mut a);
        for _ in 0..3 {
            sim.tick(&mut b);
            sim.tick(&mut a);
        }
        assert_eq!(a.kind([6, 0, 0]), Kind::Lamp { lit: true });
        assert_eq!(b.kind([6, 0, 0]), Kind::Lamp { lit: false });
        // A change scheduled from outside belongs to the instance it names.
        sim.schedule([6, 0, 0], Pending { next: Kind::Lamp { lit: true }, remaining: 1 }, 1);
        a.put([6, 0, 0], Kind::Lamp { lit: false });
        for _ in 0..3 {
            sim.tick(&mut b);
        }
        assert_eq!(b.kind([6, 0, 0]), Kind::Lamp { lit: false });
    }

    #[test]
    fn piston_pushes_two_ticks_after_reacting() {
        // A lever beside a piston facing east, two blocks in front of it.
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], piston(Dir::East, false, PistonPhase::Retracted)).put([0, 0, 1], lever(true));
        w.g.solid([1, 0, 0]).solid([2, 0, 0]);
        let mut sim = Sim::default();
        sim.touch([0, 0, 1]);
        sim.tick(&mut w);
        assert_eq!(phase_at(&w, [0, 0, 0]), PistonPhase::Retracted, "reacts a tick later");
        sim.tick(&mut w);
        assert!(matches!(phase_at(&w, [0, 0, 0]), PistonPhase::Extending { .. }));
        assert_eq!(w.g.cell([3, 0, 0]), Cell::Block { solid: false, push: Push::Reserved }, "destination held");
        assert_eq!(w.g.cell([1, 0, 0]), Cell::block(true), "blocks stay while moving");
        sim.tick(&mut w);
        assert_eq!(w.g.cell([1, 0, 0]), Cell::block(true));
        sim.tick(&mut w);
        assert_eq!(phase_at(&w, [0, 0, 0]), PistonPhase::Extended);
        assert_eq!((w.g.cell([1, 0, 0]), w.g.cell([2, 0, 0]), w.g.cell([3, 0, 0])), (HEAD, Cell::block(true), Cell::block(true)));
        // Off: the head goes 1 + 2 ticks later, the blocks stay.
        w.put([0, 0, 1], lever(false));
        sim.touch([0, 0, 1]);
        ticks(&mut sim, &mut w, 3);
        assert_eq!(phase_at(&w, [0, 0, 0]), PistonPhase::Retracting { since: 1, pull: false });
        sim.tick(&mut w);
        assert_eq!(phase_at(&w, [0, 0, 0]), PistonPhase::Retracted);
        assert_eq!((w.g.cell([1, 0, 0]), w.g.cell([2, 0, 0])), (Cell::Air, Cell::block(true)));
        assert!(sim.pistons.is_empty(), "idle pistons aren't kept");
    }

    #[test]
    fn sticky_piston_pulls_back() {
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], piston(Dir::East, true, PistonPhase::Extended)).put([0, 0, 1], lever(false));
        w.g.put([1, 0, 0], HEAD).solid([2, 0, 0]);
        let mut sim = Sim::default();
        sim.touch([0, 0, 1]);
        ticks(&mut sim, &mut w, 4);
        assert_eq!(phase_at(&w, [0, 0, 0]), PistonPhase::Retracted);
        assert_eq!((w.g.cell([1, 0, 0]), w.g.cell([2, 0, 0])), (Cell::block(true), Cell::Air));
    }

    #[test]
    fn pushed_repeater_fires_on_time() {
        // A repeater counting down its delay of 4 is pushed one block east meanwhile.
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], piston(Dir::East, false, PistonPhase::Retracted)).put([0, 0, -1], lever(true));
        w.put([1, 0, 0], repeater(Dir::North, 4)).put([1, 0, 1], Kind::PowerBlock);
        let mut sim = Sim::default();
        sim.touch([0, 0, -1]);
        sim.touch([1, 0, 1]);
        sim.tick(&mut w);
        assert!(sim.has_pending([1, 0, 0], 0));
        ticks(&mut sim, &mut w, 3);
        assert_eq!(w.kind([2, 0, 0]), repeater(Dir::North, 4), "pushed, not fired yet");
        sim.tick(&mut w);
        assert!(powered(w.kind([2, 0, 0])), "fires 4 ticks after reacting");
    }

    #[test]
    fn conflicting_pistons_wait() {
        // Both heads into [1, 0, 0]: neither moves until one loses power.
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], piston(Dir::East, false, PistonPhase::Retracted)).put([0, 0, 1], lever(true));
        w.put([2, 0, 0], piston(Dir::West, false, PistonPhase::Retracted)).put([2, 0, 1], lever(true));
        let mut sim = Sim::default();
        sim.touch([0, 0, 1]);
        sim.touch([2, 0, 1]);
        ticks(&mut sim, &mut w, 5);
        assert_eq!((phase_at(&w, [0, 0, 0]), phase_at(&w, [2, 0, 0])), (PistonPhase::Retracted, PistonPhase::Retracted));
        w.put([2, 0, 1], lever(false));
        sim.touch([2, 0, 1]);
        ticks(&mut sim, &mut w, 5);
        assert_eq!((phase_at(&w, [0, 0, 0]), phase_at(&w, [2, 0, 0])), (PistonPhase::Extended, PistonPhase::Retracted));
    }

    #[test]
    fn piston_not_powered_from_its_face() {
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], piston(Dir::East, true, PistonPhase::Retracted)).put([1, 0, 0], Kind::PowerBlock);
        let mut sim = Sim::default();
        sim.touch([1, 0, 0]);
        ticks(&mut sim, &mut w, 5);
        assert_eq!(phase_at(&w, [0, 0, 0]), PistonPhase::Retracted);
        assert!(sim.pistons.is_empty());
    }

    #[test]
    fn restored_move_lands_through_its_reservation() {
        // Restarted mid-move: the Sim knows only the phase, the world has the reservation.
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], piston(Dir::East, false, PistonPhase::Extending { since: 0 })).put([0, 0, 1], lever(true));
        w.g.solid([1, 0, 0]).put([2, 0, 0], Cell::Block { solid: false, push: Push::Reserved });
        let mut sim = Sim::default();
        sim.schedule([0, 0, 0], Pending { next: piston(Dir::East, false, PistonPhase::Extending { since: 1 }), remaining: 1 }, 0);
        sim.touch([0, 0, 1]);
        sim.tick(&mut w);
        assert_eq!(phase_at(&w, [0, 0, 0]), PistonPhase::Extended);
        assert_eq!((w.g.cell([1, 0, 0]), w.g.cell([2, 0, 0])), (HEAD, Cell::block(true)));
    }

    /// Piston B at `b` pushes into C at [1, 0, 0], a powered retracted piston facing north into
    /// an unpushable block.
    fn push_blocked_piston(b: Pos, facing: Dir) {
        let mut w = TestWorld::new(MapGrid::default());
        w.put(b, piston(facing, false, PistonPhase::Retracted)).put(step(b, Dir::North), lever(true));
        w.put([1, 0, 0], piston(Dir::North, false, PistonPhase::Retracted)).put([1, 0, 1], lever(true));
        w.g.put([1, 0, -1], Cell::Block { solid: true, push: Push::Never });
        let mut sim = Sim::default();
        sim.touch(step(b, Dir::North));
        sim.touch([1, 0, 1]);
        ticks(&mut sim, &mut w, 6);
        assert_eq!(phase_at(&w, b), PistonPhase::Extended);
        assert!(matches!(w.kind(step(step(b, facing), facing)), Kind::Piston { .. }), "C pushed");
    }

    #[test]
    fn piston_pushes_blocked_powered_piston() {
        // Whichever of the two comes first in position order.
        push_blocked_piston([0, 0, 0], Dir::East);
        push_blocked_piston([2, 0, 0], Dir::West);
    }

    #[test]
    fn push_past_the_top_is_blocked() {
        // The cell above the world reads as air but can't be written: nothing moves into it.
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], piston(Dir::Up, false, PistonPhase::Retracted)).put([1, 0, 0], lever(true));
        w.g.solid([0, 1, 0]);
        w.unwritable.insert([0, 2, 0]);
        let mut sim = Sim::default();
        sim.touch([1, 0, 0]);
        ticks(&mut sim, &mut w, 6);
        assert_eq!(phase_at(&w, [0, 0, 0]), PistonPhase::Retracted);
        assert_eq!(w.g.cell([0, 1, 0]), Cell::block(true));
    }

    #[test]
    fn sticky_pull_leaves_a_replaced_head_alone() {
        // The head was broken and a lamp put in its cell; the sticky piston then loses power.
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], piston(Dir::East, true, PistonPhase::Extended)).put([0, 0, 1], lever(false));
        w.put([1, 0, 0], Kind::Lamp { lit: false });
        w.g.solid([2, 0, 0]);
        let mut sim = Sim::default();
        sim.touch([0, 0, 1]);
        ticks(&mut sim, &mut w, 6);
        assert_eq!(phase_at(&w, [0, 0, 0]), PistonPhase::Retracted);
        assert_eq!(w.kind([1, 0, 0]), Kind::Lamp { lit: false });
        assert_eq!(w.g.cell([2, 0, 0]), Cell::block(true), "not pulled");
    }

    /// Sticky A loses power while B pushes a block north into the cell in front of A's head.
    fn pull_race(a: Pos, facing: Dir) -> (Cell, Cell) {
        let mut w = TestWorld::new(MapGrid::default());
        let head = step(a, facing);
        w.put(a, piston(facing, true, PistonPhase::Extended)).put(step(a, Dir::Down), lever(false));
        w.g.put(head, HEAD);
        w.put([2, 0, 2], piston(Dir::North, false, PistonPhase::Retracted)).put([2, 0, 3], lever(true));
        w.g.solid([2, 0, 1]);
        let mut sim = Sim::default();
        sim.touch(step(a, Dir::Down));
        sim.touch([2, 0, 3]);
        ticks(&mut sim, &mut w, 8);
        (w.g.cell([2, 0, 0]), w.g.cell(head))
    }

    #[test]
    fn pull_and_push_into_the_same_cell_dont_depend_on_order() {
        assert_eq!(pull_race([0, 0, 0], Dir::East), pull_race([4, 0, 0], Dir::West));
    }

    #[test]
    fn counting_change_is_saved_every_tick() {
        // What is left is always saved, so a restart continues from it whatever happened since
        // (a cube that unloaded stops the count, the save doesn't know when).
        let mut w = TestWorld::new(MapGrid::default());
        w.put([-1, 0, 0], lever(true)).put([0, 0, 0], repeater(Dir::East, 4));
        let mut sim = Sim::default();
        sim.touch([-1, 0, 0]);
        let mut saved = Vec::new();
        for _ in 0..5 {
            w.log.clear();
            sim.tick(&mut w);
            saved.extend(w.log.iter().filter_map(|e| if let Ev::Save(_, s) = e { Some(s.map(|(p, _)| p.remaining)) } else { None }));
        }
        assert_eq!(saved, [Some(4), Some(3), Some(2), Some(1), None]);
    }

    #[test]
    fn incomplete_network_not_written() {
        let mut w = TestWorld::new(MapGrid::default());
        w.put([12, 0, 0], lever(true));
        for x in 13..=17 {
            w.g.dust([x, 0, 0]);
        }
        w.unloaded.insert([1, 0, 0]);
        let mut sim = Sim::default();
        sim.touch([12, 0, 0]);
        ticks(&mut sim, &mut w, 45);
        assert!(w.wire_writes().is_empty());
        w.unloaded.clear();
        ticks(&mut sim, &mut w, 20);
        assert!(w.wire_writes().contains(&([13, 0, 0], 15)));
        assert_eq!(w.strength([17, 0, 0]), 11);
    }

    #[test]
    fn removed_device_drops_pending() {
        // A lever on a block turns the torch standing on it off.
        let mut w = TestWorld::new(MapGrid::default());
        w.g.solid([0, 0, 0]);
        w.put([0, 1, 0], Kind::Torch { lit: true, attached: Dir::Down }).put([1, 0, 0], Kind::Lever { on: true, attached: Dir::West });
        let mut sim = Sim::default();
        sim.touch([1, 0, 0]);
        sim.tick(&mut w);
        assert!(sim.has_pending([0, 1, 0], 0));
        w.g.air([0, 1, 0]);
        sim.tick(&mut w);
        assert_eq!(w.device_writes([0, 1, 0]), 0);
    }

    #[test]
    fn pending_counts_only_where_its_device_is() {
        // Several regions and dimensions share one Sim (G6): a world without the device, here an
        // empty one, neither counts its change down nor drops it.
        let mut a = TestWorld::new(MapGrid::default());
        a.put([-1, 0, 0], lever(true)).put([0, 0, 0], repeater(Dir::East, 3));
        let mut b = TestWorld::new(MapGrid::default());
        let mut sim = Sim::default();
        sim.touch([-1, 0, 0]);
        sim.tick(&mut a);
        ticks(&mut sim, &mut b, 5);
        assert!(b.log.is_empty(), "{:?}", b.log);
        assert_eq!(sim.pending[&([0, 0, 0], 0)].pending.remaining, 3);
        ticks(&mut sim, &mut a, 2);
        assert!(!powered(a.kind([0, 0, 0])));
        sim.tick(&mut a);
        assert!(powered(a.kind([0, 0, 0])));
    }

    #[test]
    fn frozen_retry_waits_for_its_world() {
        // A frozen network's retry comes due while another world ticks: it waits for a world
        // that has the wire.
        let mut a = TestWorld::new(MapGrid::default());
        a.put([12, 0, 0], lever(true));
        for x in 13..=17 {
            a.g.dust([x, 0, 0]);
        }
        a.unloaded.insert([1, 0, 0]);
        let mut b = TestWorld::new(MapGrid::default());
        let mut sim = Sim::default();
        sim.touch([12, 0, 0]);
        sim.tick(&mut a);
        a.unloaded.clear();
        ticks(&mut sim, &mut b, 40);
        assert!(a.wire_writes().is_empty());
        sim.tick(&mut a);
        assert!(a.wire_writes().contains(&([13, 0, 0], 15)));
    }

    #[test]
    fn range_lowered_clamps() {
        let mut w = TestWorld::new(MapGrid::default());
        w.range = Range::from_index(1);
        w.put([0, 0, 0], Kind::PowerBlock);
        for x in 1..=35 {
            w.g.dust([x, 0, 0]);
        }
        let mut sim = Sim::default();
        sim.touch([0, 0, 0]);
        sim.tick(&mut w);
        assert_eq!(w.strength([1, 0, 0]), 31);
        w.range = Range::from_index(0);
        sim.touch([0, 0, 0]);
        w.log.clear();
        sim.tick(&mut w);
        assert_eq!(w.wire_writes().iter().map(|(_, s)| *s).max(), Some(15));
        assert_eq!(w.strength([1, 0, 0]), 15);
        assert_eq!(w.strength([16, 0, 0]), 0);
    }

    #[test]
    fn extension_receiver_notified_on_change_only() {
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], lever(false)).put([40, 0, 0], lever(false));
        w.g.dust([1, 0, 0]).dust([2, 0, 0]).dust([3, 0, 0]);
        w.g.put([4, 0, 0], Cell::Device(Device { output: [Power::default(); 6], connects: 0b111111, receives: true, solid: false, kind: None }));
        let mut sim = Sim::default();
        w.put([0, 0, 0], lever(true));
        sim.touch([0, 0, 0]);
        ticks(&mut sim, &mut w, 5);
        let mut want = [SidePower::default(); 6];
        want[Dir::West.index()] = SidePower { strength: 13, strong: false, direct: true };
        assert_eq!(w.notifies(), [([4, 0, 0], want)]);
        sim.touch([0, 0, 0]);
        sim.touch([3, 0, 0]);
        w.put([40, 0, 0], lever(true));
        sim.touch([40, 0, 0]);
        ticks(&mut sim, &mut w, 3);
        assert_eq!(w.notifies().len(), 1);
        w.put([0, 0, 0], lever(false));
        sim.touch([0, 0, 0]);
        sim.tick(&mut w);
        assert_eq!(w.notifies(), [([4, 0, 0], want), ([4, 0, 0], [SidePower::default(); 6])]);
    }

    #[test]
    fn receiver_follows_strength_not_level() {
        // One more dust at the end lowers what reaches the receiver by one, a change no
        // brightness level of the wire next to it would show.
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], lever(true));
        w.g.dust([1, 0, 0]).dust([2, 0, 0]);
        w.g.put([3, 0, 0], Cell::Device(Device { output: [Power::default(); 6], connects: 0, receives: true, solid: false, kind: None }));
        let mut sim = Sim::default();
        sim.touch([0, 0, 0]);
        sim.tick(&mut w);
        let at = |s: u8| {
            let mut i = [SidePower::default(); 6];
            i[Dir::West.index()] = SidePower { strength: s, strong: false, direct: true };
            i
        };
        assert_eq!(w.notifies(), [([3, 0, 0], at(14))]);
        // Moving the lever one further away: the wire next to the receiver drops 14 → 13.
        w.g.air([0, 0, 0]).dust([0, 0, 0]);
        w.put([-1, 0, 0], lever(true));
        sim.touch([0, 0, 0]);
        sim.touch([-1, 0, 0]);
        sim.tick(&mut w);
        assert_eq!(w.notifies(), [([3, 0, 0], at(14)), ([3, 0, 0], at(13))]);
    }

    #[test]
    fn unchanged_network_solved_on_demand() {
        // A lamp fed by two networks, one from each side. When only the east one changes, the
        // west one isn't solved by the tick, and the lamp must still see it powering it.
        let mut w = TestWorld::new(MapGrid::default());
        w.put([0, 0, 0], lever(true)).put([4, 0, 0], lever(false)).put([2, 0, 0], Kind::Lamp { lit: false });
        w.g.dust([1, 0, 0]).dust([3, 0, 0]);
        let mut sim = Sim::default();
        sim.touch([0, 0, 0]);
        ticks(&mut sim, &mut w, 2);
        assert_eq!(w.kind([2, 0, 0]), Kind::Lamp { lit: true });
        for on in [true, false] {
            w.put([4, 0, 0], lever(on));
            sim.touch([4, 0, 0]);
            sim.tick(&mut w);
            assert!(!sim.has_pending([2, 0, 0], 0), "lever east {on}");
            sim.tick(&mut w);
            assert_eq!(w.kind([2, 0, 0]), Kind::Lamp { lit: true });
        }
    }
}
