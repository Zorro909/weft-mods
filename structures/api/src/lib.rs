//! Structures for any mod: the `.wst` file format, and placing structures while the world
//! generates. A mod links this crate and calls it from its own `init`; it needs neither the
//! `structures` mod loaded nor an entry in `declare_mod!`.
//!
//! - `place`: a static structure, one template stood on the ground now and then, turned
//!   whichever way the seed says, where the ground meets its conditions.
//! - `plan`: a dynamic structure. The world is split into square regions, and the mod's planner
//!   lays out each region from the seed and the terrain's columns: any number of templates
//!   (`template` registers them) and blocks, anywhere inside the region.
//! - `locate`: the nearest structure of a name, generated yet or not.
//!
//! Everything is worked out from the seed and the terrain's columns, never from blocks, so
//! every cube and every client gets the same answer. Structures don't know about each other,
//! so two can overlap.

mod fit;
mod format;
mod generator;
mod locate;
mod packing;
mod part;
mod placement;
mod plan;
mod planned;
mod planning;
mod random;
mod region;
mod stamp;
mod templates;
mod turn;

pub use fit::{Fit, Ground};
pub use format::Structure;
pub use generator::Planner;
pub use locate::{locate, names};
pub use part::{Foundation, Placed};
pub use placement::{Placement, place};
pub use plan::{Plan, PlanError};
pub use planning::{Planning, plan};
pub use random::Random;
pub use region::Region;
pub use templates::template;
pub use turn::Turn;
