//! Strong and weak power (spec §5): devices strong-power blocks, wire weak-powers them, and only
//! strong power reaches wire through a block.

use crate::{Cell, Dir, Pos, SidePower, View, step};

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct BlockPower {
    pub strong: u8,
    pub weak: u8,
}

/// How strongly and weakly the block at `p` is powered; zero unless it's a solid block.
pub fn block_power(v: &impl View, p: Pos) -> BlockPower {
    if !matches!(v.cell(p), Cell::Block { solid: true, .. }) {
        return BlockPower::default();
    }
    let mut bp = BlockPower::default();
    for d in Dir::ALL {
        match v.cell(step(p, d)) {
            Cell::Device(dev) => {
                let out = dev.output[d.opposite().index()];
                if out.strong {
                    bp.strong = bp.strong.max(out.strength);
                }
            }
            Cell::Wire(w) if w.powers_blocks && wire_powers(v, step(p, d), d.opposite()) => bp.weak = bp.weak.max(w.strength),
            _ => {}
        }
    }
    bp
}

/// The strong power of the solid block at `p`: only devices give it, so only its neighbors are
/// read (`block_power` also works out the weak power wire gives it).
fn strong_into(v: &impl View, p: Pos) -> u8 {
    Dir::ALL
        .iter()
        .map(|d| match v.cell(step(p, *d)) {
            Cell::Device(dev) if dev.output[d.opposite().index()].strong => dev.output[d.opposite().index()].strength,
            _ => 0,
        })
        .max()
        .unwrap_or(0)
}

/// Whether wire at `wire` reaches its neighbor on side `toward`: the block it sits on, and what
/// its shape points into.
fn wire_powers(v: &impl View, wire: Pos, toward: Dir) -> bool {
    toward == Dir::Down || (toward != Dir::Up && crate::points_into(v, wire, step(wire, toward)))
}

/// The strength wire at `p` gets from outside its network: adjacent devices and strongly powered
/// blocks, never weakly powered ones.
pub fn source_into_wire(v: &impl View, p: Pos) -> u8 {
    source_from(v, p, &Dir::ALL.map(|d| v.cell(step(p, d))))
}

/// `source_into_wire` with the cells around `p` already read, by `Dir::ALL`.
pub(crate) fn source_from(v: &impl View, p: Pos, around: &[Cell; 6]) -> u8 {
    let mut best = 0;
    for d in Dir::ALL {
        let q = step(p, d);
        let s = match &around[d.index()] {
            Cell::Device(dev) => dev.output[d.opposite().index()].strength,
            Cell::Block { solid: true, .. } => strong_into(v, q),
            _ => 0,
        };
        best = best.max(s);
    }
    best
}

/// What reaches each side of a receiver at `p`. A side nothing reaches (strength 0) is always
/// `SidePower::default()`, so unpowered wire next to a receiver isn't a change.
pub fn inputs(v: &impl View, p: Pos) -> [SidePower; 6] {
    let mut i = [SidePower::default(); 6];
    for d in Dir::ALL {
        let q = step(p, d);
        i[d.index()] = match v.cell(q) {
            Cell::Device(dev) => {
                let out = dev.output[d.opposite().index()];
                SidePower { strength: out.strength, strong: out.strong, direct: true }
            }
            Cell::Wire(w) if wire_powers(v, q, d.opposite()) => SidePower { strength: w.strength, strong: false, direct: true },
            Cell::Block { solid: true, .. } => {
                let bp = block_power(v, q);
                if bp.strong > 0 {
                    SidePower { strength: bp.strong, strong: true, direct: false }
                } else {
                    SidePower { strength: bp.weak, strong: false, direct: false }
                }
            }
            _ => SidePower::default(),
        };
        if i[d.index()].strength == 0 {
            i[d.index()] = SidePower::default();
        }
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testgrid::MapGrid;
    use crate::{Device, Power, Wire};

    /// A lever-like source: `down_strong` strong into the block below, 15 weak on the sides.
    fn lever(down_strong: u8) -> Cell {
        let mut o = [Power::default(); 6];
        o[Dir::Down.index()] = Power { strength: down_strong, strong: true };
        for d in Dir::HORIZONTAL {
            o[d.index()] = Power { strength: 15, strong: false };
        }
        Cell::Device(Device { output: o, connects: 0b111111, receives: false, solid: false, kind: None })
    }

    #[test]
    fn strong_block_powers_dust() {
        let mut g = MapGrid::default();
        g.put([0, 1, 0], lever(15)).solid([0, 0, 0]).dust([1, 0, 0]);
        assert_eq!(source_into_wire(&g, [1, 0, 0]), 15);
    }

    #[test]
    fn weak_block_never_powers_dust() {
        let mut g = MapGrid::default();
        g.solid([0, 0, 0]).put([0, 1, 0], Cell::Wire(Wire { kind: 0, links: u32::MAX, powers_blocks: true, strength: 9 })).dust([1, 0, 0]);
        assert_eq!(block_power(&g, [0, 0, 0]).weak, 9);
        assert_eq!(source_into_wire(&g, [1, 0, 0]), 0);
    }

    #[test]
    fn device_powers_adjacent_dust_directly() {
        let mut g = MapGrid::default();
        g.put([0, 0, 0], lever(15)).dust([1, 0, 0]);
        assert_eq!(source_into_wire(&g, [1, 0, 0]), 15);
    }

    #[test]
    fn receiver_sees_kind_and_side() {
        let mut g = MapGrid::default();
        g.put([0, 1, 0], lever(15)).solid([0, 0, 0]);
        let i = inputs(&g, [1, 0, 0]);
        assert_eq!(i[Dir::West.index()], SidePower { strength: 15, strong: true, direct: false });
    }

    #[test]
    fn unpowered_side_is_default() {
        let mut g = MapGrid::default();
        g.dust([0, 0, 0]).put([2, 0, 0], lever(0));
        let i = inputs(&g, [1, 0, 0]);
        assert_eq!(i[Dir::West.index()], SidePower::default());
        assert_eq!(i[Dir::Down.index()], SidePower::default());
        assert_eq!(i[Dir::East.index()], SidePower { strength: 15, strong: false, direct: true });
    }

    #[test]
    fn non_solid_block_not_powered() {
        let mut g = MapGrid::default();
        g.put([0, 1, 0], lever(15)).put([0, 0, 0], Cell::block(false));
        assert_eq!(block_power(&g, [0, 0, 0]), BlockPower { strong: 0, weak: 0 });
    }
}
