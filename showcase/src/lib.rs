//! Showcase: plays the films mods make with `showcase_api`, flying the camera through their
//! shots, and records them when the game is started with `--showcase <mod:film>`.
//!
//! `/showcase play <mod:film>` plays one in a running game, `/showcase gallery <mod>` shows
//! every block of a mod, and `/camera` makes camera paths by hand. The server half prepares the
//! player and builds or asks for the set (`server`, `gallery`); the client half receives the
//! shots and flies the camera (`director`, `playback`, `rig`, `view`).

mod camera;
mod director;
mod gallery;
mod launch;
mod menus;
mod playback;
mod rig;
mod server;
mod view;

use std::sync::{Mutex, MutexGuard};

use weft_sdk::*;

declare_mod!(deps: [], init: init);

fn init(r: &mut Registry) {
    server::register(r);
    gallery::register(r);
    view::register(r);
    director::register(r);
    camera::register(r);
    launch::register(r);
}

/// `mutex` locked, even if a panic left it poisoned: the state stays usable.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}
