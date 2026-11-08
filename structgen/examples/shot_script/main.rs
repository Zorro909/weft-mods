//! Writes the game script `tools/structure-shots.sh` runs: each structure file given is placed
//! in the flatlands and photographed from around it, above it and inside it.
//!
//! `cargo run --example shot_script -- [--styles "brick_roof;birch_roof,spruce_frame"]
//! [--turn right] <file.wst>...`
//!
//! Each `;`-split entry of `--styles` is one look at every structure (`/structure load`'s
//! styles); without, each is shot once with its defaults. `--turn` places them turned, so
//! another side faces the front views. A shot is named `<structure>[-<look>]-<view>.png`.
//!
//! `--layout <file>` places several structures side by side instead and shoots them together
//! (see `layout.rs`), as `<layout>[-<look>]-<view>.png`.

mod layout;

use std::path::Path;

use structgen_api::Structure;

/// Where the player stands for each placing: `/structure load` puts the structure two blocks
/// north of them, its floor in place of the grass (y 3) they stand on.
const FEET: [i32; 3] = [0, 4, 0];
/// How far above the feet `via` puts the eye. A little over the player's 1.62, so the feet
/// don't round down into the block below and the structure land a block low.
const EYE: f32 = 1.7;

fn main() {
    let mut looks = vec![String::new()];
    let mut turn = "none".to_string();
    let mut files = Vec::new();
    let mut layout = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--styles" => {
                let list = args.next().expect("--styles needs a list of looks");
                looks = list.split(';').map(str::to_string).collect();
            }
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
        for look in &looks {
            println!("{}", layout::shots(&placed, look, &label(&name, look)));
        }
    }
    for file in &files {
        let (name, [sx, sy, sz]) = read(file);
        let size = match turn.as_str() {
            "right" | "left" => [sz, sy, sx],
            _ => [sx, sy, sz],
        };
        for look in &looks {
            println!("{}", shots(&name, size, &turn, look));
        }
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

/// The structure's name (its file's stem) and size.
fn read(file: &str) -> (String, [i32; 3]) {
    let bytes = std::fs::read(file).unwrap_or_else(|e| panic!("could not read {file}: {e}"));
    let structure =
        Structure::decode(&bytes).unwrap_or_else(|| panic!("{file} is no structure file"));
    let name = Path::new(file)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned());
    (name.unwrap_or_default(), structure.size())
}

/// Places one structure in one look, takes every view of it, and takes it away again.
fn shots(name: &str, size: [i32; 3], turn: &str, look: &str) -> String {
    let [x, y, z] = FEET;
    let mut lines = vec![
        format!("via {x}.5 {} {z}.5 0 0", y as f32 + EYE),
        "travel 0.2".to_string(),
        "look 0 0".to_string(),
        format!("chat /structure load {name} {turn} {look}")
            .trim_end()
            .to_string(),
        "wait 1".to_string(),
        "idle".to_string(),
    ];
    let label = label(name, look);
    for (view, eye, target) in views(size) {
        lines.extend(view_line(&label, view, eye, target));
    }
    lines.push("chat /structure undo".to_string());
    lines.push("wait 0.5".to_string());
    lines.join("\n")
}

/// What shots of `name` in `look` are called.
fn label(name: &str, look: &str) -> String {
    match look {
        "" => name.to_string(),
        _ => format!("{name}-{}", look.replace(',', "+")),
    }
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
    let lowest = [FEET[0] - sx / 2, FEET[1] - 1, FEET[2] - 1 - sz];
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
