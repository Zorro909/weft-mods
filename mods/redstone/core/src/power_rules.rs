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

/// Whether wire at `wire` weak-powers its neighbor on side `toward`: always the block it sits on.
fn wire_powers(_v: &impl View, _wire: Pos, toward: Dir) -> bool {
    toward == Dir::Down
}

/// The strength wire at `p` gets from outside its network: adjacent devices and strongly powered
/// blocks, never weakly powered ones.
pub fn source_into_wire(v: &impl View, p: Pos) -> u8 {
    let mut best = 0;
    for d in Dir::ALL {
        let q = step(p, d);
        let s = match v.cell(q) {
            Cell::Device(dev) => dev.output[d.opposite().index()].strength,
            Cell::Block { solid: true, .. } => block_power(v, q).strong,
            _ => 0,
        };
        best = best.max(s);
    }
    best
}

/// What reaches each side of a receiver at `p`.
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
    fn non_solid_block_not_powered() {
        let mut g = MapGrid::default();
        g.put([0, 1, 0], lever(15)).put([0, 0, 0], Cell::block(false));
        assert_eq!(block_power(&g, [0, 0, 0]), BlockPower { strong: 0, weak: 0 });
    }
}
