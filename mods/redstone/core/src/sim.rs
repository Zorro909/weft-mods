//! The tick (spec §6, §7): apply due device changes at once, solve the wire networks they touch,
//! write each changed wire once, then let receivers react for a later tick. Everything iterates in
//! position order, so the same build gives the same result wherever it is and in whatever order it
//! was changed. Circuits reaching into unloaded cubes freeze (§7.1).

use std::collections::{BTreeMap, BTreeSet};
use std::mem::discriminant;

use crate::devices::sides_of;
use crate::network::Recorder;
use crate::{Cell, Device, Dir, Kind, Pos, Range, Shape, SidePower, View, cube_of, inputs, links, network, react, shape, solve, step};

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
    /// Sets a wire's strength and shape; true if either changed.
    fn write_wire(&mut self, p: Pos, strength: u8, shape: Shape) -> bool;
    fn write_device(&mut self, p: Pos, k: Kind);
    /// What reaches an extension receiver changed.
    fn notify(&mut self, p: Pos, inputs: [SidePower; 6]);
    /// Stores (or clears) a device's pending change so it survives unload and restart; `true` when
    /// it is frozen and not counting down.
    fn save_pending(&mut self, p: Pos, pending: Option<(Pending, bool)>);
}

#[derive(Default)]
pub struct Sim {
    pending: BTreeMap<Pos, Entry>,
    dirty: BTreeSet<Pos>,
    /// Positions in a frozen circuit, retried when their countdown runs out.
    frozen_dirty: BTreeMap<Pos, u32>,
    /// What extension receivers were last told, only non-zero ones.
    last_inputs: BTreeMap<Pos, [SidePower; 6]>,
}

struct Entry {
    pending: Pending,
    /// The cubes its circuit spans; `None` until the next tick works them out.
    cubes: Option<BTreeSet<[i32; 3]>>,
    frozen: bool,
}

impl Sim {
    /// Something at `p` changed: a device, a block, a wire.
    pub fn touch(&mut self, p: Pos) {
        self.dirty.insert(p);
    }

    /// Starts (or replaces) the pending change of the device at `p`. It is saved at the start of
    /// the next tick.
    pub fn schedule(&mut self, p: Pos, pending: Pending) {
        self.pending.insert(p, Entry { pending, cubes: None, frozen: false });
    }

    /// The device at `p` is gone.
    pub fn forget(&mut self, p: Pos) {
        self.pending.remove(&p);
        self.frozen_dirty.remove(&p);
        self.last_inputs.remove(&p);
    }

    pub fn has_pending(&self, p: Pos) -> bool {
        self.pending.contains_key(&p)
    }

    pub fn tick(&mut self, w: &mut impl World) {
        let range = w.range();

        // 1. A pending change whose circuit reaches an unloaded cube stops counting down.
        for (p, e) in &mut self.pending {
            let fresh = e.cubes.is_none();
            let cubes = e.cubes.get_or_insert_with(|| reach(&*w, *p));
            let frozen = !cubes.iter().all(|c| w.cube_loaded(*c));
            if fresh || frozen != e.frozen {
                w.save_pending(*p, Some((e.pending, frozen)));
            }
            e.frozen = frozen;
        }

        // 2. Apply every change due now, all at once.
        let mut due = Vec::new();
        for (p, e) in &mut self.pending {
            if !e.frozen {
                e.pending.remaining = e.pending.remaining.saturating_sub(1);
                if e.pending.remaining == 0 {
                    due.push(*p);
                }
            }
        }
        for p in due {
            let e = self.pending.remove(&p).expect("due entry");
            let next = e.pending.next;
            // Dropped if the device was removed or replaced since.
            if let Cell::Device(Device { kind: Some(k), .. }) = w.cell(p)
                && discriminant(&k) == discriminant(&next)
            {
                w.write_device(p, next);
                w.save_pending(p, None);
                self.dirty.insert(p);
            }
        }
        self.frozen_dirty.retain(|p, n| {
            *n -= 1;
            if *n == 0 {
                self.dirty.insert(*p);
            }
            *n > 0
        });

        // 3 + 4. Solve each network next to a change once and write the wires that changed.
        let mut seeds = BTreeSet::new();
        for p in &self.dirty {
            around(&*w, *p, &mut seeds);
        }
        let mut seen = BTreeSet::new();
        let mut changed = Vec::new();
        for s in seeds {
            if seen.contains(&s) || !matches!(w.cell(s), Cell::Wire(_)) {
                continue;
            }
            let net = network(&*w, s);
            seen.extend(net.wires.iter().copied());
            if net.incomplete {
                self.frozen_dirty.entry(s).or_insert(RETRY);
                continue;
            }
            for (p, strength) in solve(&*w, &net, range) {
                let Cell::Wire(wire) = w.cell(p) else { continue };
                let sh = shape(links(&*w, p, &wire));
                if w.write_wire(p, strength, sh) {
                    changed.push(p);
                }
            }
        }

        // 5. Receivers whose inputs may have changed react, for a later tick.
        let mut candidates = BTreeSet::new();
        for p in self.dirty.iter().chain(&changed) {
            around(&*w, *p, &mut candidates);
        }
        for p in candidates {
            let Cell::Device(dev) = w.cell(p) else { continue };
            if !dev.receives {
                continue;
            }
            let r = Recorder::new(&*w);
            let i = inputs(&r, p);
            if r.unloaded.get() {
                self.frozen_dirty.entry(p).or_insert(RETRY);
                continue;
            }
            match dev.kind {
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
    }

    fn react(&mut self, w: &mut impl World, p: Pos, k: Kind, i: [SidePower; 6], range: Range) {
        let container = if matches!(k, Kind::Comparator { .. }) { w.container(p) } else { None };
        let lockers = match k {
            Kind::Repeater { facing, .. } => side_lockers(&*w, p, facing),
            _ => false,
        };
        if let Some(e) = self.pending.get(&p) {
            // One pending change per device, except that locking or unlocking a repeater replaces
            // a change that would leave it in the wrong lock state.
            match e.pending.next {
                Kind::Repeater { locked, .. } if locked != lockers => {
                    self.pending.remove(&p);
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
            self.pending.insert(p, Entry { pending, cubes: Some(cubes), frozen });
        }
    }
}

/// Ticks between retries of a frozen circuit (spec §7.1).
const RETRY: u32 = 20;

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
    use crate::{Power, device};

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

    /// A grid whose built-in devices take their outputs from their kind and the current range,
    /// like the adapter does, with unloadable cubes and a log of everything written.
    struct TestWorld {
        g: MapGrid,
        range: Range,
        unloaded: BTreeSet<[i32; 3]>,
        shapes: BTreeMap<Pos, Shape>,
        log: Vec<Ev>,
    }

    impl TestWorld {
        fn new(g: MapGrid) -> TestWorld {
            TestWorld { g, range: R15, unloaded: BTreeSet::new(), shapes: BTreeMap::new(), log: Vec::new() }
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
        fn strength(&self, p: Pos) -> u8 {
            match self.g.cell(p) {
                Cell::Wire(w) => w.strength,
                c => panic!("no wire at {p:?}: {c:?}"),
            }
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
        fn container(&self, _: Pos) -> Option<u8> {
            None
        }
        fn write_wire(&mut self, p: Pos, strength: u8, shape: Shape) -> bool {
            let Some(Cell::Wire(w)) = self.g.cells.get_mut(&p) else { panic!("write_wire on non-wire {p:?}") };
            if w.strength == strength && self.shapes.get(&p) == Some(&shape) {
                return false;
            }
            w.strength = strength;
            self.shapes.insert(p, shape);
            self.log.push(Ev::Wire(p, strength));
            true
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
        assert_eq!(sim.pending[&[6, 0, 0]].pending, Pending { next: Kind::Lamp { lit: true }, remaining: 1 });
        w.log.clear();
        sim.tick(&mut w);
        assert_eq!(w.kind([6, 0, 0]), Kind::Lamp { lit: true });
        assert!(w.wire_writes().is_empty());
        assert!(!sim.has_pending([6, 0, 0]));
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
            w.g
        };
        let done = run(start.clone(), 0);
        assert_eq!(done.cell([4, 0, 4]), Cell::Device(device(Kind::Lamp { lit: false }, R15)));
        assert!(matches!(done.cell([2, 0, 2]), Cell::Device(Device { kind: Some(Kind::Repeater { powered: true, .. }), .. })));
        let (mut g, mut want) = (start, done);
        for k in 1..=3 {
            g = g.rotated();
            want = want.rotated();
            assert_eq!(run(g.clone(), k).cells, want.cells, "after {k} turns");
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
        assert_eq!(sim.pending[&[0, 0, 0]].pending.remaining, 3);
        sim.tick(&mut w);
        w.unloaded.insert([0, 0, 0]);
        w.log.clear();
        ticks(&mut sim, &mut w, 10);
        assert_eq!(w.device_writes([0, 0, 0]), 0);
        let next = Kind::Repeater { facing: Dir::East, delay: 3, powered: true, locked: false };
        assert_eq!(w.log, [Ev::Save([0, 0, 0], Some((Pending { next, remaining: 2 }, true)))]);
        w.unloaded.clear();
        sim.tick(&mut w);
        assert_eq!(w.device_writes([0, 0, 0]), 0);
        sim.tick(&mut w);
        assert!(powered(w.kind([0, 0, 0])));
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
        assert!(sim.has_pending([0, 1, 0]));
        w.g.air([0, 1, 0]);
        sim.tick(&mut w);
        assert_eq!(w.device_writes([0, 1, 0]), 0);
        assert!(!sim.has_pending([0, 1, 0]));
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
}
