//! Serverscript: a server runs scripts from a folder, written in Rhai, that react to what
//! happens in the game, answer its questions and add commands. They change nothing a client has
//! to know of, so players join with the mod alone, without the scripts.
//!
//! `--scripts <folder>` on the command line names the folder; without it the mod runs none.
//! `script` loads and calls them, `events` and `commands` hand them the game, `bindings` is what
//! they call back, and `store` is where they keep things, under data keys named as `name`
//! says, a player's under the number `player` gives them.

mod bindings;
mod commands;
mod events;
mod name;
mod player;
mod script;
mod store;
mod usage;
mod value;

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use weft_sdk::{Registry, declare_mod, info, launch_option, warn};

use crate::script::Scripts;
use crate::store::STORE;

/// The launch option that names the scripts folder.
const SCRIPTS_OPTION: &str = "scripts";

/// The store's file, in the scripts folder.
const DATA_FILE: &str = "data.bin";

declare_mod!(deps: [], init: init);

fn init(r: &mut Registry) {
    player::register(r);
    commands::register_setting(r);
    let Some(folder) = launch_option(SCRIPTS_OPTION).map(PathBuf::from) else {
        return;
    };
    let (scripts, mut problems) = Scripts::load(&folder, bindings::register);
    problems.extend(STORE.open(&folder.join(DATA_FILE)));
    for problem in problems {
        warn!("{problem}");
    }
    let (files, commands) = (scripts.len(), scripts.commands().len());
    info!(
        "{files} scripts with {commands} commands from {}",
        folder.display()
    );
    events::register(r);
    commands::register(r, &scripts, folder);
    script::replace(scripts);
}

/// `mutex` locked, even if a panic left it poisoned: the state stays usable.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| error.into_inner())
}

/// Whether every language file of the mod has `key`.
#[cfg(test)]
fn translated(key: &str) -> bool {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/lang");
    let files = std::fs::read_dir(&folder).expect("serverscript has no assets/lang folder");
    files.flatten().all(|file| {
        let text = std::fs::read_to_string(file.path()).unwrap_or_default();
        text.lines()
            .any(|line| line.split(" = ").next() == Some(key))
    })
}
