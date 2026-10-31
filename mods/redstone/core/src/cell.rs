use crate::{Dir, Kind, Pos, Power};

/// What a position looks like to the redstone rules.
#[derive(Clone, PartialEq, Debug)]
pub enum Cell {
    /// Not loaded: whatever reads it can't finish and freezes.
    Unloaded,
    Air,
    /// Any block that isn't redstone. Only solid ones can be powered.
    Block { solid: bool, push: Push },
    Wire(Wire),
    Device(Device),
}

/// What a piston does with a block in its way.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Push {
    Move,
    Break,
    Never,
    /// A moving block's destination, held free until it lands (only a piston resuming its own
    /// move after a restart goes through it).
    Reserved,
}

impl Cell {
    /// A plain block; solid ones move with pistons, others break.
    pub fn block(solid: bool) -> Cell {
        Cell::Block { solid, push: if solid { Push::Move } else { Push::Break } }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Wire {
    /// Dust is kind 0. Two wires connect when each one's `links` has the other's kind bit.
    pub kind: u32,
    pub links: u32,
    /// Weak-powers the block below and the blocks its shape points into (dust does, insulated
    /// wire doesn't).
    pub powers_blocks: bool,
    /// Worlds don't store it and report 0: strength follows from the sources, so the tick solves
    /// it when it's needed (`sim::Solved`).
    pub strength: u8,
}

impl Wire {
    pub fn dust(strength: u8) -> Wire {
        Wire { kind: 0, links: u32::MAX, powers_blocks: true, strength }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Device {
    /// What it emits into its neighbor on each side (by `Dir::index`), strengths already clamped
    /// to the world's range.
    pub output: [Power; 6],
    /// Sides wire connects to, a bit per `Dir::index`.
    pub connects: u8,
    pub receives: bool,
    pub solid: bool,
    /// The built-in device it is; `None` for other mods' blocks.
    pub kind: Option<Kind>,
}

impl Device {
    pub fn connects_on(&self, d: Dir) -> bool {
        self.connects & (1 << d.index()) != 0
    }
}

/// The world as the rules read it.
pub trait View {
    fn cell(&self, p: Pos) -> Cell;
}
