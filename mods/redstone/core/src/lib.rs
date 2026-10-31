//! The redstone rules, without Weft: the world is read through `View` and every rule is tested
//! against an in-memory grid. The `redstone` mod maps Weft's blocks onto this.

mod dir;
mod power;

pub use dir::*;
pub use power::*;
