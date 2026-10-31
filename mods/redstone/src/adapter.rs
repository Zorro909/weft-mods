//! Weft's world as the redstone rules see it (`redstone_core::World`).
//!
//! Loaded or not (G1): Weft reads unloaded positions as air, and `set` fails only where nothing is
//! loaded, so an air read is probed with `set(p, air)`, which changes nothing where it succeeds.
//! Above a dimension's top and below its bottom nothing is ever loaded either, and a mod can't
//! ask where those are. The rule: an unloaded position right above a loaded cube's top layer, or
//! right below a loaded cube's bottom layer, is the edge of the dimension and reads as air. The
//! price: a circuit crossing the top or bottom of a player's loaded column (simulation distance
//! straight up or down) is solved without the part beyond it instead of freezing.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

use redstone_api::{COMPARATOR_READING, CONNECTS, EMITS, INPUT_CHANGED, Inputs, OUTPUT, RECEIVER, SidePowerV, WIRE};
use redstone_core::{Cell, Device, Kind, Pending, Pos, Power, Range, Shape, SidePower, View, Wire, container_level, cube_of, network, solve, step};
use weft_sdk::abi::block_flags;
use weft_sdk::{World, block_of, max_stack, query, state_flags, state_with};

use crate::{blocks, persist};

pub struct WeftWorld<'a> {
    w: &'a World,
    range: Range,
    /// Whether a cube is loaded, probed once per tick.
    cubes: RefCell<HashMap<[i32; 3], bool>>,
    /// Every block this tick wrote, so `on_neighbor` can tell them from other changes.
    pub own: Vec<(Pos, u32)>,
}

impl<'a> WeftWorld<'a> {
    pub fn new(w: &'a World) -> WeftWorld<'a> {
        WeftWorld { w, range: redstone_api::range(), cubes: RefCell::new(HashMap::new()), own: Vec::new() }
    }

    /// Whether `p` is loaded, for a position known to read as air (so the probe writes air over
    /// air, which changes nothing).
    fn probe_air(&self, p: Pos) -> bool {
        *self.cubes.borrow_mut().entry(cube_of(p)).or_insert_with(|| self.w.set(p, 0))
    }

    /// Whether the cube containing `p` is loaded.
    fn probe(&self, p: Pos) -> bool {
        if let Some(l) = self.cubes.borrow().get(&cube_of(p)) {
            return *l;
        }
        if self.w.get(p) != 0 {
            self.cubes.borrow_mut().insert(cube_of(p), true);
            return true;
        }
        self.probe_air(p)
    }

    /// An air read at `p`: air, unloaded, or past the dimension's top or bottom (module docs).
    fn air_or_unloaded(&self, p: Pos) -> Cell {
        if self.probe_air(p) {
            return Cell::Air;
        }
        let [x, y, z] = p;
        let edge = (y.rem_euclid(16) == 0 && self.probe([x, y - 1, z])) || (y.rem_euclid(16) == 15 && self.probe([x, y + 1, z]));
        if edge { Cell::Air } else { Cell::Unloaded }
    }

    fn solid(s: u32) -> bool {
        // Only full opaque cubes conduct: glass, slabs and plants don't.
        let both = block_flags::SOLID | block_flags::OPAQUE;
        state_flags(s) & both == both
    }

    /// Another mod's emitter or receiver.
    fn extension(&self, p: Pos, s: u32) -> Option<Cell> {
        let emits = EMITS.on_block(s);
        let connects = CONNECTS.on_block(s);
        let receives = RECEIVER.on_block(s);
        if emits.is_none() && connects.is_none() && !receives {
            return None;
        }
        let base = emits.map_or([SidePowerV { strength: 0, strong: 0, direct: 0 }; 6], |e| e.sides);
        let output = std::array::from_fn(|face| {
            let b = base[face];
            let v = b.strength as f64 + if b.strong != 0 { 256.0 } else { 0.0 };
            let v = self.w.compute(&OUTPUT, v, &weft_sdk::abi::Query { pos: p, other: face as u64, state: s, ..query() });
            let v = if v.is_finite() { v.clamp(0.0, 511.0) as u32 } else { 0 };
            let strength = (v % 256) as u8;
            Power { strength: if strength == redstone_api::FULL { self.range.max() } else { self.range.clamp(strength) }, strong: v >= 256 }
        });
        Some(Cell::Device(Device { output, connects: connects.unwrap_or(0), receives, solid: Self::solid(s), kind: None }))
    }

    /// The state another mod's wire block takes for `strength` and `shape`: its `level` and side
    /// properties, where it has them.
    fn other_wire_state(&self, s: u32, strength: u8, shape: &Shape) -> u32 {
        static LEVELS: Mutex<Option<HashMap<u32, u8>>> = Mutex::new(None);
        let block = block_of(s);
        let levels = *LEVELS.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashMap::new).entry(block).or_insert_with(|| {
            (1..=64u8).take_while(|n| state_with(s, "level", &n.to_string()).is_some()).count() as u8 + 1
        });
        let mut out = s;
        if levels >= 2 {
            out = state_with(out, "level", &self.range.level(strength, levels).to_string()).unwrap_or(out);
        }
        for (i, side) in ["west", "east", "north", "south"].iter().enumerate() {
            let v = if shape.sides[i] == redstone_core::Link::Up { "up" } else if shape.points[i] { "side" } else { "none" };
            out = state_with(out, side, v).unwrap_or(out);
        }
        out
    }

    fn write(&mut self, p: Pos, state: u32) {
        if self.w.get(p) != state && self.w.set(p, state) {
            self.own.push((p, state));
        }
    }
}

impl View for WeftWorld<'_> {
    fn cell(&self, p: Pos) -> Cell {
        let s = self.w.get(p);
        if s == 0 {
            return self.air_or_unloaded(p);
        }
        if let Some(k) = blocks::kind_of(s) {
            return Cell::Device(redstone_core::device(k, self.range));
        }
        if let Some(t) = WIRE.on_block(s) {
            return Cell::Wire(Wire { kind: t.kind, links: t.links, powers_blocks: t.powers_blocks != 0, strength: 0 });
        }
        if let Some(c) = self.extension(p, s) {
            return c;
        }
        Cell::block(Self::solid(s))
    }
}

impl redstone_core::World for WeftWorld<'_> {
    fn range(&self) -> Range {
        self.range
    }

    fn cube_loaded(&self, cube: [i32; 3]) -> bool {
        let p = cube.map(|c| c * 16);
        // The cube just past a loaded column's top or bottom counts as loaded (module docs).
        self.probe(p) || self.probe([p[0], p[1] - 16, p[2]]) || self.probe([p[0], p[1] + 16, p[2]])
    }

    fn container(&self, p: Pos) -> Option<u8> {
        let Some(Kind::Comparator { facing, .. }) = blocks::kind_of(self.w.get(p)) else { return None };
        // The block behind, or the one behind that through a solid block without an entity.
        let mut at = step(p, facing.opposite());
        let mut be = self.w.block_entity(at);
        if be == 0 && Self::solid(self.w.get(at)) {
            at = step(at, facing.opposite());
            be = self.w.block_entity(at);
        }
        let inv = (be != 0).then(|| self.w.inventory(be)).filter(|i| i.size() > 0);
        let base = inv.as_ref().map_or(0, |inv| {
            let full: f64 = (0..inv.size()).map(|i| inv.get(i)).filter(|s| s.count > 0).map(|s| s.count as f64 / max_stack(s.item).max(1) as f64).sum();
            container_level(full / inv.size() as f64, self.range)
        });
        let v = self.w.compute(&COMPARATOR_READING, base as f64, &weft_sdk::abi::Query { pos: at, state: self.w.get(at), ..query() });
        let v = if v.is_finite() { v.clamp(0.0, self.range.max() as f64) as u8 } else { 0 };
        // A container, or a block some mod gives a reading.
        (inv.is_some() || v > 0).then_some(v)
    }

    fn instance(&self, p: Pos) -> u64 {
        // Block entity ids are unique across dimensions (G6).
        self.w.block_entity(p)
    }

    fn write_wire(&mut self, p: Pos, strength: u8, shape: Shape) {
        let s = self.w.get(p);
        let new = if blocks::DUST.is(s) { blocks::dust_state(self.range.level(strength, blocks::DUST_LEVELS), &shape) } else { self.other_wire_state(s, strength, &shape) };
        self.write(p, new);
    }

    fn write_device(&mut self, p: Pos, k: Kind) {
        if let Some(s) = blocks::state_of(k) {
            self.write(p, s);
        }
    }

    fn notify(&mut self, p: Pos, inputs: [SidePower; 6]) {
        let sides = inputs.map(|i| SidePowerV { strength: i.strength, strong: i.strong as u8, direct: i.direct as u8 });
        self.w.emit(&INPUT_CHANGED, Inputs { pos: p, sides });
    }

    fn save_pending(&mut self, p: Pos, pending: Option<(Pending, bool)>) {
        persist::save(self.w, p, pending);
    }
}

/// The strength of the wire at `p` right now (the `STRENGTH` chain).
pub fn strength_at(w: &World, p: Pos) -> u8 {
    let v = WeftWorld::new(w);
    if !matches!(v.cell(p), Cell::Wire(_)) {
        return 0;
    }
    let net = network(&v, p);
    if net.incomplete {
        return 0;
    }
    solve(&v, &net, v.range).get(&p).copied().unwrap_or(0)
}

/// Whether redstone has anything at `p` or next to it.
pub fn redstone_near(v: &WeftWorld, p: Pos) -> bool {
    std::iter::once(p).chain(redstone_core::Dir::ALL.map(|d| redstone_core::step(p, d))).any(|q| matches!(v.cell(q), Cell::Wire(_) | Cell::Device(_)))
}
