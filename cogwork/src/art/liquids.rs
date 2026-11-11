//! Pictures of oil and what carries it: oil itself (a slow sheen drifting over it), the ground
//! of a seep, the oil bucket, and the liquid tube's reinforced glass.

use super::{CLEAR, GLASS_METAL, GLINT, IRON, OIL, Picture, Rgba, noise};

/// Frames of the oil's sheen, one every `oil_still.anim`'s frametime.
const OIL_FRAMES: usize = 8;
/// Where the edges of a liquid tube lie in its picture: its hub is 4 to 12.
const TUBE_EDGES: [usize; 2] = [4, 11];

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/oil_still", oil_still()),
        ("block/oily_dirt", soaked(&DIRT, 31)),
        ("block/oil_sand", soaked(&SAND, 47)),
        ("block/liquid_tube", liquid_tube()),
        ("block/liquid_tube_hub", liquid_tube_hub()),
        ("item/oil_bucket", oil_bucket()),
    ]
}

/// Dirt and sand, as base's look, darkest last.
const DIRT: [Rgba; 4] = [
    [0x79, 0x55, 0x3a, 255],
    [0x8b, 0x63, 0x45, 255],
    [0x6a, 0x48, 0x30, 255],
    [0x5a, 0x3c, 0x27, 255],
];
const SAND: [Rgba; 4] = [
    [0xdb, 0xcf, 0xa3, 255],
    [0xe6, 0xdb, 0xb0, 255],
    [0xcf, 0xc0, 0x8f, 255],
    [0xbf, 0xae, 0x7d, 255],
];

/// Oil: nearly black, a band of sheen drifting across it frame by frame.
fn oil_still() -> Picture {
    let frames: Vec<Picture> = (0..OIL_FRAMES).map(oil_frame).collect();
    Picture::stacked(&frames)
}

fn oil_frame(frame: usize) -> Picture {
    let mut picture = Picture::surface(&OIL[..3], 11);
    picture.each(|x, y, color| {
        // A diagonal band, two pixels of every sixteen, moving two a frame.
        let band = (x + 2 * y + 16 - 2 * frame % 16) % 16;
        match (band, noise(x, y, 5) % 3) {
            (0, 0) => OIL[4],
            (0 | 1, _) => OIL[3],
            _ => color,
        }
    });
    picture
}

/// `ground` soaked in blots of oil, a glint of sheen here and there.
fn soaked(ground: &[Rgba; 4], salt: u32) -> Picture {
    let mut picture = Picture::surface(ground, salt);
    picture.each(|x, y, color| {
        // Blots are blocks of three by three pixels, roughly half of them soaked.
        let blot = noise(x / 3, y / 3, salt + 1) % 5;
        let edge = noise(x, y, salt + 2).is_multiple_of(4);
        match blot {
            0 | 1 if noise(x, y, salt + 3).is_multiple_of(9) => OIL[4],
            0 | 1 => OIL[1],
            2 if !edge => OIL[2],
            _ => color,
        }
    });
    picture
}

/// The liquid tube's glass: clear, a thin iron edge along both sides of every arm and around
/// the hub, and one streak of metallic sheen with a glint across each, so it reads as
/// reinforced glass without hiding the liquid.
fn liquid_tube() -> Picture {
    let inside = |at: usize| (TUBE_EDGES[0]..=TUBE_EDGES[1]).contains(&at);
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let on_edge = TUBE_EDGES.contains(&x) || TUBE_EDGES.contains(&y);
        if on_edge && (inside(x) || inside(y)) {
            IRON[1]
        } else {
            CLEAR
        }
    });
    // A streak across the hub and across each arm: sheen with a glint at its head.
    for (x, y) in [
        (6, 8),
        (7, 7),
        (1, 8),
        (2, 7),
        (13, 8),
        (14, 7),
        (7, 2),
        (8, 1),
        (7, 14),
        (8, 13),
    ] {
        picture.set(x, y, GLASS_METAL);
    }
    for (x, y) in [(8, 6), (3, 6), (15, 6), (9, 0), (9, 12)] {
        picture.set(x, y, GLINT);
    }
    picture
}

/// The hub: the tube's glass, held at its corners by iron brackets with a rivet each.
fn liquid_tube_hub() -> Picture {
    let mut picture = liquid_tube();
    let bracket = Picture::drawn(&["mmm", "mr.", "m.."], &[('m', IRON[1]), ('r', IRON[2])]);
    let [low, high] = TUBE_EDGES;
    for (x, y, flip_x, flip_y) in [
        (low, low, false, false),
        (high - 2, low, true, false),
        (low, high - 2, false, true),
        (high - 2, high - 2, true, true),
    ] {
        for by in 0..3 {
            for bx in 0..3 {
                let from = (
                    if flip_x { 2 - bx } else { bx },
                    if flip_y { 2 - by } else { by },
                );
                let color = bracket.get(from.0, from.1);
                if color[3] > 0 {
                    picture.set(x + bx, y + by, color);
                }
            }
        }
    }
    picture
}

/// A bucket of oil, seen a little from above.
fn oil_bucket() -> Picture {
    Picture::drawn(
        &[
            "................",
            "................",
            "....dLLLLLLd....",
            "..dLoooOOoooLd..",
            "..dLoOooooooLd..",
            "..dmLLLLLLLLmd..",
            "...dmmmmmmmmd...",
            "...dmMmmmmmmd...",
            "...dmMmmmmmmd...",
            "....dmmmmmmd....",
            "....dmMmmmmd....",
            "....dmmmmmmd....",
            ".....dmmmmd.....",
            ".....dddddd.....",
            "................",
            "................",
        ],
        &[
            ('d', IRON[0]),
            ('m', IRON[2]),
            ('M', IRON[3]),
            ('L', [0xc6, 0xc6, 0xc6, 255]),
            ('o', OIL[0]),
            ('O', OIL[3]),
        ],
    )
}
