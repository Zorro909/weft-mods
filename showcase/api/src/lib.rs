//! The interface between a mod's films and the `showcase` mod that plays them.
//!
//! A film is a set the mod builds in the world and a list of camera shots around it. The mod
//! registers it with `film`, usually behind a cargo feature, and lists `showcase` in
//! `declare_mod!` while that feature is on. The showcase mod does the rest: it picks the place,
//! asks the mod to build there (`PLAY`), receives the shots (`TIMELINE`) and flies the camera
//! through them on the player's client.
//!
//! Everything a film says is plain data, so the two mods only meet through the events and
//! channels below, whose layouts the engine checks when the mods load.

mod area;
mod film;
mod name;
mod set;
mod waiting;
mod wire;

pub use area::Area;
pub use film::{Ease, Film, Look, Path, Shot, Step, crane, dolly, flyby, follow, hold, orbit};
pub use name::Name;
pub use set::{FilmOptions, Set, film, send, signal, subject};
pub use waiting::Waiting;
use weft_sdk::*;
pub use weftkit::{BlockState, Pos};
pub use wire::{TIMELINE_VERSION, Timeline};

value! {
    /// Asks the mod that owns `film` (`mymod:tour`) to build its set at `origin` for `player`
    /// and send the shots.
    pub struct Play { pub film: Name, pub player: u64, pub origin: Pos }
}
pub static PLAY: Event<Play> = Event::named("showcase:play");

value! {
    /// Fired on the server when a film reaches `Film::cue(name)`, for the mod that owns `film`.
    pub struct Cue { pub film: Name, pub player: u64, pub origin: Pos, pub name: Name }
}
pub static CUE: Event<Cue> = Event::named("showcase:cue");

/// A film, from the mod that built it to the player's client: a `Timeline` as `Timeline::encode`
/// writes it and `Timeline::decode` reads it back.
pub static TIMELINE: Channel<Vec<u8>> = Channel::named("showcase:timeline");

value! {
    /// Lets a waiting film go on (`Film::wait_for`).
    pub struct Signal { pub name: Name }
}
pub static SIGNAL: Channel<Signal> = Channel::named("showcase:signal");

value! {
    /// What the shots naming `name` follow or look at: `entity`, or the world position `pos`
    /// when `entity` is 0.
    pub struct Subject { pub name: Name, pub entity: u64, pub pos: [f64; 3] }
}
pub static SUBJECT: Channel<Subject> = Channel::named("showcase:subject");
