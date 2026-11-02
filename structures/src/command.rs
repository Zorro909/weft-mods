//! `/structure`: one command, since the engine takes only the first of a name.
//! - `save` writes a box of the world to a file, `load` places one in front of the player and
//!   `undo` takes the last load back
//! - `pos1` and `pos2` select a box for `save`, as the wand does
//! - `locate` finds the nearest structure

use weft_sdk::*;

use crate::{load, locate, save, selection, worldgen};

type Branch = fn(&mut ServerCommand);

/// Each branch by its first word.
const BRANCHES: &[(&str, Branch)] = &[
    ("save", save::run),
    ("load", load::run),
    ("undo", load::undo),
    ("pos1", selection::first),
    ("pos2", selection::second),
    ("locate", locate::run),
];

/// After the structures are registered, so `locate` completes their names.
pub fn register(r: &mut Registry) {
    let structures = locate::short_names().join(",");
    let files = worldgen::embedded_names().join(",");
    let usage = [
        "save <name:word> [from:pos] [to:pos]".to_string(),
        format!("load <file:word({files})> [turn:word(none,right,back,left)]"),
        "undo".to_string(),
        "pos1 [at:pos]".to_string(),
        "pos2 [at:pos]".to_string(),
        format!("locate <structure:word({structures})> [radius:int]"),
    ]
    .join(" | ");
    r.command("structure", &usage, |cmd| {
        if let Some((_, run)) = BRANCHES.iter().find(|(word, _)| cmd.has(word)) {
            run(cmd);
        }
    });
}
