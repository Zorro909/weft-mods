//! Pictures of the refinery column: its plain casing of dark riveted steel, and once built one
//! framed panel a face (`compact` connected textures, as the oil engine's): dark steel posts and
//! rails round the face and a copper band at the foot of every block, so a built column reads as
//! a stack of rings. Its ports are drawn on top of either: the heater's fire grate, the
//! condenser's cooling fins and the tap's flange and valve wheel. The controller's front has a
//! temperature dial and a lamp; and the screen's temperature bar has its band marked.

use super::{BRASS, CLEAR, COPPER, IRON, Picture, STEEL, noise};
use crate::refinery::heat::{BAND as HEAT_BAND, SCALE};

/// Texels thick: the frame round a formed face.
const FRAME: usize = 3;
/// The rows of the copper band at the foot of every block of a formed side.
const BAND: [usize; 2] = [12, 13];

/// Which face of the column a tile is for.
#[derive(Clone, Copy, PartialEq)]
enum Face {
    Side,
    TopOrBottom,
}

/// The frame's edges a tile of `compact` has: left and right, top and bottom, or its corners
/// alone; in its order: alone, joined all round, joined above and below, joined either side,
/// an inner corner.
const TILES: [(bool, bool, bool); 5] = [
    (true, true, false),
    (false, false, false),
    (true, false, false),
    (false, true, false),
    (false, false, true),
];

pub fn all() -> Vec<(&'static str, Picture)> {
    let mut pictures = vec![
        ("block/refinery_casing", casing()),
        ("block/refinery_heater_port", heater_port()),
        ("block/refinery_condenser_port", condenser_port()),
        ("block/refinery_tap_port", tap_port()),
        ("block/refinery_front", front(false)),
        ("block/refinery_front_on", front(true)),
        ("gui/refinery_heat", heat_track()),
        ("gui/refinery_heat_fill", heat_fill()),
    ];
    pictures.extend(TILE_NAMES.into_iter().zip(tiles()));
    pictures
}

/// The panel's tiles: a side's 0 to 4, then the top's 5 to 9.
const TILE_NAMES: [&str; 10] = [
    "block/refinery_panel/0",
    "block/refinery_panel/1",
    "block/refinery_panel/2",
    "block/refinery_panel/3",
    "block/refinery_panel/4",
    "block/refinery_panel/5",
    "block/refinery_panel/6",
    "block/refinery_panel/7",
    "block/refinery_panel/8",
    "block/refinery_panel/9",
];

fn tiles() -> Vec<Picture> {
    let faces = [Face::Side, Face::TopOrBottom];
    faces
        .into_iter()
        .flat_map(|face| TILES.map(|edges| tile(face, edges)))
        .collect()
}

/// Dark steel plate, a texel lighter or darker here and there.
fn plate(salt: u32) -> Picture {
    let mut picture = Picture::filled(STEEL[2]);
    picture.each(|x, y, _| match noise(x, y, salt) % 12 {
        0 => STEEL[1],
        1 => STEEL[3],
        _ => STEEL[2],
    });
    picture
}

/// The plain casing: a plate with lit and shaded seams and a rivet in each corner.
fn casing() -> Picture {
    let mut picture = plate(853);
    picture.each(|x, y, color| match (x, y) {
        (_, 15) | (15, _) => STEEL[0],
        (_, 0) | (0, _) => STEEL[3],
        _ => color,
    });
    for at in [(2, 2), (12, 2), (2, 12), (12, 12)] {
        rivet(&mut picture, at);
    }
    picture
}

/// A tile of `face` with the frame along `edges` (left and right, top and bottom, corners).
fn tile(face: Face, (left_right, top_bottom, corners): (bool, bool, bool)) -> Picture {
    let mut picture = plate(857);
    if face == Face::Side {
        for x in 0..16 {
            picture.set(x, BAND[0], COPPER[3]);
            picture.set(x, BAND[1], COPPER[1]);
        }
        for x in [3, 7, 11] {
            picture.set(x, BAND[0], COPPER[4]);
        }
    }
    picture.each(|x, y, color| {
        let on_side = x.min(15 - x) < FRAME;
        let on_end = y.min(15 - y) < FRAME;
        let framed =
            (on_side && left_right) || (on_end && top_bottom) || (on_side && on_end && corners);
        if !framed {
            return color;
        }
        let from_edge = x.min(15 - x).min(y.min(15 - y));
        [STEEL[0], STEEL[4], STEEL[1]][from_edge.min(FRAME - 1)]
    });
    for at in [(1, 1), (13, 1), (1, 13), (13, 13)] {
        let on_side = left_right || corners;
        let on_end = top_bottom || corners;
        if (on_side && on_end) || (left_right && top_bottom) {
            bolt(&mut picture, at);
        }
    }
    picture
}

/// The heater's fire grate: a dark iron box with bars over glowing embers.
fn heater_port() -> Picture {
    Picture::drawn(
        &[
            "................",
            "................",
            "................",
            "..dddddddddddd..",
            "..dmmmmmmmmmmd..",
            "..dmbobobobobd..",
            "..dmbrbrbrbrbd..",
            "..dmbobybobobd..",
            "..dmbrbobrbrbd..",
            "..dmbobobybobd..",
            "..dmbrbrbobrbd..",
            "..dmmmmmmmmmmd..",
            "..dddddddddddd..",
            "................",
            "................",
            "................",
        ],
        &[
            ('d', STEEL[0]),
            ('m', IRON[1]),
            ('b', IRON[0]),
            ('r', [0x9a, 0x2a, 0x10, 255]),
            ('o', [0xd8, 0x62, 0x18, 255]),
            ('y', [0xf4, 0xb8, 0x40, 255]),
        ],
    )
}

/// The condenser's cooling fins: pale steel slats round a water pipe.
fn condenser_port() -> Picture {
    Picture::drawn(
        &[
            "................",
            "..dddddddddddd..",
            "..dLsLsLsLsLsd..",
            "..dLsLsLsLsLsd..",
            "..dLsLsLsLsLsd..",
            "..dLsLsLsLsLsd..",
            "..dwwwwwwwwwwd..",
            "..dWWWWWWWWWWd..",
            "..dwwwwwwwwwwd..",
            "..dLsLsLsLsLsd..",
            "..dLsLsLsLsLsd..",
            "..dLsLsLsLsLsd..",
            "..dLsLsLsLsLsd..",
            "..dddddddddddd..",
            "................",
            "................",
        ],
        &[
            ('d', STEEL[0]),
            ('L', IRON[3]),
            ('s', IRON[1]),
            ('w', [0x2c, 0x5c, 0xa0, 255]),
            ('W', [0x5c, 0x94, 0xd4, 255]),
        ],
    )
}

/// The tap: a round flange with a red valve wheel, and a spout below.
fn tap_port() -> Picture {
    Picture::drawn(
        &[
            "................",
            "................",
            ".....dddddd.....",
            "....dmmmmmmd....",
            "...dmRRRRRRmd...",
            "...dmR.RR.Rmd...",
            "...dmRRbbRRmd...",
            "...dmRRbbRRmd...",
            "...dmR.RR.Rmd...",
            "...dmRRRRRRmd...",
            "....dmmmmmmd....",
            ".....ddmmdd.....",
            "......dmmd......",
            "......dmmd......",
            "......dccd......",
            "................",
        ],
        &[
            ('d', STEEL[0]),
            ('m', IRON[2]),
            ('R', [0xb0, 0x24, 0x1c, 255]),
            ('b', BRASS[3]),
            ('c', STEEL[1]),
        ],
    )
}

/// The controller's front: the casing with a temperature dial over a lamp, `lit` while it
/// distils.
fn front(lit: bool) -> Picture {
    let mut picture = casing();
    let lamp = if lit {
        [0xff, 0xa8, 0x30, 255]
    } else {
        [0x5a, 0x3a, 0x1a, 255]
    };
    let dial = Picture::drawn(
        &[
            "..bbbbbb..",
            ".bwwwwwwb.",
            "bwwwwwwrwb",
            "bwwwwwrwwb",
            "bwwwwrwwwb",
            "bwwwkwwwwb",
            "bwwwwwwwwb",
            ".bwwwwwwb.",
            "..bbbbbb..",
        ],
        &[
            ('b', BRASS[2]),
            ('w', [0xe8, 0xe4, 0xd8, 255]),
            ('r', [0xc0, 0x20, 0x18, 255]),
            ('k', STEEL[0]),
        ],
    );
    picture.overlay(&dial, (3, 2));
    picture.rect((6, 12), (10, 14), lamp);
    picture.rect((5, 11), (11, 12), STEEL[0]);
    picture.rect((5, 14), (11, 15), STEEL[0]);
    picture
}

/// The temperature bar's track: as the power bar's, its band a green stretch of the face.
fn heat_track() -> Picture {
    let (width, height) = (80, 8);
    let in_band = |x: usize| {
        let degrees = x as f32 / width as f32 * SCALE;
        HEAT_BAND.contains(&degrees)
    };
    let mut picture = Picture::sized(width, height, [0x14, 0x15, 0x20, 255]);
    picture.each(|x, y, color| {
        let inside = (1..width - 1).contains(&x) && (1..height - 1).contains(&y);
        match (inside, in_band(x)) {
            (false, _) => color,
            (true, true) => [0x1e, 0x4a, 0x2a, 255],
            (true, false) => [0x27, 0x28, 0x3b, 255],
        }
    });
    picture
}

/// The temperature bar's fill: cold blue, through amber, to red hot.
fn heat_fill() -> Picture {
    let (width, height) = (80, 8);
    let mut picture = Picture::sized(width, height, CLEAR);
    picture.each(|x, y, _| {
        if !(1..width - 1).contains(&x) || !(1..height - 1).contains(&y) {
            return CLEAR;
        }
        let share = x as f32 / width as f32;
        let lit = if y == 1 { 40 } else { 0 };
        let mix = |cold: u8, hot: u8| {
            let blend = cold as f32 + (hot as f32 - cold as f32) * share;
            (blend as u8).saturating_add(lit)
        };
        [mix(0x40, 0xe0), mix(0x70, 0x50), mix(0xc0, 0x20), 255]
    });
    picture
}

/// A rivet head, its top left at `(x, y)`: lit, mid and in shade.
fn rivet(picture: &mut Picture, (x, y): (usize, usize)) {
    picture.set(x, y, STEEL[4]);
    picture.set(x + 1, y, STEEL[3]);
    picture.set(x, y + 1, STEEL[3]);
    picture.set(x + 1, y + 1, STEEL[0]);
}

/// A brass bolt head two texels wide, its top left at `(x, y)`.
fn bolt(picture: &mut Picture, (x, y): (usize, usize)) {
    picture.set(x, y, BRASS[4]);
    picture.set(x + 1, y, BRASS[2]);
    picture.set(x, y + 1, BRASS[2]);
    picture.set(x + 1, y + 1, BRASS[0]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_formed_side_has_a_copper_band_where_it_is_not_framed() {
        let joined = &tiles()[1];
        assert_eq!(joined.get(8, BAND[0]), COPPER[3], "the band's lit row");
        assert_eq!(joined.get(8, 4), plate(857).get(8, 4), "plate above it");
        let alone = &tiles()[0];
        assert_eq!(alone.get(0, BAND[0]), STEEL[0], "the frame over the band");
    }

    #[test]
    fn the_top_has_no_band() {
        let top = &tiles()[6];
        assert_ne!(top.get(8, BAND[0]), COPPER[3], "plain plate on top");
    }

    #[test]
    fn the_track_marks_the_band_alone() {
        let track = heat_track();
        let green = [0x1e, 0x4a, 0x2a, 255];
        let marked = (0..80).filter(|&x| track.get(x, 4) == green).count();
        assert!(
            (8..=12).contains(&marked),
            "{marked} texels: 60 degrees of 500"
        );
    }
}
