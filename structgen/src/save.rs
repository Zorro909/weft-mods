//! `/structure save <name> [from] [to]`: writes a box of the world to `structures/<name>.wst`
//! in the folder the game was started from, ready to copy into `assets/structures/`. Without
//! corners, the player's selection (`/structure pos1`, `pos2`).

use std::path::{Path, PathBuf};

use structgen_api::{Area, Pos, Structure};
use weft_sdk::*;

use crate::selection;

/// Longest side of a box, so one command can't stall the server reading millions of blocks.
pub const MAX_SIDE: i32 = 128;
/// Relative to the working directory: the SDK doesn't tell a mod where the game keeps its files.
const FOLDER: &str = "structures";

pub fn run(cmd: &mut ServerCommand) {
    let Some(name) = cmd.text("name").map(str::to_string) else {
        return;
    };
    if !valid_name(&name) {
        return cmd.reply("structgen.bad_name", &[Arg::Text(&name)]);
    }
    let Some(area) = chosen_area(cmd) else {
        return cmd.reply("structgen.no_selection", &[]);
    };
    let size = area.size();
    if size.iter().any(|&side| side > MAX_SIDE) {
        return cmd.reply("structgen.too_big", &[Arg::Int(MAX_SIDE as i64)]);
    }
    let Some(names) = read_box(&cmd.world(), &area) else {
        return cmd.reply("structgen.not_loaded", &[]);
    };
    let Some(structure) = Structure::from_names(size.map(|side| side as u16), names) else {
        return cmd.reply("structgen.too_many_states", &[]);
    };
    match write_file(&name, &structure) {
        Ok(path) => {
            let [x, y, z] = size.map(|side| Arg::Int(side as i64));
            cmd.reply(
                "structgen.saved",
                &[
                    Arg::Text(&name),
                    x,
                    y,
                    z,
                    Arg::Text(&path.to_string_lossy()),
                ],
            );
        }
        Err(e) => cmd.reply(
            "structgen.write_failed",
            &[Arg::Text(&name), Arg::Text(&e.to_string())],
        ),
    }
}

/// The box between the two corner blocks given, else the player's selection.
fn chosen_area(cmd: &ServerCommand) -> Option<Area> {
    if let (Some(from), Some(to)) = (cmd.pos("from"), cmd.pos("to")) {
        return Some(Area::between(from.block().into(), to.block().into()));
    }
    selection::selected(cmd.world().dimension(), cmd.player())
}

/// Where structure `name` is saved, relative to the folder the game was started from.
pub fn path_of(name: &str) -> PathBuf {
    Path::new(FOLDER).join(format!("{name}.wst"))
}

/// Becomes the file's name and the template's.
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Each block's state name, in `Structure::blocks` order; `None` if any of it isn't loaded.
fn read_box(world: &World, area: &Area) -> Option<Vec<&'static str>> {
    let size = area.size();
    let mut names = Vec::with_capacity(size.iter().product::<i32>() as usize);
    for y in 0..size[1] {
        for z in 0..size[2] {
            for x in 0..size[0] {
                let state = world.block_loaded(area.lo + Pos::new(x, y, z))?;
                names.push(block_name(state));
            }
        }
    }
    Some(names)
}

fn write_file(name: &str, structure: &Structure) -> std::io::Result<PathBuf> {
    let path = std::env::current_dir()?.join(path_of(name));
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, structure.encode())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_covers_both_corners_in_any_order() {
        let area = Area::between(Pos::new(3, 64, -2), Pos::new(1, 62, -4));
        assert_eq!(
            area.lo,
            Pos::new(1, 62, -4),
            "the lowest block of each axis"
        );
        assert_eq!(area.size(), [3, 3, 3], "both corner blocks inside");
    }

    #[test]
    fn names_stay_inside_the_folder() {
        assert!(valid_name("small_hut2"), "letters, digits and underscores");
        assert!(!valid_name("../hut"), "no paths");
        assert!(!valid_name("Hut"), "no capitals");
        assert!(!valid_name(""), "not empty");
    }
}
