//! Layouts: several structures placed side by side and photographed together, to see tiles
//! such as paths and streets join up, or a whole settlement. A layout file has one structure a
//! line, and may name views of its own:
//!
//! ```text
//! # name           turn   x   z   [styles]
//! path_corner      right  0   0
//! hall_house       left   5   0   brick_roof,cobble_ground
//! # view name  eye x y z   target x y z    (y above the ground)
//! view  green  30 2 50     40 0 30
//! ```
//!
//! `x` and `z` are the north-west corner of the structure as placed (turned), on the
//! flatlands; `#` starts a comment. A structure is a file of this mod's `assets/structures/`,
//! or of the layout's own folder. Of the styles, only `cobble_ground` shows, as the paved copy
//! of a building: the engine's `/structure load` takes no palette, so the rest show their
//! defaults.

use std::path::Path;

use crate::{EYE, Read, load, read, view_line};

/// Where a player stands on the flatlands: on the grass, at y 3.
const FEET_Y: i32 = 4;

pub struct Layout {
    placed: Vec<Placed>,
    views: Vec<(String, [f32; 3], [f32; 3])>,
}

/// One structure of a layout.
struct Placed {
    /// `/structure load`'s words for it, the corner to go in.
    load: String,
    corner: [i32; 2],
    /// As turned.
    size: [i32; 3],
}

/// The layout in `file`.
pub fn read_layout(file: &str) -> Layout {
    let text =
        std::fs::read_to_string(file).unwrap_or_else(|e| panic!("could not read {file}: {e}"));
    let folders = [
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/structures"),
        Path::new(file)
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf(),
    ];
    let mut layout = Layout {
        placed: Vec::new(),
        views: Vec::new(),
    };
    for line in text
        .lines()
        .map(|line| line.split('#').next().unwrap_or("").trim())
    {
        let words: Vec<&str> = line.split_whitespace().collect();
        let number = |v: &str| {
            v.parse::<f32>()
                .unwrap_or_else(|_| panic!("{file}: {v} in {line:?}"))
        };
        match words[..] {
            [] => {}
            ["view", name, ex, ey, ez, tx, ty, tz] => {
                let lift = |y: &str| number(y) + FEET_Y as f32;
                let eye = [number(ex), lift(ey), number(ez)];
                layout
                    .views
                    .push((name.to_string(), eye, [number(tx), lift(ty), number(tz)]));
            }
            [name, turn, x, z, ref styles @ ..] => {
                let file = folders
                    .iter()
                    .map(|f| f.join(format!("{name}.json")))
                    .find(|f| f.exists());
                let file = file.unwrap_or_else(|| panic!("no structure file for {name}"));
                let read: Read = read(&file.to_string_lossy());
                let paved = styles
                    .iter()
                    .any(|s| s.split(',').any(|s| s == "cobble_ground"));
                layout.placed.push(Placed {
                    load: load(&read, turn, paved),
                    corner: [number(x) as i32, number(z) as i32],
                    size: crate::turned(read.size, turn),
                });
            }
            _ => panic!("{file}: {line:?} is not `name turn x z [styles]`, nor a view"),
        }
    }
    layout
}

/// Places every structure of the layout, then photographs all of it: from above, from each
/// corner, along it at eye height, and from the layout's own views. Shots are
/// `<label>-<view>.png`.
pub fn shots(layout: &Layout, label: &str) -> String {
    let mut lines = Vec::new();
    for placed in &layout.placed {
        lines.extend(place(placed));
    }
    lines.push("wait 1".to_string());
    lines.push("idle".to_string());
    for (view, eye, target) in views(&layout.placed)
        .into_iter()
        .chain(layout.views.clone())
    {
        lines.extend(view_line(label, &view, eye, target));
    }
    lines.join("\n")
}

/// Goes over `placed`, so its ground is loaded, and places it at its corner.
fn place(placed: &Placed) -> Vec<String> {
    let [x, z] = placed.corner;
    let [sx, _, sz] = placed.size;
    vec![
        format!(
            "via {}.5 {} {}.5 0 0",
            x + sx / 2,
            FEET_Y as f32 + EYE,
            z + sz / 2
        ),
        "travel 0.1".to_string(),
        // `travel` only starts the move: a load before it ends may find the ground not loaded.
        "wait 0.4".to_string(),
        crate::place(&placed.load, [x, FEET_Y - 1, z]),
        "wait 0.2".to_string(),
    ]
}

/// The views every layout gets: above it, from its four corners, and along it at eye height.
fn views(placed: &[Placed]) -> Vec<(String, [f32; 3], [f32; 3])> {
    let low = [0, 1].map(|i| placed.iter().map(|p| p.corner[i]).min().unwrap_or(0) as f32);
    let high = [0, 1].map(|i| {
        let ends = placed.iter().map(|p| p.corner[i] + p.size[i * 2]);
        ends.max().unwrap_or(0) as f32
    });
    let ground = FEET_Y as f32;
    let middle = [(low[0] + high[0]) / 2.0, ground, (low[1] + high[1]) / 2.0];
    let span = (high[0] - low[0]).max(high[1] - low[1]);
    let corner = |dx: f32, dz: f32| {
        let reach = span * 0.42;
        [
            middle[0] + dx * reach,
            ground + span * 0.25,
            middle[2] + dz * reach,
        ]
    };
    let eye = ground + EYE;
    [
        (
            "above",
            [middle[0], ground + span * 0.75, middle[2] + 0.1 * span],
            middle,
        ),
        ("north_west", corner(-1.0, -1.0), middle),
        ("north_east", corner(1.0, -1.0), middle),
        ("south_east", corner(1.0, 1.0), middle),
        ("south_west", corner(-1.0, 1.0), middle),
        ("along_south", [middle[0], eye, high[1] + 2.0], middle),
        ("along_west", [low[0] - 2.0, eye, middle[2]], middle),
    ]
    .map(|(name, eye, target)| (name.to_string(), eye, target))
    .into()
}
