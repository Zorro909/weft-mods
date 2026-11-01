//! Structures for any mod: the `.wst` file format, and `place`, which stands a structure on the
//! ground while the world generates.
//!
//! A mod links this crate and calls `place` from its own `init`; it needs neither the
//! `structures` mod loaded nor an entry in `declare_mod!`. The placing runs as that mod's own
//! decoration (`Registry::decorate`), from the seed and the terrain's columns alone:
//! - one chance per grid cell, at a spot in the cell from the seed
//! - only where the biome under its middle has a tag, and the ground under all of it varies by
//!   at most `max_slope` blocks and has no cave mouth
//! - the bottom layer on the median ground height; a foundation block fills down to the ground
//!   under it, and ground rising above it is cut away
//!
//! Structures don't know about each other yet, so two can overlap.

mod format;
mod packing;
mod placement;
mod site;
mod stamp;

pub use format::Structure;
pub use placement::{Placement, place};
