//! Pictures of the pump: dark steel banded with brass, vents and a lamp on each side that lights
//! while it works, a cage of brass posts over its plunger, a hazard-striped foot, and its hose. Where a
//! picture lies on the model is `assets/models/block/pump.json`: the body's sides take rows 4 to
//! 13 of the side, the foot's sides rows 13 to 16 of the base.

use super::{BRASS, IRON, OIL, Picture, Rgba, STEEL, noise};

const HAZARD_YELLOW: Rgba = [0xec, 0xbc, 0x20, 255];
const HAZARD_BLACK: Rgba = [0x22, 0x22, 0x24, 255];
const LAMP_DARK: Rgba = [0x4a, 0x26, 0x14, 255];
const LAMP_LIT: Rgba = [0xff, 0xe0, 0x8a, 255];
const LAMP_GLOW: Rgba = [0xf0, 0xa8, 0x30, 255];

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/pump_side", body_side(false)),
        ("block/pump_side_on", body_side(true)),
        ("block/pump_top", top()),
        ("block/pump_base", base()),
        ("block/pump_brass", brass()),
        ("block/pump_rod", rod()),
        ("block/pump_hose", hose()),
    ]
}

/// A side of the body: a steel panel between brass bands with a rivet in each corner, three
/// vent louvres, and a lamp in a brass bezel, dark while idle and lit amber while it works.
fn body_side(working: bool) -> Picture {
    let mut picture = Picture::surface(&STEEL[1..4], 3);
    for y in [4, 12] {
        picture.rect((0, y), (16, y + 1), BRASS[2]);
        picture.rect((0, y + 1), (16, y + 2), BRASS[0]);
    }
    for (x, y) in [(1, 6), (14, 6), (1, 11), (14, 11)] {
        picture.set(x, y, STEEL[4]);
    }
    for y in [6, 8, 10] {
        picture.rect((3, y), (9, y + 1), STEEL[0]);
        picture.rect((3, y + 1), (9, y + 2), STEEL[4]);
    }
    let (lamp, glow) = if working {
        (LAMP_LIT, LAMP_GLOW)
    } else {
        (LAMP_DARK, LAMP_DARK)
    };
    let bezel = Picture::drawn(
        &[".bbb.", "bllgb", "bllgb", "bgggb", ".bbb."],
        &[('b', BRASS[3]), ('l', lamp), ('g', glow)],
    );
    picture.overlay(&bezel, (10, 6));
    picture
}

/// The top: a steel deck, its hatch round the hole the hose goes down.
fn top() -> Picture {
    let mut picture = Picture::surface(&STEEL[1..4], 7);
    picture.rect((4, 4), (12, 12), STEEL[0]);
    picture.rect((5, 5), (11, 11), BRASS[1]);
    picture.rect((6, 6), (10, 10), OIL[0]);
    for (x, y) in [(5, 5), (10, 5), (5, 10), (10, 10)] {
        picture.set(x, y, BRASS[4]);
    }
    picture
}

/// The foot: steel, its bottom three rows striped black and yellow like the excavator's.
fn base() -> Picture {
    let mut picture = Picture::surface(&STEEL[0..3], 13);
    picture.each(|x, y, color| match y {
        13..=15 if (x + y) % 4 < 2 => HAZARD_YELLOW,
        13..=15 => HAZARD_BLACK,
        12 => STEEL[4],
        _ => color,
    });
    picture
}

/// The cage's posts and cap: brass, worn lighter here and there.
fn brass() -> Picture {
    let mut picture = Picture::surface(&BRASS[1..4], 17);
    picture.each(|x, y, color| {
        if noise(x, y, 19).is_multiple_of(11) {
            BRASS[4]
        } else {
            color
        }
    });
    picture
}

/// The plunger: polished steel, lit down one side.
fn rod() -> Picture {
    let mut picture = Picture::filled(IRON[2]);
    picture.each(|x, _, _| match x % 4 {
        0 => IRON[1],
        1 => IRON[3],
        _ => IRON[2],
    });
    picture
}

/// The hose: black rubber, a brass band at the top and the middle of each block.
fn hose() -> Picture {
    let rubber = [
        [0x1c, 0x1d, 0x20, 255],
        [0x26, 0x28, 0x2c, 255],
        [0x30, 0x32, 0x37, 255],
    ];
    let mut picture = Picture::surface(&rubber, 23);
    for y in [0, 8] {
        picture.rect((0, y), (16, y + 1), BRASS[3]);
        picture.rect((0, y + 1), (16, y + 2), BRASS[1]);
    }
    picture
}
