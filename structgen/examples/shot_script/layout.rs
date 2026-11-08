//! Layouts: several structures placed side by side and photographed together, to see tiles
//! such as paths and streets join up. A layout file has one structure a line:
//!
//! ```text
//! # name           turn   x   z
//! path_corner      right  0   0
//! path_straight    right  5   0
//! ```
//!
//! `x` and `z` are the north-west corner of the structure as placed (turned), on the
//! flatlands; `#` starts a comment.

use std::path::Path;

use crate::{EYE, read, view_line};

/// `/structure load` puts a structure this many blocks in front of the player.
const GAP: i32 = 2;
/// Where a player stands on the flatlands: on the grass, at y 3.
const FEET_Y: i32 = 4;

/// One structure of a layout.
pub struct Placed {
    name: String,
    turn: String,
    corner: [i32; 2],
    /// As turned.
    size: [i32; 3],
}

/// The structures of layout `file`, their sizes read from this mod's `assets/structures/`.
pub fn read_layout(file: &str) -> Vec<Placed> {
    let text =
        std::fs::read_to_string(file).unwrap_or_else(|e| panic!("could not read {file}: {e}"));
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/structures");
    let lines = text
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim());
    lines
        .filter(|line| !line.is_empty())
        .map(|line| {
            let words: Vec<&str> = line.split_whitespace().collect();
            let [name, turn, x, z] = words[..] else {
                panic!("{file}: {line:?} is not `name turn x z`");
            };
            let number = |v: &str| {
                v.parse()
                    .unwrap_or_else(|_| panic!("{file}: {v} in {line:?}"))
            };
            let (_, [sx, sy, sz]) = read(&assets.join(format!("{name}.wst")).to_string_lossy());
            Placed {
                name: name.to_string(),
                turn: turn.to_string(),
                corner: [number(x), number(z)],
                size: match turn {
                    "right" | "left" => [sz, sy, sx],
                    _ => [sx, sy, sz],
                },
            }
        })
        .collect()
}

/// Places every structure of the layout in one look, then photographs all of it: from above,
/// from each corner, and along it at eye height. Shots are `<label>-<view>.png`.
pub fn shots(layout: &[Placed], look: &str, label: &str) -> String {
    let mut lines = Vec::new();
    for placed in layout {
        lines.extend(place(placed, look));
    }
    lines.push("wait 1".to_string());
    lines.push("idle".to_string());
    for (view, eye, target) in views(layout) {
        lines.extend(view_line(label, view, eye, target));
    }
    lines.join("\n")
}

/// Stands so that `/structure load` puts `placed` at its corner, and loads it.
fn place(placed: &Placed, look: &str) -> Vec<String> {
    let [x, z] = placed.corner;
    let [sx, _, sz] = placed.size;
    let feet = [x + sx / 2, FEET_Y, z + GAP + sz - 1];
    let load = format!(
        "chat /structure load {} {} {look}",
        placed.name, placed.turn
    );
    vec![
        format!(
            "via {}.5 {} {}.5 0 0",
            feet[0],
            feet[1] as f32 + EYE,
            feet[2]
        ),
        "travel 0.1".to_string(),
        // `travel` only starts the move: a load before it ends lands somewhere on the way.
        "wait 0.4".to_string(),
        "look 0 0".to_string(),
        load.trim_end().to_string(),
        "wait 0.2".to_string(),
    ]
}

/// The views of a layout: above it, from its four corners, and along it at eye height.
fn views(layout: &[Placed]) -> Vec<(&'static str, [f32; 3], [f32; 3])> {
    let low = [0, 1].map(|i| layout.iter().map(|p| p.corner[i]).min().unwrap_or(0) as f32);
    let high = [0, 1].map(|i| {
        let ends = layout.iter().map(|p| p.corner[i] + p.size[i * 2]);
        ends.max().unwrap_or(0) as f32
    });
    let middle = [
        (low[0] + high[0]) / 2.0,
        FEET_Y as f32,
        (low[1] + high[1]) / 2.0,
    ];
    let span = (high[0] - low[0]).max(high[1] - low[1]);
    let corner = |dx: f32, dz: f32| {
        let reach = span * 0.55;
        [
            middle[0] + dx * reach,
            FEET_Y as f32 + span * 0.35,
            middle[2] + dz * reach,
        ]
    };
    let eye = FEET_Y as f32 + EYE;
    vec![
        (
            "above",
            [
                middle[0],
                FEET_Y as f32 + span * 0.9,
                middle[2] + 0.1 * span,
            ],
            middle,
        ),
        ("north_west", corner(-1.0, -1.0), middle),
        ("north_east", corner(1.0, -1.0), middle),
        ("south_east", corner(1.0, 1.0), middle),
        ("south_west", corner(-1.0, 1.0), middle),
        (
            "along_south",
            [middle[0], eye, high[1] + 2.0],
            [middle[0], FEET_Y as f32, middle[2]],
        ),
        (
            "along_west",
            [low[0] - 2.0, eye, middle[2]],
            [middle[0], FEET_Y as f32, middle[2]],
        ),
    ]
}
