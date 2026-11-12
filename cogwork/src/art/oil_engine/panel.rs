//! The tiles a formed oil engine's casing joins into one framed panel a face with (`compact`
//! connected textures: alone, joined all round, joined above and below, joined left and right,
//! an inner corner), on riveted plates of pale iron worn by use but kept up: a scratch, a few
//! specks of rust, no dents. A side's frame is copper posts left and right, a dark steel beam
//! on top with soot under it, and a grimy steel skirt at the bottom, bolted at the corners with
//! brass; the top and bottom faces' frame is dark steel all round (tiles 5 to 9, their own rule
//! in `oil_engine_panel.ctm`).

use crate::art::{BRASS, COPPER, Picture, Rgba, STEEL, noise};

/// Texels thick: the posts at the sides, the beam on top, the skirt at the bottom.
const POST: usize = 3;
const BEAM: usize = 3;
const SKIRT: usize = 5;
/// The panel's iron: its face, and the rare texel a shade lighter or darker, so a plate reads as
/// smooth metal and not as stone.
const IRON: [Rgba; 3] = [
    [0x90, 0x92, 0x96, 255],
    [0x8c, 0x8e, 0x92, 255],
    [0x94, 0x96, 0x9a, 255],
];
/// A plate's seams with the plates beside it: lit above and left, shaded below and right.
const SEAM_LIT: Rgba = [0xa8, 0xaa, 0xae, 255];
const SEAM_SHADE: Rgba = [0x6c, 0x6e, 0x72, 255];
/// A plate's rivet head, two texels wide: lit, mid and in shade.
const RIVET: [Rgba; 3] = [
    [0xc6, 0xc8, 0xca, 255],
    [0xa6, 0xa8, 0xab, 255],
    [0x66, 0x68, 0x6c, 255],
];
const SCRATCH: Rgba = [0xae, 0xb0, 0xb3, 255];
const RUST: [Rgba; 2] = [[0x86, 0x60, 0x44, 255], [0x9a, 0x72, 0x52, 255]];
/// Soot under a side's beam, darker near it.
const SOOT: [Rgba; 2] = [[0x74, 0x76, 0x7a, 255], [0x82, 0x84, 0x88, 255]];
/// Oily grime on the skirt and the plate just above it.
const GRIME: [Rgba; 3] = [
    [0x2a, 0x27, 0x24, 255],
    [0x35, 0x32, 0x2e, 255],
    [0x6e, 0x6b, 0x66, 255],
];

/// Which face of the engine a tile is for.
#[derive(Clone, Copy, PartialEq)]
enum Face {
    Side,
    TopOrBottom,
}

/// Which edges of a tile carry the frame, and whether it has the frame's corners alone.
#[derive(Clone, Copy)]
struct Edges {
    left_right: bool,
    top_bottom: bool,
    corners: bool,
}

/// The five tiles of `compact`, in its order.
const TILES: [Edges; 5] = [
    Edges::new(true, true, false),
    Edges::new(false, false, false),
    Edges::new(true, false, false),
    Edges::new(false, true, false),
    Edges::new(false, false, true),
];

impl Edges {
    const fn new(left_right: bool, top_bottom: bool, corners: bool) -> Edges {
        Edges {
            left_right,
            top_bottom,
            corners,
        }
    }
}

/// What a texel of a tile is: a side's copper post, or the top's steel rail along its sides.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Trim {
    Plate,
    Post,
    Rail,
    Beam,
    Skirt,
    Corner,
}

/// The tiles' textures: a side's 0 to 4, then the top's 5 to 9.
const NAMES: [&str; 10] = [
    "block/oil_engine_panel/0",
    "block/oil_engine_panel/1",
    "block/oil_engine_panel/2",
    "block/oil_engine_panel/3",
    "block/oil_engine_panel/4",
    "block/oil_engine_panel/5",
    "block/oil_engine_panel/6",
    "block/oil_engine_panel/7",
    "block/oil_engine_panel/8",
    "block/oil_engine_panel/9",
];

/// The tiles, each with its texture.
pub fn tiles() -> Vec<(&'static str, Picture)> {
    let faces = [Face::Side, Face::TopOrBottom];
    let tiles = faces
        .into_iter()
        .flat_map(|face| TILES.map(|edges| tile(face, edges)));
    NAMES.into_iter().zip(tiles).collect()
}

/// The pale iron plate every part of the engine is made of, a block's worth, its seams with
/// the plates beside it lit and shaded, worn a little, riveted along its top and bottom.
pub fn plate() -> Picture {
    let mut picture = Picture::filled(IRON[0]);
    picture.each(|x, y, _| match (x, y, noise(x, y, 101) % 16) {
        (_, 15, _) | (15, _, _) => SEAM_SHADE,
        (_, 0, _) | (0, _, _) => SEAM_LIT,
        (.., 0) => IRON[1],
        (.., 1) => IRON[2],
        _ => IRON[0],
    });
    for at in [(4, 6), (5, 7), (6, 7), (7, 8)] {
        picture.set(at.0, at.1, SCRATCH);
    }
    for (at, shade) in [((11, 5), 0), ((12, 5), 1), ((5, 10), 1)] {
        picture.set(at.0, at.1, RUST[shade]);
    }
    for x in [2, 7, 12] {
        for y in [2, 12] {
            rivet(&mut picture, (x, y));
        }
    }
    picture
}

/// A plate's rivet head, its top left at `(x, y)`.
fn rivet(picture: &mut Picture, (x, y): (usize, usize)) {
    let [lit, mid, shade] = RIVET;
    picture.set(x, y, lit);
    picture.set(x + 1, y, mid);
    picture.set(x, y + 1, mid);
    picture.set(x + 1, y + 1, shade);
}

/// A tile of `face` with the frame along `edges`.
fn tile(face: Face, edges: Edges) -> Picture {
    let mut picture = plate();
    if face == Face::Side && edges.top_bottom {
        soot_and_grime(&mut picture);
    }
    picture.each(|x, y, color| paint(trim(face, edges, x, y), x, y).unwrap_or(color));
    bolt_the_frame(&mut picture, face, edges);
    picture
}

/// What the texel at (`x`, `y`) of a tile of `face` with the frame along `edges` is.
fn trim(face: Face, edges: Edges, x: usize, y: usize) -> Trim {
    let bottom = if face == Face::Side { SKIRT } else { BEAM };
    let on_side = x.min(15 - x) < POST;
    let on_end = y < BEAM || 15 - y < bottom;
    match (on_side && edges.left_right, on_end && edges.top_bottom) {
        (true, true) => Trim::Corner,
        _ if on_side && on_end && edges.corners => Trim::Corner,
        (true, false) if face == Face::Side => Trim::Post,
        (true, false) => Trim::Rail,
        (false, true) if y < BEAM || face == Face::TopOrBottom => Trim::Beam,
        (false, true) => Trim::Skirt,
        (false, false) => Trim::Plate,
    }
}

/// The color of a texel of the frame that is `trim`, or none on the plate. Each edge of the
/// frame is dark along the panel's outside.
fn paint(trim: Trim, x: usize, y: usize) -> Option<Rgba> {
    let rough = noise(x, y, 131) % 6;
    let color = match trim {
        Trim::Plate => return None,
        Trim::Corner => STEEL[1],
        Trim::Post | Trim::Rail if x == 0 || x == 15 => STEEL[0],
        Trim::Rail if x == 1 || x == 14 => STEEL[3 + (rough == 0) as usize],
        Trim::Rail => STEEL[2],
        Trim::Post if rough == 0 => COPPER[1],
        Trim::Post if x == 1 || x == 14 => COPPER[3],
        Trim::Post => COPPER[2],
        Trim::Beam | Trim::Skirt if y == 0 || y == 15 => STEEL[0],
        Trim::Beam if y == 1 || y == 14 => STEEL[3 + (rough == 0) as usize],
        Trim::Beam => STEEL[2],
        Trim::Skirt if y == 16 - SKIRT => STEEL[3],
        Trim::Skirt => [GRIME[0], GRIME[1], STEEL[1]][rough as usize % 3],
    };
    Some(color)
}

/// Soot down from the beam in a few streaks, and grime up from the skirt, both on the plate
/// and fading: a tile's top half for the top of a side, its bottom half for its foot.
fn soot_and_grime(picture: &mut Picture) {
    for x in 0..16 {
        // A streak under one texel in three, at most four long.
        let roll = noise(x, 0, 137);
        let length = if roll.is_multiple_of(3) {
            1 + (roll / 3 % 4) as usize
        } else {
            0
        };
        for down in 0..length {
            picture.set(x, BEAM + down, SOOT[(down * 2 >= length) as usize]);
        }
        for up in 1..=3 {
            // Each row up a little cleaner: one texel in two, three, four.
            if noise(x, up, 139).is_multiple_of(up as u32 + 1) {
                picture.set(x, 16 - SKIRT - up, GRIME[2]);
            }
        }
    }
}

/// Rivets along the frame's beam, skirt and posts, and a brass bolt in each of its corners.
fn bolt_the_frame(picture: &mut Picture, face: Face, edges: Edges) {
    let on = |trim: Trim, x: usize, y: usize| self::trim(face, edges, x, y) == trim;
    let bottom_row = if face == Face::Side { 13 } else { 14 };
    for x in [5, 9] {
        for (y, trim) in [(1, Trim::Beam), (bottom_row, Trim::Skirt), (14, Trim::Beam)] {
            if on(trim, x, y) {
                stud(picture, (x, y), STEEL[4], STEEL[0]);
            }
        }
    }
    for y in [4, 8] {
        for x in [1, 13] {
            if on(Trim::Post, x, y) {
                stud(picture, (x, y), COPPER[4], COPPER[0]);
            } else if on(Trim::Rail, x, y) {
                stud(picture, (x, y), STEEL[4], STEEL[0]);
            }
        }
    }
    let bolt_row = if face == Face::Side { 12 } else { 13 };
    for at in [(1, 1), (13, 1), (1, bolt_row), (13, bolt_row)] {
        if on(Trim::Corner, at.0, at.1) {
            bolt(picture, at);
        }
    }
}

/// A stud on the frame at `(x, y)`: a lit head, its shadow below right.
fn stud(picture: &mut Picture, (x, y): (usize, usize), head: Rgba, shadow: Rgba) {
    picture.set(x, y, head);
    picture.set(x + 1, y + 1, shadow);
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
    fn a_side_alone_has_its_beam_skirt_and_posts() {
        let alone = TILES[0];
        assert_eq!(trim(Face::Side, alone, 7, 1), Trim::Beam, "a beam on top");
        assert_eq!(trim(Face::Side, alone, 7, 12), Trim::Skirt, "a skirt below");
        assert_eq!(
            trim(Face::Side, alone, 1, 7),
            Trim::Post,
            "posts at the sides"
        );
        assert_eq!(trim(Face::Side, alone, 14, 13), Trim::Corner, "corners");
        assert_eq!(trim(Face::Side, alone, 7, 7), Trim::Plate, "plate inside");
    }

    #[test]
    fn the_top_has_the_beam_all_round() {
        let alone = TILES[0];
        let face = Face::TopOrBottom;
        assert_eq!(trim(face, alone, 7, 14), Trim::Beam, "a beam at the bottom");
        assert_eq!(trim(face, alone, 7, 11), Trim::Plate, "no skirt");
        assert_eq!(
            trim(face, alone, 1, 7),
            Trim::Rail,
            "steel at the sides too"
        );
    }

    #[test]
    fn joined_tiles_keep_only_their_edges() {
        let [_, joined, sides, ends, corners] = TILES;
        assert_eq!(trim(Face::Side, joined, 1, 1), Trim::Plate, "all plate");
        assert_eq!(trim(Face::Side, sides, 1, 1), Trim::Post, "posts run on");
        assert_eq!(trim(Face::Side, ends, 1, 1), Trim::Beam, "the beam runs on");
        assert_eq!(
            trim(Face::Side, corners, 1, 1),
            Trim::Corner,
            "an inner corner"
        );
        assert_eq!(trim(Face::Side, corners, 7, 1), Trim::Plate, "and no edge");
    }

    #[test]
    fn the_frame_fits_a_quadrant() {
        // `compact` cuts every tile in four, so no edge may be deeper than half of it.
        assert!(POST.max(BEAM).max(SKIRT) <= 8, "within half a tile");
    }
}
