//! Pictures of the steel works' items and liquid: coke, sulfur, the steel bloom, slag, the steel
//! ingot, the forge hammer, the steel drill bit as an item and on the rig, and creosote, still
//! and in its bucket.

use super::steel::FIRE;
use super::{IRON, OIL, Picture, Rgba, STEEL, noise};

/// Creosote: a dark amber-brown tar, dark to its sheen.
const CREOSOTE: [Rgba; 5] = [
    [0x1e, 0x10, 0x06, 255],
    [0x2c, 0x18, 0x08, 255],
    [0x3a, 0x22, 0x0c, 255],
    [0x6a, 0x40, 0x14, 255],
    [0x96, 0x64, 0x24, 255],
];
/// Frames of creosote's sheen, one every `creosote_still.anim`'s frametime.
const CREOSOTE_FRAMES: usize = 8;
/// Coke: grey-black and porous.
const COKE: [Rgba; 4] = [
    [0x24, 0x24, 0x28, 255],
    [0x3a, 0x3a, 0x40, 255],
    [0x56, 0x56, 0x5c, 255],
    [0x7a, 0x7a, 0x80, 255],
];
/// Sulfur, dark to light yellow.
const SULFUR: [Rgba; 4] = [
    [0x9a, 0x86, 0x10, 255],
    [0xc8, 0xb2, 0x20, 255],
    [0xe6, 0xd4, 0x3c, 255],
    [0xf8, 0xee, 0x8a, 255],
];
/// Slag: grey-brown glassy rock.
const SLAG: [Rgba; 4] = [
    [0x3c, 0x34, 0x30, 255],
    [0x58, 0x4c, 0x44, 255],
    [0x74, 0x68, 0x5a, 255],
    [0x8e, 0x86, 0x7a, 255],
];
/// A handle's wood, dark to light.
const HANDLE: [Rgba; 3] = [
    [0x4e, 0x36, 0x1e, 255],
    [0x6e, 0x4e, 0x2c, 255],
    [0x8c, 0x68, 0x3c, 255],
];

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/creosote_still", creosote_still()),
        ("item/creosote_bucket", bucket_of(&CREOSOTE)),
        ("item/coke", lump(&COKE, 461)),
        ("item/sulfur", sulfur()),
        (
            "item/steel_bloom",
            lump(&[FIRE[0], FIRE[1], FIRE[2], FIRE[3]], 467),
        ),
        ("item/slag", lump(&SLAG, 479)),
        ("item/steel_ingot", ingot()),
        ("item/forge_hammer", hammer()),
        ("item/steel_bit", super::quarry::bit_icon(&steel_metal())),
        (
            "entity/steel_drill_bit",
            super::quarry::drill_bit(&steel_metal()),
        ),
    ]
}

/// Steel as a bit is drawn: dark to light, bluer than iron.
fn steel_metal() -> [Rgba; 4] {
    [STEEL[0], STEEL[2], STEEL[4], [0x9a, 0xa4, 0xb0, 255]]
}

/// Creosote: dark amber, a band of sheen drifting across it as over oil.
fn creosote_still() -> Picture {
    let frames: Vec<Picture> = (0..CREOSOTE_FRAMES)
        .map(|frame| {
            let mut picture = Picture::surface(&CREOSOTE[..3], 487);
            picture.each(|x, y, color| {
                let band = (x + 2 * y + 16 - 2 * frame % 16) % 16;
                match (band, noise(x, y, 491) % 3) {
                    (0, 0) => CREOSOTE[4],
                    (0 | 1, _) => CREOSOTE[3],
                    _ => color,
                }
            });
            picture
        })
        .collect();
    Picture::stacked(&frames)
}

/// An iron bucket full of `liquid`, as the oil bucket is drawn.
pub(super) fn bucket_of(liquid: &[Rgba; 5]) -> Picture {
    let mut picture = super::liquids::oil_bucket();
    picture.each(|_, _, color| {
        if color == OIL[0] {
            liquid[1]
        } else if color == OIL[3] {
            liquid[3]
        } else {
            color
        }
    });
    picture
}

/// A rough lump of `stone`: a round heap, lit from the top left, pitted.
fn lump(stone: &[Rgba; 4], salt: u32) -> Picture {
    let rows = [
        "................",
        "................",
        "................",
        "......aaab......",
        "....aaabbbbc....",
        "...aabbbbbbcc...",
        "..aabbbbbbbccc..",
        "..abbbbbbbbccc..",
        "..abbbbbbbcccd..",
        "..bbbbbbbcccdd..",
        "...bbbbbccccd...",
        "....bccccddd....",
        "......cddd......",
        "................",
        "................",
        "................",
    ];
    let key = [
        ('a', stone[3]),
        ('b', stone[2]),
        ('c', stone[1]),
        ('d', stone[0]),
    ];
    let mut picture = Picture::drawn(&rows, &key);
    picture.each(|x, y, color| {
        let pit = color[3] > 0 && noise(x, y, salt).is_multiple_of(5);
        if pit { stone[0] } else { color }
    });
    picture
}

/// Sulfur: a small heap of yellow powder.
fn sulfur() -> Picture {
    let rows = [
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
        ".......aa.......",
        ".....aabbb......",
        "....abbbbbbc....",
        "...abbbbbbbcc...",
        "..abbbbbbbbccc..",
        "..bbbbbbbbcccd..",
        "...ccccccccdd...",
        "................",
        "................",
        "................",
    ];
    let key = [
        ('a', SULFUR[3]),
        ('b', SULFUR[2]),
        ('c', SULFUR[1]),
        ('d', SULFUR[0]),
    ];
    let mut picture = Picture::drawn(&rows, &key);
    picture.each(|x, y, color| {
        let grain = color[3] > 0 && noise(x, y, 499).is_multiple_of(4);
        if grain { SULFUR[1] } else { color }
    });
    picture
}

/// A steel ingot: a dark blue-grey bar, its top lit.
fn ingot() -> Picture {
    Picture::drawn(
        &[
            "................",
            "................",
            "................",
            "................",
            "................",
            "......llllllld..",
            "....llhhhhhhld..",
            "..llhhhhhhhlmd..",
            "..lmmmmmmmmlmd..",
            "..mmmmmmmmmmdd..",
            "..mmmmmmmmmmd...",
            "..dddddddddd....",
            "................",
            "................",
            "................",
            "................",
        ],
        &[
            ('l', STEEL[4]),
            ('h', [0x9a, 0xa4, 0xb0, 255]),
            ('m', STEEL[3]),
            ('d', STEEL[1]),
        ],
    )
}

/// The forge hammer: a heavy iron head on a wooden handle.
fn hammer() -> Picture {
    Picture::drawn(
        &[
            "................",
            "......llll......",
            ".....lmmmmd.....",
            "....lmmmmmmd....",
            ".....dmmmmmmd...",
            "......dmmmmd....",
            ".....wsdddd.....",
            "....wsw..d......",
            "...wsw..........",
            "..wsw...........",
            ".wsw............",
            "wsw.............",
            "sw..............",
            "................",
            "................",
            "................",
        ],
        &[
            ('l', IRON[3]),
            ('m', IRON[2]),
            ('d', IRON[0]),
            ('w', HANDLE[2]),
            ('s', HANDLE[0]),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_creosote_bucket_holds_creosote_not_oil() {
        let bucket = bucket_of(&CREOSOTE);
        let oily = (0..16).any(|y| (0..16).any(|x| bucket.get(x, y) == OIL[0]));
        assert!(!oily, "no oil left in it");
        assert_eq!(bucket.get(5, 3), CREOSOTE[1], "creosote where the oil was");
    }

    #[test]
    fn a_hammer_has_a_head_and_a_handle() {
        let hammer = hammer();
        assert_eq!(hammer.get(7, 3), IRON[2], "its head");
        assert_eq!(hammer.get(0, 12), HANDLE[0], "its handle");
    }
}
