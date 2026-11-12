//! Writes the game script `tools/structure-shots.sh` runs: each structure file given is placed
//! in the flatlands and photographed from around it, above it and inside it.
//!
//! `cargo run --example shot_script -- [--turn right] <file.json>...`
//!
//! `--turn` places them turned, so another side faces the front views; a building of the mod
//! as the mod registers it turned, any other file as the engine turns it. Its slots show their
//! defaults: the engine's `/structure load` takes no palette. A shot is named
//! `<structure>-<view>.png`.
//!
//! `--layout <file>` places several structures side by side instead and shoots them together
//! (see `layout.rs`), as `<layout>-<view>.png`.

mod layout;
// The mod's own file reader and writer; each example uses part of it.
#[allow(dead_code)]
#[path = "../../src/structure_file.rs"]
mod structure_file;

use std::path::Path;

use crate::structure_file::Structure;

/// Where the camera stands for each placing: the structure goes two blocks north of it, its
/// floor in place of the grass (y 3) under it.
const FEET: [i32; 3] = [0, 4, 0];
/// The mod's own buildings, which it registers turned and paved too.
const BUILDINGS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/structures");
/// How far above the feet `via` puts the eye. A little over the player's 1.62, so the feet
/// don't round down into the block below and the structure land a block low.
const EYE: f32 = 1.7;

fn main() {
    let mut turn = "none".to_string();
    let mut files = Vec::new();
    let mut layout = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--turn" => turn = args.next().expect("--turn needs none, right, back or left"),
            "--layout" => layout = Some(args.next().expect("--layout needs a layout file")),
            _ => files.push(arg),
        }
    }
    assert!(
        !files.is_empty() || layout.is_some(),
        "name at least one structure file, or a layout"
    );
    println!("{}", setup());
    if let Some(file) = &layout {
        let placed = layout::read_layout(file);
        let name = Path::new(file)
            .file_stem()
            .map_or(String::new(), |s| s.to_string_lossy().into_owned());
        println!("{}", layout::shots(&placed, &name));
    }
    for file in &files {
        let read = read(file);
        let size = turned(read.size, &turn);
        println!("{}", shots(&read.name, &load(&read, &turn, false), size));
    }
    println!("quit");
}

/// Into the flatlands at noon, as a spectator so nothing falls or gets in the way.
fn setup() -> String {
    [
        "loaded",
        "chat /gamemode spectator",
        "chat /dim base:flatlands",
        "wait 2",
        "loaded",
        "chat /weather clear",
    ]
    .join("\n")
}

/// A structure file as the script needs it.
pub struct Read {
    /// Its file's stem.
    pub name: String,
    pub size: [i32; 3],
    /// Whether it is one of the mod's buildings.
    pub building: bool,
    /// Whether it has ground in its box, which a paved copy paves.
    pub ground: bool,
}

fn read(file: &str) -> Read {
    let text =
        std::fs::read_to_string(file).unwrap_or_else(|e| panic!("could not read {file}: {e}"));
    let structure = Structure::from_json(&text)
        .unwrap_or_else(|why| panic!("{file} is no structure file: {why}"));
    let path = Path::new(file);
    let name = path.file_stem().map(|s| s.to_string_lossy().into_owned());
    let own = Path::new(BUILDINGS).join(path.file_name().unwrap_or_default());
    let building =
        (own.canonicalize().ok()).is_some_and(|own| path.canonicalize().ok() == Some(own));
    Read {
        name: name.unwrap_or_default(),
        size: structure.size(),
        building,
        ground: structure.blocks().any(|(_, cell)| cell == "@ground"),
    }
}

/// A size of x, y and z once turned by `turn`.
fn turned([sx, sy, sz]: [i32; 3], turn: &str) -> [i32; 3] {
    match turn {
        "right" | "left" => [sz, sy, sx],
        _ => [sx, sy, sz],
    }
}

/// What `/structure load` takes to place `read` turned by `turn` at a corner, as
/// `<template> {}` with the corner to go in: one of the mod's buildings as it registers it
/// turned (`structgen:hof/right`), paved if `paved` and it has ground; any other file by its
/// name, turned by the engine.
pub fn load(read: &Read, turn: &str, paved: bool) -> String {
    if !read.building {
        let quarters = ["none", "right", "back", "left"]
            .iter()
            .position(|t| *t == turn);
        let quarters = quarters.unwrap_or_else(|| panic!("no turn {turn}"));
        return format!("{} {{}} {quarters}", read.name);
    }
    let way = if turn == "none" {
        String::new()
    } else {
        format!("/{turn}")
    };
    let paving = if paved && read.ground { "/paved" } else { "" };
    format!("structgen:{}{way}{paving} {{}}", read.name)
}

/// The lowest corner of a structure of `size` placed in front of `FEET`.
fn lowest([sx, _, sz]: [i32; 3]) -> [i32; 3] {
    [FEET[0] - sx / 2, FEET[1] - 1, FEET[2] - 1 - sz]
}

/// Places one structure (`load`'s words), takes every view of it, and takes it away again.
fn shots(name: &str, load: &str, size: [i32; 3]) -> String {
    let [x, y, z] = FEET;
    let mut lines = vec![
        format!("via {x}.5 {} {z}.5 0 0", y as f32 + EYE),
        "travel 0.2".to_string(),
        "look 0 0".to_string(),
        place(load, lowest(size)),
        "wait 1".to_string(),
        "idle".to_string(),
    ];
    for (view, eye, target) in views(size) {
        lines.extend(view_line(name, view, eye, target));
    }
    lines.push("chat /structure undo".to_string());
    lines.push("wait 0.5".to_string());
    lines.join("\n")
}

/// The script's line placing `load` (`load`'s words) with its lowest corner at `corner`.
pub fn place(load: &str, [x, y, z]: [i32; 3]) -> String {
    format!(
        "chat /structure load {}",
        load.replace("{}", &format!("{x} {y} {z}"))
    )
}

/// Goes to `eye`, looks at `target` and takes shot `<label>-<view>.png`.
fn view_line(label: &str, view: &str, eye: [f32; 3], target: [f32; 3]) -> Vec<String> {
    vec![
        format!(
            "via {:.1} {:.1} {:.1} @ {:.1} {:.1} {:.1}",
            eye[0], eye[1], eye[2], target[0], target[1], target[2]
        ),
        "travel 0.3".to_string(),
        "wait 0.8".to_string(),
        format!("shot {label}-{view}.png"),
        "wait 0.3".to_string(),
    ]
}

/// Each view's name, eye and the point it looks at, for a structure of `size` (turned) placed
/// from `FEET`. The front views look at its north side.
fn views([sx, sy, sz]: [i32; 3]) -> Vec<(&'static str, [f32; 3], [f32; 3])> {
    let lowest = lowest([sx, sy, sz]);
    let [sx, sy, sz] = [sx, sy, sz].map(|v| v as f32);
    let [x0, y0, z0] = lowest.map(|v| v as f32);
    let middle = [x0 + sx / 2.0, y0 + sy * 0.4, z0 + sz / 2.0];
    let far = sx.max(sz).max(sy) * 0.75 + 6.0;
    let at = |dx: f32, dy: f32, dz: f32| [middle[0] + dx, middle[1] + dy, middle[2] + dz];
    // Inside, from near one end wall toward the other, on the ground floor and, in a building
    // tall enough to have one, an upper floor.
    let inside = |y: f32, from: f32, to: f32| {
        (
            [middle[0], y, z0 + sz * from],
            [middle[0], y - 0.8, z0 + sz * to],
        )
    };
    let mut views = vec![
        ("front", at(-0.55 * far, 0.45 * far, -far), middle),
        (
            "front_low",
            at(0.0, 1.8 - sy * 0.4, -0.9 * far),
            at(0.0, 0.0, 0.0),
        ),
        ("back", at(0.6 * far, 0.4 * far, far), middle),
        ("side", at(-far, 0.25 * far, 0.15 * far), middle),
        ("above", at(0.3 * far, 1.3 * far, -0.4 * far), middle),
    ];
    for (name, (eye, target)) in [
        ("inside_north", inside(y0 + 2.4, 0.75, 0.0)),
        ("inside_south", inside(y0 + 2.4, 0.25, 1.0)),
    ] {
        views.push((name, eye, target));
    }
    if sy >= 12.0 {
        let (eye, target) = inside(y0 + 6.4, 0.75, 0.0);
        views.push(("inside_upper", eye, target));
    }
    views
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The game keeps the eye height `via` gives as an f32 and takes the player's 1.62 off it
    /// as an f64.
    #[test]
    fn the_feet_land_on_the_block_meant() {
        let eye = (FEET[1] as f32 + EYE) as f64;
        assert_eq!((eye - 1.62).floor() as i32, FEET[1], "not the block below");
    }
}
