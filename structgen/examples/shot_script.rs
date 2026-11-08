//! Writes the game script `tools/structure-shots.sh` runs: each structure file given is placed
//! in the flatlands and photographed from around it, above it and inside it.
//!
//! `cargo run --example shot_script -- [--styles "brick_roof;birch_roof,spruce_frame"] <file.wst>...`
//!
//! Each `;`-split entry of `--styles` is one look at every structure (`/structure load`'s
//! styles); without, each is shot once with its defaults. A shot is named
//! `<structure>[-<look>]-<view>.png`.

use std::path::Path;

use structgen_api::Structure;

/// Where the player stands for each placing: `/structure load` puts the structure two blocks
/// north of them, its floor in place of the grass (y 3) they stand on.
const FEET: [i32; 3] = [0, 4, 0];

fn main() {
    let mut looks = vec![String::new()];
    let mut files = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--styles" => {
                let list = args.next().expect("--styles needs a list of looks");
                looks = list.split(';').map(str::to_string).collect();
            }
            _ => files.push(arg),
        }
    }
    assert!(!files.is_empty(), "name at least one structure file");
    println!("{}", setup());
    for file in &files {
        let (name, size) = read(file);
        for look in &looks {
            println!("{}", shots(&name, size, look));
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
fn shots(name: &str, size: [i32; 3], look: &str) -> String {
    let [x, y, z] = FEET;
    let mut lines = vec![
        format!("via {x}.5 {} {z}.5 0 0", y as f32 + 1.62),
        "travel 0.2".to_string(),
        "look 0 0".to_string(),
        format!("chat /structure load {name} none {look}")
            .trim_end()
            .to_string(),
        "wait 1".to_string(),
        "idle".to_string(),
    ];
    let label = match look {
        "" => name.to_string(),
        _ => format!("{name}-{}", look.replace(',', "+")),
    };
    for (view, eye, target) in views(size) {
        lines.push(format!(
            "via {:.1} {:.1} {:.1} @ {:.1} {:.1} {:.1}",
            eye[0], eye[1], eye[2], target[0], target[1], target[2]
        ));
        lines.push("travel 0.3".to_string());
        lines.push("wait 0.8".to_string());
        lines.push(format!("shot {label}-{view}.png"));
        lines.push("wait 0.3".to_string());
    }
    lines.push("chat /structure undo".to_string());
    lines.push("wait 0.5".to_string());
    lines.join("\n")
}

/// Each view's name, eye and the point it looks at, for a structure of `size` placed from
/// `FEET`. Its front is the north side, as saved.
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
