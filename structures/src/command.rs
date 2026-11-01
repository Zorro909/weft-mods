//! `/structure`: `save` writes a box of the world to a file, `locate` finds the nearest
//! structure. One command, since the engine takes only the first of a name.

use weft_sdk::*;

use crate::{locate, save};

/// After the structures are registered, so `locate` completes their names.
pub fn register(r: &mut Registry) {
    let names = locate::short_names().join(",");
    let usage = format!(
        "save <name:word> <from:pos> <to:pos> | locate <structure:word({names})> [radius:int]"
    );
    r.command("structure", &usage, |cmd| {
        if cmd.has("locate") {
            locate::run(cmd);
        } else {
            save::run(cmd);
        }
    });
}
