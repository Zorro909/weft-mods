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

mod film;
mod set;
mod wire;

pub use film::{Ease, Film, Look, Path, Shot, Step, crane, dolly, flyby, follow, hold, orbit};
pub use set::{Set, film, signal, subject};
use weft_sdk::*;
pub use wire::{Timeline, VERSION, decode};

/// A name in an event or packet: UTF-8, padded with zeros.
pub type Name = [u8; 64];

value! {
    /// Asks the mod that owns `film` (`mymod:tour`) to build its set at `origin` for `player`
    /// and send the shots.
    pub struct Play { pub film: Name, pub player: u64, pub origin: [i32; 3] }
}
pub static PLAY: Event<Play> = Event::named("showcase:play");

value! {
    /// Fired on the server when a film reaches `Film::cue(name)`, for the mod that owns `film`.
    pub struct Cue { pub film: Name, pub player: u64, pub origin: [i32; 3], pub name: Name }
}
pub static CUE: Event<Cue> = Event::named("showcase:cue");

/// A film's steps in the `wire` format, from the mod that built it to the player's client.
pub static TIMELINE: Channel<Vec<u8>> = Channel::named("showcase:timeline");

value! {
    /// Lets a waiting film go on (`Film::wait_for`).
    pub struct Signal { pub name: Name }
}
pub static SIGNAL: Channel<Signal> = Channel::named("showcase:signal");

value! {
    /// What a shot named `name` follows or looks at: `entity`, or `pos` where it is 0.
    pub struct Subject { pub name: Name, pub entity: u64, pub pos: [f64; 3] }
}
pub static SUBJECT: Channel<Subject> = Channel::named("showcase:subject");

/// `text` as a `Name`, cut short at 64 bytes.
pub fn name(text: &str) -> Name {
    let mut out = [0; 64];
    let mut len = text.len().min(64);
    while !text.is_char_boundary(len) {
        len -= 1;
    }
    out[..len].copy_from_slice(&text.as_bytes()[..len]);
    out
}

/// The text of a `Name`.
pub fn name_text(name: &Name) -> &str {
    let len = name.iter().position(|b| *b == 0).unwrap_or(name.len());
    std::str::from_utf8(&name[..len]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_survive_the_round_trip() {
        assert_eq!(name_text(&name("cogwork:tubes")), "cogwork:tubes");
    }

    #[test]
    fn long_names_are_cut_on_a_character() {
        let long = "é".repeat(40);
        assert_eq!(name_text(&name(&long)), "é".repeat(32));
    }
}
