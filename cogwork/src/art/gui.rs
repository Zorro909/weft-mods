//! The liquid bar on the pump's screen, drawn like the power bar beside it: a sunken track and
//! a fill. The bar shows any liquid, so its fill is a neutral glassy blue rather than one
//! liquid's color, and the track marks every bucket the pump holds.

use super::{CLEAR, Picture, Rgba};

const WIDTH: usize = 80;
const HEIGHT: usize = 8;
/// Buckets the pump holds: a mark on the track at each.
const BUCKETS: usize = (crate::pump::CAPACITY / crate::liquid::BUCKET) as usize;

/// The power bar's track colors: its rim, inner rim, face, and lower lip.
const RIM: Rgba = [0x14, 0x15, 0x20, 255];
const INNER: Rgba = [0x1d, 0x1e, 0x2d, 255];
const FACE: Rgba = [0x27, 0x28, 0x3b, 255];
const LIP: Rgba = [0x30, 0x32, 0x48, 255];
const MARK: Rgba = [0x3a, 0x3d, 0x58, 255];
/// The fill, its top row a bright surface, darkening toward the bottom.
const FILL: [Rgba; 6] = [
    [0xd8, 0xf0, 0xff, 255],
    [0x8c, 0xc8, 0xec, 255],
    [0x6a, 0xb0, 0xe0, 255],
    [0x50, 0x96, 0xd0, 255],
    [0x3c, 0x7c, 0xb8, 255],
    [0x2c, 0x62, 0x9c, 255],
];

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![("gui/tank", track()), ("gui/tank_fill", fill())]
}

/// The empty track, marked at every bucket.
fn track() -> Picture {
    let mut picture = Picture::sized(WIDTH, HEIGHT, RIM);
    picture.rect((1, 1), (WIDTH - 1, HEIGHT - 1), INNER);
    picture.rect((2, 2), (WIDTH - 1, HEIGHT - 2), FACE);
    picture.rect((2, HEIGHT - 2), (WIDTH - 1, HEIGHT - 1), LIP);
    for bucket in 1..BUCKETS {
        let x = bucket * WIDTH / BUCKETS;
        picture.rect((x, 2), (x + 1, HEIGHT - 2), MARK);
    }
    picture
}

/// The fill, inside the track's rim.
fn fill() -> Picture {
    let mut picture = Picture::sized(WIDTH, HEIGHT, CLEAR);
    for (row, color) in FILL.iter().enumerate() {
        picture.rect((1, row + 1), (WIDTH - 1, row + 2), *color);
    }
    picture
}
