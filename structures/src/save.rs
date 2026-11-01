//! `/structure save <name> <from> <to>`: writes a box of the world to `structures/<name>.wst`
//! in the folder the game was started from, ready to copy into `assets/structures/`.

use std::path::PathBuf;

use structures_api::Structure;
use weft_sdk::*;

/// Longest side of a box, so one command can't stall the server reading millions of blocks.
const MAX_SIDE: i32 = 128;
/// Relative to the working directory: the SDK doesn't tell a mod where the game keeps its files.
const FOLDER: &str = "structures";

pub fn register(r: &mut Registry) {
    r.command("structure", "save <name:word> <from:pos> <to:pos>", save);
}

fn save(cmd: &mut ServerCommand) {
    let (Some(name), Some(from), Some(to)) = (
        cmd.text("name").map(str::to_string),
        cmd.pos("from"),
        cmd.pos("to"),
    ) else {
        return;
    };
    if !valid_name(&name) {
        return cmd.reply("structures.bad_name", &[Arg::Text(&name)]);
    }
    let (lo, size) = block_box(from, to);
    if size.iter().any(|&side| side > MAX_SIDE) {
        return cmd.reply("structures.too_big", &[Arg::Int(MAX_SIDE as i64)]);
    }
    let Some(structure) = read_box(&cmd.world(), lo, size) else {
        return cmd.reply("structures.not_loaded", &[]);
    };
    match write_file(&name, &structure) {
        Ok(path) => {
            let [x, y, z] = size.map(|side| Arg::Int(side as i64));
            cmd.reply(
                "structures.saved",
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
            "structures.write_failed",
            &[Arg::Text(&name), Arg::Text(&e.to_string())],
        ),
    }
}

/// Becomes the file's name and the template's.
fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// The lowest block of the box between two corners, and its size in blocks.
fn block_box(from: [f64; 3], to: [f64; 3]) -> ([i32; 3], [i32; 3]) {
    let (from, to) = (from.map(|v| v.floor() as i32), to.map(|v| v.floor() as i32));
    let lo = [0, 1, 2].map(|i| from[i].min(to[i]));
    let size = [0, 1, 2].map(|i| (from[i] - to[i]).abs() + 1);
    (lo, size)
}

/// `None` if any of it isn't loaded.
fn read_box(world: &World, lo: [i32; 3], size: [i32; 3]) -> Option<Structure> {
    let mut names = Vec::with_capacity(size.iter().product::<i32>() as usize);
    for y in 0..size[1] {
        for z in 0..size[2] {
            for x in 0..size[0] {
                let state = world.get_loaded([lo[0] + x, lo[1] + y, lo[2] + z])?;
                names.push(block_name(state));
            }
        }
    }
    Some(Structure::from_names(size.map(|side| side as u16), names))
}

fn write_file(name: &str, structure: &Structure) -> std::io::Result<PathBuf> {
    let dir = std::env::current_dir()?.join(FOLDER);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{name}.wst"));
    std::fs::write(&path, structure.encode())?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_covers_both_corners_in_any_order() {
        let (lo, size) = block_box([3.7, 64.0, -1.2], [1.1, 62.5, -4.0]);
        assert_eq!(lo, [1, 62, -4], "the lowest block of each axis");
        assert_eq!(
            size,
            [3, 3, 3],
            "both corner blocks inside, -1.2 being block -2"
        );
    }

    #[test]
    fn names_stay_inside_the_folder() {
        assert!(valid_name("small_hut2"), "letters, digits and underscores");
        assert!(!valid_name("../hut"), "no paths");
        assert!(!valid_name("Hut"), "no capitals");
        assert!(!valid_name(""), "not empty");
    }
}
