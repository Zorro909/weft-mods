//! Wire networks: everything one wire connects to, solved as a unit (spec §6).

use std::cell::{Cell as StdCell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use crate::{Cell, Dir, Link, Pos, Range, View, links, source_into_wire, step};

pub struct Network {
    pub wires: BTreeSet<Pos>,
    /// The wires each wire connects to.
    pub edges: BTreeMap<Pos, Vec<Pos>>,
    /// Something it needed to read isn't loaded: the network can't be solved (it freezes).
    pub incomplete: bool,
    /// The cubes (16³) of every position read, to check later that all of them are still loaded.
    pub cubes: BTreeSet<[i32; 3]>,
}

/// The network of the wire at `start`.
pub fn network(v: &impl View, start: Pos) -> Network {
    let r = Recorder { v, cubes: RefCell::new(BTreeSet::new()), unloaded: StdCell::new(false) };
    let mut wires = BTreeSet::new();
    let mut edges = BTreeMap::new();
    let mut queue = std::collections::VecDeque::from([start]);
    while let Some(p) = queue.pop_front() {
        if wires.contains(&p) {
            continue;
        }
        let Cell::Wire(w) = r.cell(p) else { continue };
        wires.insert(p);
        let ls = links(&r, p, &w);
        let mut next = Vec::new();
        for (d, l) in Dir::HORIZONTAL.into_iter().zip(ls) {
            let q = step(p, d);
            let n = match l {
                Link::None => continue,
                // Flat may also be a connectable device, which isn't part of the network.
                Link::Flat => q,
                Link::Up => step(q, Dir::Up),
                Link::Down => step(q, Dir::Down),
            };
            if matches!(r.cell(n), Cell::Wire(_)) {
                next.push(n);
            }
        }
        // Read what feeds it too, so an unloaded source makes the network incomplete.
        source_into_wire(&r, p);
        queue.extend(next.iter().copied());
        edges.insert(p, next);
    }
    Network { wires, edges, incomplete: r.unloaded.get(), cubes: r.cubes.into_inner() }
}

/// A view that notes which cubes were read and whether any position was unloaded.
struct Recorder<'a, V> {
    v: &'a V,
    cubes: RefCell<BTreeSet<[i32; 3]>>,
    unloaded: StdCell<bool>,
}

impl<V: View> View for Recorder<'_, V> {
    fn cell(&self, p: Pos) -> Cell {
        self.cubes.borrow_mut().insert(p.map(|c| c.div_euclid(16)));
        let c = self.v.cell(p);
        if c == Cell::Unloaded {
            self.unloaded.set(true);
        }
        c
    }
}

/// The strength of every wire in `net`: the best of its sources minus the distance, never below 0.
pub fn solve(v: &impl View, net: &Network, range: Range) -> BTreeMap<Pos, u8> {
    let mut best: BTreeMap<Pos, u8> = net.wires.iter().map(|p| (*p, 0)).collect();
    // A bucket per strength, highest first: each wire is settled by its strongest arrival.
    let mut buckets: Vec<Vec<Pos>> = vec![Vec::new(); range.max() as usize + 1];
    for p in &net.wires {
        let s = range.clamp(source_into_wire(v, *p));
        if s > 0 {
            best.insert(*p, s);
            buckets[s as usize].push(*p);
        }
    }
    for s in (1..buckets.len()).rev() {
        let here = std::mem::take(&mut buckets[s]);
        for p in here {
            if best[&p] != s as u8 {
                continue;
            }
            for n in net.edges.get(&p).into_iter().flatten() {
                let to = s as u8 - 1;
                if best.get(n).is_some_and(|b| *b < to) {
                    best.insert(*n, to);
                    buckets[to as usize].push(*n);
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testgrid::{MapGrid, rotate_pos};
    use crate::{Device, FULL, Power, Wire};

    /// A source giving `s` weak to every side.
    fn source(s: u8) -> Cell {
        Cell::Device(Device { output: [Power { strength: s, strong: false }; 6], connects: 0b111111, receives: false, solid: false, kind: None })
    }

    #[test]
    fn fade_one_per_block_each_range() {
        for (i, max) in [(0, 15), (1, 31), (2, 63)] {
            let mut g = MapGrid::default();
            g.put([-1, 0, 0], source(FULL));
            for x in 0..=max + 1 {
                g.dust([x, 0, 0]);
            }
            let s = solve(&g, &network(&g, [0, 0, 0]), Range::from_index(i));
            assert_eq!(s[&[0, 0, 0]], max as u8);
            assert_eq!(s[&[max - 1, 0, 0]], 1);
            assert_eq!(s[&[max, 0, 0]], 0);
            assert_eq!(s.len(), max as usize + 2);
        }
    }

    #[test]
    fn highest_source_wins() {
        let mut g = MapGrid::default();
        g.put([-1, 0, 0], source(15)).put([11, 0, 0], source(15));
        for x in 0..=10 {
            g.dust([x, 0, 0]);
        }
        let s = solve(&g, &network(&g, [0, 0, 0]), Range::from_index(0));
        assert_eq!(s[&[5, 0, 0]], 10);
        assert_eq!(s[&[9, 0, 0]], 14);
    }

    #[test]
    fn loop_without_source_is_zero() {
        let mut g = MapGrid::default();
        for p in [[0, 0, 0], [1, 0, 0], [2, 0, 0], [2, 0, 1], [2, 0, 2], [1, 0, 2], [0, 0, 2], [0, 0, 1]] {
            g.put(p, Cell::Wire(Wire::dust(9)));
        }
        let n = network(&g, [0, 0, 0]);
        assert_eq!(n.wires.len(), 8);
        assert!(solve(&g, &n, Range::from_index(0)).values().all(|&x| x == 0));
    }

    #[test]
    fn unloaded_marks_incomplete() {
        let mut g = MapGrid::default();
        g.dust([0, 0, 0]).dust([1, 0, 0]).unloaded([2, 0, 0]);
        assert!(network(&g, [0, 0, 0]).incomplete);
        g.air([2, 0, 0]);
        assert!(!network(&g, [0, 0, 0]).incomplete);
    }

    #[test]
    fn unloaded_source_side_marks_incomplete() {
        // The source of a block two away could be unloaded: a strongly powered block's power is read too.
        let mut g = MapGrid::default();
        g.dust([0, 0, 0]).solid([0, 0, 1]).unloaded([0, 0, 2]);
        assert!(network(&g, [0, 0, 0]).incomplete);
    }

    #[test]
    fn cubes_cover_wires() {
        let mut g = MapGrid::default();
        g.dust([15, 0, 0]).dust([16, 0, 0]);
        let n = network(&g, [15, 0, 0]);
        assert!(n.cubes.contains(&[0, 0, 0]) && n.cubes.contains(&[1, 0, 0]));
        assert_eq!(n.wires.len(), 2);
    }

    #[test]
    fn steps_are_part_of_the_network() {
        let mut g = MapGrid::default();
        g.put([-1, 0, 0], source(15)).dust([0, 0, 0]).solid([1, 0, 0]).dust([1, 1, 0]).dust([2, 1, 0]);
        let s = solve(&g, &network(&g, [0, 0, 0]), Range::from_index(0));
        assert_eq!(s[&[2, 1, 0]], 13);
    }

    #[test]
    fn solve_rotation_invariant() {
        let mut g = MapGrid::default();
        let mut out = [Power::default(); 6];
        out[Dir::East.index()] = Power { strength: 12, strong: false };
        g.put([-1, 0, 0], Cell::Device(Device { output: out, connects: 0b111111, receives: false, solid: false, kind: None }));
        g.dust([0, 0, 0]).dust([1, 0, 0]).dust([1, 0, 1]).dust([1, 0, 2]).solid([2, 0, 2]).dust([2, 1, 2]).dust([0, 0, 2]).put([0, 0, 3], source(7));
        let base = solve(&g, &network(&g, [0, 0, 0]), Range::from_index(0));
        let mut r = g.clone();
        for k in 1..=3 {
            r = r.rotated();
            let start = (0..k).fold([0, 0, 0], |q, _| rotate_pos(q));
            let got = solve(&r, &network(&r, start), Range::from_index(0));
            for (p, s) in &base {
                let rp = (0..k).fold(*p, |q, _| rotate_pos(q));
                assert_eq!(got[&rp], *s, "{p:?} after {k} turns");
            }
        }
    }
}
