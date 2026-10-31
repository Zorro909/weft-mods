//! What other mods need to work with redstone: names and value layouts only, never the mod's
//! code (the statics compile into whoever uses them, so they carry full names).
//!
//! - An emitter sets `EMITS` (and `CONNECTS` for the sides wire should run to) on its states, or
//!   answers the `OUTPUT` chain when its output isn't fixed per state.
//! - A receiver is tagged `RECEIVER` and listens to `INPUT_CHANGED`.
//! - A wire type sets `WIRE` and may have a `level` property (any number of values, at least 2)
//!   that redstone keeps at the wire's brightness level. Strength itself is never stored; ask
//!   `strength(..)`.
//! - Anything that changes one of these blocks by code calls `set_and_announce` (or emits
//!   `CHANGED`), so redstone notices.

use weft_sdk::*;

pub use redstone_core::{FULL, Range};

/// The world's signal range: 16, 32 or 64 levels. Change with `/rule redstone:signal_range <0|1|2>`.
pub static RANGE: Config<u32> = Config::named("redstone:signal_range");

/// The range of the world that is running now.
pub fn range() -> Range {
    Range::from_index(RANGE.get())
}

value! {
    /// Power on one side. `strength` 255 (`FULL`) is the world's maximum, whatever the range;
    /// `strong` and `direct` are 0 or 1.
    pub struct SidePowerV { pub strength: u8, pub strong: u8, pub direct: u8 }
}

value! {
    /// What a block emits to each side, by face index (-X +X -Y +Y -Z +Z).
    pub struct Emits { pub sides: [SidePowerV; 6] }
}
pub static EMITS: Component<Emits> = Component::named("redstone:emits");

/// The sides wire connects to, a bit per face index. Change it with a state change (and
/// `set_and_announce`), and wire around reshapes.
pub static CONNECTS: Component<u8> = Component::named("redstone:connects");

/// Blocks that want `INPUT_CHANGED`.
pub static RECEIVER: Tag = Tag::named("redstone:receiver");

value! {
    /// A wire block. `kind` 0 is dust; two wires connect when each one's `links` has the other's
    /// kind bit (kinds from 32 up connect only to their own kind). `powers_blocks` 1: weak-powers
    /// the block below and the blocks its shape points into, like dust.
    pub struct WireType { pub kind: u32, pub links: u32, pub powers_blocks: u8 }
}
pub static WIRE: Component<WireType> = Component::named("redstone:wire");

value! {
    pub struct Changed { pub pos: [i32; 3] }
}
/// A redstone-relevant block changed by code. Redstone notices placing and breaking by players
/// and its own blocks' neighbors by itself.
pub static CHANGED: Event<Changed> = Event::named("redstone:changed");

value! {
    /// What reaches each side of a receiver now, by face index.
    pub struct Inputs { pub pos: [i32; 3], pub sides: [SidePowerV; 6] }
}
/// What reaches a `RECEIVER` block changed. Its reaction should take effect one tick later or
/// more (spec: every device reacts at least one tick after its input).
pub static INPUT_CHANGED: Event<Inputs> = Event::named("redstone:input_changed");

/// An emitter's output on one side, for outputs that aren't fixed per state: the query has `pos`
/// and `other` = the face index; the value is `strength + 256 * strong` (strength 255 = full).
/// It starts from the block's `EMITS`.
pub static OUTPUT: Chain = Chain::named("redstone:output");

/// The strength of the wire at the query's `pos`, solved when asked: 0 for no wire, an unpowered
/// one or one whose circuit is frozen. Use `strength(..)`.
pub static STRENGTH: Chain = Chain::named("redstone:strength");

/// A comparator's reading of the block at the query's `pos` (the one behind it, or behind the solid
/// block behind it): the value starts at the fill level of the block entity's inventory, 0 to the
/// range's maximum (`range()`). Machines answer with their progress, tanks with their level.
/// Comparators re-read every tick, as Weft doesn't say when an inventory changes (G17).
pub static COMPARATOR_READING: Chain = Chain::named("redstone:comparator_reading");

/// Presses the button at `pos`, like a player using it (`press`).
pub static PRESS: Event<Changed> = Event::named("redstone:press");

/// Pistons never move these.
pub static UNPUSHABLE: Tag = Tag::named("redstone:unpushable");

/// Sets a block and tells redstone about it. False if the position isn't loaded.
pub fn set_and_announce(w: &World, pos: [i32; 3], state: u32) -> bool {
    let ok = w.set(pos, state);
    if ok {
        w.emit(&CHANGED, Changed { pos });
    }
    ok
}

/// The strength of the wire at `pos` (see `STRENGTH`).
pub fn strength(w: &World, pos: [i32; 3]) -> u8 {
    w.compute(&STRENGTH, 0.0, &abi::Query { pos, ..query() }) as u8
}

/// Presses the button at `pos` at the end of the tick, like a player using it: on for its length
/// (stone 20 ticks, wood 30). A button already on stays as it is.
pub fn press(w: &World, pos: [i32; 3]) {
    w.emit(&PRESS, Changed { pos });
}
