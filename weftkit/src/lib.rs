//! Typed versions of what the Weft SDK passes around bare, shared by the mods in this
//! repository: block positions that can be saved (`Pos`), boxes of them (`Area`), block states
//! (`BlockState`) and `World`'s block calls taking and giving them (`TypedWorld`). A proving
//! ground for what could become part of the SDK.
//!
//! Everything here builds on one Weft release, the one every mod using it names: a mod moving
//! to another release moves weftkit and every other mod with it.

mod area;
mod block_state;
mod pos;
mod typed_world;

pub use area::Area;
pub use block_state::BlockState;
pub use pos::Pos;
pub use typed_world::TypedWorld;
