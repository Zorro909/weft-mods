//! `/structure`, each branch handled by its own function:
//! - `save` writes a box of the world to a file, `load` places one in front of the player and
//!   `undo` takes the last load back
//! - `pos1` and `pos2` select a box for `save`, as the wand does
//! - `locate` finds the nearest structure

use weft_sdk::*;

use crate::{load, locate, save, selection, worldgen};

const NAME: &str = "structure";

/// After the structures are registered, so `locate` completes their names.
pub fn register(r: &mut Registry) {
    let structures = locate::short_names().join(",");
    let files = worldgen::embedded_names().join(",");
    r.command(NAME, "save <name:word> [from:pos] [to:pos]", save::run);
    let load_usage = format!("load <file:word({files})> [turn:word(none,right,back,left)]");
    r.command_branch(NAME, &load_usage, load::run);
    r.command_branch(NAME, "undo", load::undo);
    r.command_branch(NAME, "pos1 [at:pos]", selection::first);
    r.command_branch(NAME, "pos2 [at:pos]", selection::second);
    let locate_usage = format!("locate <structure:word({structures})> [radius:int]");
    r.command_branch(NAME, &locate_usage, locate::run);
}
