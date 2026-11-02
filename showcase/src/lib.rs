//! Showcase: plays the films mods make with `showcase_api`, flying the camera through their
//! shots, and records them when the game is started with `--showcase <mod:film>`.
//!
//! `/showcase play <mod:film>` plays one in a running game. The server half prepares the
//! player and asks the film's mod to build its set (`server`); the client half receives the
//! shots and flies the camera (`director`, `playback`, `rig`).

mod director;
mod launch;
mod playback;
mod rig;
mod server;

use weft_sdk::*;

declare_mod!(deps: [], init: init);

fn init(r: &mut Registry) {
    server::register(r);
    director::register(r);
    launch::register(r);
}
