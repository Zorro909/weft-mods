//! Typed versions of what the Weft SDK passes around bare, shared by the mods in this
//! repository: block positions that can be saved (`Pos`) and block states (`BlockState`). A
//! proving ground for what could become part of the SDK.
//!
//! Everything here builds on one Weft release, the one every mod using it names: a mod moving
//! to another release moves weftkit and every other mod with it.

mod block_state;
mod pos;

pub use block_state::BlockState;
pub use pos::Pos;
