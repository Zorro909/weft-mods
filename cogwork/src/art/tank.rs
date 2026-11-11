//! Pictures of the liquid tank: clear glass with a glint, a slim dark steel frame, brass corner
//! caps, and a riveted copper base round its bottom with a steel floor inside. Where a picture
//! lies on the model is `tank::shape`: the base's sides take its bottom five rows.

use super::{BRASS, CLEAR, COPPER, GLINT, Picture, STEEL};

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/liquid_tank_glass", glass()),
        ("block/liquid_tank_frame", frame()),
        ("block/liquid_tank_cap", cap()),
        ("block/liquid_tank_base", base()),
        ("block/liquid_tank_floor", floor()),
    ]
}

/// Clear glass with two glints running down across it.
fn glass() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    for step in 0..4 {
        picture.set(3 + step, 6 - step, GLINT);
        picture.set(10 + step, 13 - step, GLINT);
    }
    picture.set(4, 6, GLINT);
    picture
}

/// Dark steel, a lighter line along its top and left.
fn frame() -> Picture {
    let mut picture = Picture::surface(&STEEL[0..3], 29);
    picture.each(|x, y, color| if x == 0 || y == 0 { STEEL[3] } else { color });
    picture
}

/// Brass with a darker rim every three pixels, so each cap shows its edge and a rivet.
fn cap() -> Picture {
    let mut picture = Picture::surface(&BRASS[2..4], 37);
    picture.each(|x, y, color| match (x % 13, y % 13) {
        (0, _) | (_, 0) | (12, _) | (_, 12) => BRASS[0],
        (1, 1) | (11, 1) | (1, 11) | (11, 11) => BRASS[4],
        _ => color,
    });
    picture.rect((3, 3), (13, 13), BRASS[2]);
    picture
}

/// Copper plate, its bottom five rows a band between steel edges with a rivet every four pixels.
/// The band is flat copper: it is all of the base a side shows, and noise there reads as dirt.
fn base() -> Picture {
    let mut picture = Picture::surface(&COPPER[1..4], 41);
    picture.each(|x, y, color| match (y, x % 4) {
        (11 | 15, _) => STEEL[1],
        (13, 1) => COPPER[4],
        (14, 1) => COPPER[0],
        (12 | 13, _) => COPPER[3],
        (14, _) => COPPER[2],
        _ => color,
    });
    picture
}

/// The floor inside: a steel plate in four, riveted at the corners.
fn floor() -> Picture {
    let mut picture = Picture::surface(&STEEL[2..5], 43);
    picture.each(|x, y, color| match (x % 8, y % 8) {
        (0, _) | (_, 0) => STEEL[1],
        (1, 1) | (6, 1) | (1, 6) | (6, 6) => STEEL[4],
        _ => color,
    });
    picture
}
