//! Typed versions of what the Weft SDK passes around bare, shared by the mods in this
//! repository: block positions that can be saved (`Pos`), boxes of them (`Area`), block states
//! (`BlockState`), `World`'s block calls taking and giving them (`TypedWorld`) and generation's
//! reads giving them (`TypedGeneration`), and a player's key that can be saved (`PlayerKey`);
//! with the feature `config`, a mod's settings file (`ConfigFile`); with the feature `store`,
//! its persistent data (`Table`). A proving ground for what could become part of the SDK.
//!
//! Everything here builds on one Weft release, the one every mod using it names: a mod moving
//! to another release moves weftkit and every other mod with it.

mod area;
mod block_state;
#[cfg(feature = "config")]
mod config;
#[cfg(any(feature = "config", feature = "store"))]
mod folder;
mod player_key;
mod pos;
#[cfg(feature = "store")]
mod store;
mod typed_generation;
mod typed_world;

pub use area::Area;
pub use block_state::BlockState;
#[cfg(feature = "config")]
pub use config::ConfigFile;
pub use player_key::PlayerKey;
pub use pos::Pos;
#[cfg(feature = "store")]
pub use store::Table;
pub use typed_generation::TypedGeneration;
pub use typed_world::TypedWorld;
