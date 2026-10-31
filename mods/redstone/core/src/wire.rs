//! How wire connects and what its shape points into (spec §8): Minecraft Java's rules, except that
//! wire alone is always a cross and every connection works both ways.

use crate::{Cell, Device, Dir, Pos, View, Wire, step};

/// A wire's connection on one horizontal side.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Link {
    None,
    Flat,
    /// Climbs the side of the neighboring block to wire on top of it.
    Up,
    /// Runs down the side of this block to wire one lower.
    Down,
}

/// The connections of wire `w` at `p`, by `Dir::HORIZONTAL`.
pub fn links(v: &impl View, p: Pos, w: &Wire) -> [Link; 4] {
    links_from(v, p, w, &Dir::ALL.map(|d| v.cell(step(p, d))))
}

/// `links` with the cells around `p` already read, by `Dir::ALL`.
pub(crate) fn links_from(v: &impl View, p: Pos, w: &Wire, around: &[Cell; 6]) -> [Link; 4] {
    Dir::HORIZONTAL.map(|d| {
        let q = step(p, d);
        match &around[d.index()] {
            Cell::Wire(b) if linked(w, b) => return Link::Flat,
            Cell::Device(dev) if dev.connects_on(d.opposite()) => return Link::Flat,
            _ => {}
        }
        if stair(v, p, d, w) {
            Link::Up
        } else if stair(v, step(q, Dir::Down), d.opposite(), w) {
            Link::Down
        } else {
            Link::None
        }
    })
}

/// Two wire kinds connect when each lists the other. Kinds past 31 have no bit and only connect
/// to their own kind.
pub fn linked(a: &Wire, b: &Wire) -> bool {
    let has = |w: &Wire, kind: u32| if kind < 32 { w.links & (1 << kind) != 0 } else { w.kind == kind };
    has(a, b.kind) && has(b, a.kind)
}

/// A step from wire at `lower` up to wire on top of its neighbor on side `d`, linked to `w`: the
/// neighbor carries the upper wire, and no solid block sits right above the lower one. The same
/// test from either end, so steps always work both ways. Anything unloaded: no step.
fn stair(v: &impl View, lower: Pos, d: Dir, w: &Wire) -> bool {
    let side = step(lower, d);
    // In this order, stopping at the first no: most sides of most wires carry nothing.
    let carries = match v.cell(side) {
        Cell::Block { .. } => true,
        Cell::Device(dev) => dev.solid,
        _ => false,
    };
    carries
        && matches!(v.cell(lower), Cell::Wire(b) if linked(w, &b))
        && matches!(v.cell(step(side, Dir::Up)), Cell::Wire(b) if linked(w, &b))
        && matches!(v.cell(step(lower, Dir::Up)), Cell::Air | Cell::Wire(_) | Cell::Block { solid: false, .. } | Cell::Device(Device { solid: false, .. }))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shape {
    pub sides: [Link; 4],
    /// The sides the shape runs to, by `Dir::HORIZONTAL`.
    pub points: [bool; 4],
}

pub fn shape(links: [Link; 4]) -> Shape {
    let linked: Vec<usize> = (0..4).filter(|&i| links[i] != Link::None).collect();
    let points = match linked.as_slice() {
        [] => [true; 4],
        // Index pairs in `Dir::HORIZONTAL` (west/east, north/south) differ in the lowest bit.
        [i] => std::array::from_fn(|j| j == *i || j == i ^ 1),
        _ => links.map(|l| l != Link::None),
    };
    Shape { sides: links, points }
}

/// Whether wire at `wire_pos` points into `target`, one of its horizontal neighbors, so it
/// weak-powers it.
pub fn points_into(v: &impl View, wire_pos: Pos, target: Pos) -> bool {
    let Cell::Wire(w) = v.cell(wire_pos) else { return false };
    let Some(i) = Dir::HORIZONTAL.iter().position(|d| step(wire_pos, *d) == target) else { return false };
    let s = shape(links(v, wire_pos, &w));
    // Climbing a block's side isn't pointing into it.
    s.points[i] && s.sides[i] != Link::Up
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testgrid::{MapGrid, rotate_pos};
    use crate::{Power, block_power};

    fn dust() -> Wire {
        Wire::dust(0)
    }

    fn shape_at(g: &MapGrid, p: Pos) -> Shape {
        let Cell::Wire(w) = g.cell(p) else { panic!("no wire at {p:?}") };
        shape(links(g, p, &w))
    }

    #[test]
    fn alone_is_cross() {
        let mut g = MapGrid::default();
        g.dust([0, 0, 0]);
        assert_eq!(shape_at(&g, [0, 0, 0]).points, [true; 4]);
    }

    #[test]
    fn one_link_is_line_through() {
        let mut g = MapGrid::default();
        g.dust([0, 0, 0]).dust([-1, 0, 0]);
        let s = shape_at(&g, [0, 0, 0]);
        assert_eq!(s.sides, [Link::Flat, Link::None, Link::None, Link::None]);
        assert_eq!(s.points, [true, true, false, false]);
    }

    #[test]
    fn corner_points_only_connected() {
        let mut g = MapGrid::default();
        g.dust([0, 0, 0]).dust([-1, 0, 0]).dust([0, 0, -1]);
        assert_eq!(shape_at(&g, [0, 0, 0]).points, [true, false, true, false]);
    }

    #[test]
    fn step_up_connects_both_ways() {
        let mut g = MapGrid::default();
        g.dust([0, 0, 0]).solid([1, 0, 0]).dust([1, 1, 0]);
        assert_eq!(links(&g, [0, 0, 0], &dust())[1], Link::Up);
        assert_eq!(links(&g, [1, 1, 0], &dust())[0], Link::Down);
    }

    #[test]
    fn solid_above_lower_dust_cuts_step() {
        let mut g = MapGrid::default();
        g.dust([0, 0, 0]).solid([1, 0, 0]).dust([1, 1, 0]).solid([0, 1, 0]);
        assert_eq!(links(&g, [0, 0, 0], &dust())[1], Link::None);
        assert_eq!(links(&g, [1, 1, 0], &dust())[0], Link::None);
    }

    #[test]
    fn glass_stairs_two_way() {
        // Lower dust, then dust on a glass block, then dust on two glass blocks.
        let mut g = MapGrid::default();
        g.dust([0, 0, 0]).put([1, 0, 0], Cell::block(false)).dust([1, 1, 0]);
        g.put([2, 0, 0], Cell::block(false)).put([2, 1, 0], Cell::block(false)).dust([2, 2, 0]);
        assert_eq!(links(&g, [0, 0, 0], &dust())[1], Link::Up);
        assert_eq!(links(&g, [1, 1, 0], &dust()), [Link::Down, Link::Up, Link::None, Link::None]);
        assert_eq!(links(&g, [2, 2, 0], &dust())[0], Link::Down);
    }

    #[test]
    fn connectable_device_side() {
        let mut g = MapGrid::default();
        let dev = |connects| Cell::Device(Device { output: [Power::default(); 6], connects, receives: true, solid: false, kind: None });
        g.dust([0, 0, 0]).put([-1, 0, 0], dev(1 << Dir::East.index())).put([1, 0, 0], dev(1 << Dir::East.index()));
        assert_eq!(links(&g, [0, 0, 0], &dust())[..2], [Link::Flat, Link::None]);
    }

    #[test]
    fn insulated_kind_links_only_itself() {
        let blue = Wire { kind: 1, links: 1 << 1, powers_blocks: false, strength: 0 };
        let mut g = MapGrid::default();
        g.put([0, 0, 0], Cell::Wire(blue.clone())).dust([-1, 0, 0]).put([1, 0, 0], Cell::Wire(blue.clone()));
        assert_eq!(links(&g, [0, 0, 0], &blue)[..2], [Link::None, Link::Flat]);
        assert_eq!(links(&g, [-1, 0, 0], &dust())[1], Link::None);
    }

    #[test]
    fn weak_power_into_pointed_block() {
        // A line from the west ending at a solid block: powers it. A corner beside a block: doesn't.
        let mut g = MapGrid::default();
        g.put([-1, 0, 0], Cell::Wire(Wire::dust(6))).put([0, 0, 0], Cell::Wire(Wire::dust(5))).solid([1, 0, 0]);
        assert_eq!(block_power(&g, [1, 0, 0]).weak, 5);
        let mut g = MapGrid::default();
        g.dust([-1, 0, 0]).dust([0, 0, -1]).put([0, 0, 0], Cell::Wire(Wire::dust(5))).solid([1, 0, 0]);
        assert_eq!(block_power(&g, [1, 0, 0]).weak, 0);
        assert!(!points_into(&g, [0, 0, 0], [1, 0, 0]));
    }

    #[test]
    fn climbed_block_not_powered() {
        let mut g = MapGrid::default();
        g.put([0, 0, 0], Cell::Wire(Wire::dust(9))).solid([1, 0, 0]).dust([1, 1, 0]);
        assert!(!points_into(&g, [0, 0, 0], [1, 0, 0]));
    }

    #[test]
    fn shapes_rotation_invariant() {
        let mut g = MapGrid::default();
        g.dust([0, 0, 0]).dust([1, 0, 0]).dust([1, 0, 1]).solid([2, 0, 0]).dust([2, 1, 0]).dust([3, 1, 0]).solid([0, 1, 1]).dust([-2, 0, 0]);
        let mut r = g.clone();
        for k in 1..=3 {
            r = r.rotated();
            for p in g.cells.iter().filter(|(_, c)| matches!(c, Cell::Wire(_))).map(|(p, _)| *p) {
                let rp = (0..k).fold(p, |q, _| rotate_pos(q));
                let (a, b) = (shape_at(&g, p), shape_at(&r, rp));
                for (i, d) in Dir::HORIZONTAL.iter().enumerate() {
                    let rd = (0..k).fold(*d, |x, _| x.rotate_y());
                    let j = Dir::HORIZONTAL.iter().position(|x| *x == rd).unwrap();
                    assert_eq!((a.sides[i], a.points[i]), (b.sides[j], b.points[j]), "side {d:?} of {p:?} after {k} turns");
                }
            }
        }
    }
}
