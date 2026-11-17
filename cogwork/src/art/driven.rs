//! Pictures of the moving parts of the driven processing machines: the shaker screen's tray, an
//! iron frame round an iron wire mesh with a heap of Rubble on it, and the grinding mill's
//! runner, a granite stone on an iron spindle with an oak crown wheel. The stone, granite and
//! oak are the hand machines' (`processing`).

use super::processing::{GRANITE, OAK, STONE, oak};
use super::{CLEAR, IRON, Picture, Rgba, Unwrap, noise};

/// The tray's texture and where its model's unwrap puts each box: the back, front and two side
/// rims, the mesh, and the heap's two layers.
const TRAY_SIZE: (usize, usize) = (64, 64);
const TRAY_RIMS: [Unwrap; 4] = [
    Unwrap::new((0, 0), (14, 3, 1)),
    Unwrap::new((0, 4), (14, 1, 1)),
    Unwrap::new((32, 0), (1, 3, 12)),
    Unwrap::new((32, 16), (1, 3, 12)),
];
const TRAY_MESH: Unwrap = Unwrap::new((0, 32), (12, 1, 12));
const HEAP: [Unwrap; 2] = [
    Unwrap::new((0, 48), (9, 1, 7)),
    Unwrap::new((34, 48), (5, 1, 4)),
];
/// The runner's texture and its unwrap: the stone's core and four sides, the spindle, the crown
/// wheel and its four teeth.
const RUNNER_SIZE: (usize, usize) = (64, 32);
const STONE_BOXES: [Unwrap; 5] = [
    Unwrap::new((0, 0), (8, 4, 8)),
    Unwrap::new((0, 12), (1, 4, 6)),
    Unwrap::new((14, 12), (1, 4, 6)),
    Unwrap::new((28, 12), (6, 4, 1)),
    Unwrap::new((28, 17), (6, 4, 1)),
];
const SPINDLE: Unwrap = Unwrap::new((32, 0), (2, 7, 2));
const CROWN: [Unwrap; 5] = [
    Unwrap::new((40, 0), (6, 1, 6)),
    Unwrap::new((44, 8), (2, 1, 1)),
    Unwrap::new((50, 8), (2, 1, 1)),
    Unwrap::new((44, 10), (1, 1, 2)),
    Unwrap::new((50, 10), (1, 1, 2)),
];

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("entity/shaker_screen_tray", tray()),
        ("entity/grinding_mill_runner", runner()),
    ]
}

/// Iron, now and then a little darker.
fn iron(x: usize, y: usize) -> Rgba {
    if noise(x, y, 151).is_multiple_of(4) {
        IRON[1]
    } else {
        IRON[2]
    }
}

/// Iron wire woven every third pixel each way, clear between.
fn wire(x: usize, y: usize) -> Rgba {
    match (x % 3 == 1, y % 3 == 1) {
        (true, true) => IRON[3],
        (true, false) | (false, true) => IRON[2],
        (false, false) => CLEAR,
    }
}

/// Loose broken stone: chunks of two pixels, mostly mid grey, a few lit or in a dark gap.
fn rubble(x: usize, y: usize) -> Rgba {
    let shade = match noise(x / 2, y / 2, 157) % 6 {
        0 => 0,
        1 | 2 => 1,
        3 | 4 => 2,
        _ => 3,
    };
    STONE[shade]
}

/// The tray: iron rims, lit on top, round an iron wire mesh, and a heap of Rubble.
fn tray() -> Picture {
    let (width, height) = TRAY_SIZE;
    let mut picture = Picture::sized(width, height, CLEAR);
    for rim in TRAY_RIMS {
        rim.paint(&mut picture, iron);
        rim.paint_caps(
            &mut picture,
            |_, _, edge| if edge { IRON[2] } else { IRON[3] },
        );
    }
    TRAY_MESH.paint(&mut picture, iron);
    TRAY_MESH.paint_caps(&mut picture, |x, y, _| wire(x, y));
    for layer in HEAP {
        layer.paint(&mut picture, rubble);
    }
    picture
}

/// The runner: speckled granite with dressed grooves on top, an iron spindle and an oak crown
/// wheel with darker teeth.
fn runner() -> Picture {
    let (width, height) = RUNNER_SIZE;
    let mut picture = Picture::sized(width, height, CLEAR);
    let granite = |x: usize, y: usize| GRANITE[1 + (noise(x, y, 163) % 3) as usize];
    for stone in STONE_BOXES {
        stone.paint(&mut picture, granite);
        stone.paint_caps(&mut picture, |x, y, edge| {
            if edge || x % 3 == 0 {
                GRANITE[0]
            } else {
                granite(x, y)
            }
        });
    }
    SPINDLE.paint(&mut picture, iron);
    SPINDLE.paint_caps(&mut picture, |_, _, _| IRON[3]);
    let [wheel, teeth @ ..] = CROWN;
    wheel.paint(&mut picture, oak);
    wheel.paint_caps(
        &mut picture,
        |x, y, edge| if edge { OAK[0] } else { oak(x, y) },
    );
    for tooth in teeth {
        tooth.paint(&mut picture, |_, _| OAK[0]);
    }
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_textures_match_their_models() {
        let checks = [
            (
                include_str!("../../assets/models/shaker_screen_tray.geo.json"),
                TRAY_SIZE,
                [TRAY_RIMS.as_slice(), &[TRAY_MESH], &HEAP].concat(),
            ),
            (
                include_str!("../../assets/models/grinding_mill_runner.geo.json"),
                RUNNER_SIZE,
                [STONE_BOXES.as_slice(), &[SPINDLE], &CROWN].concat(),
            ),
        ];
        for (model, (width, height), boxes) in checks {
            for unwrap in boxes {
                let (w, h, d) = unwrap.size;
                let (u, v) = unwrap.at;
                let cube = format!("\"size\": [{w}, {h}, {d}], \"uv\": [{u}, {v}]");
                assert!(model.contains(&cube), "a box {cube}");
            }
            assert!(
                model.contains(&format!("\"texture_width\": {width}")),
                "as wide"
            );
            assert!(
                model.contains(&format!("\"texture_height\": {height}")),
                "as high"
            );
        }
    }

    #[test]
    fn the_mesh_lets_sand_through() {
        let holes = (0..12)
            .flat_map(|y| (0..12).map(move |x| (x, y)))
            .filter(|&(x, y)| wire(x, y) == CLEAR)
            .count();
        assert_eq!(holes, 64, "four in nine clear");
    }
}
