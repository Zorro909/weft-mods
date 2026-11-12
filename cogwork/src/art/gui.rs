//! The liquid bars on the pump's and the oil engine's screens, drawn like the power bar beside
//! them: a sunken track and a fill, the track marked at every bucket the machine holds. The
//! pump's bar shows any liquid, so its fill is a neutral glassy blue rather than one liquid's
//! color; the engine has a bar of oil and one of water.

use super::{CLEAR, OIL, Picture, Rgba};

const WIDTH: usize = 80;
const HEIGHT: usize = 8;
/// Buckets the pump holds, and the engine of each liquid: a mark on the track at each.
const PUMP_BUCKETS: usize = (crate::pump::CAPACITY / crate::liquid::BUCKET) as usize;
const ENGINE_BUCKETS: usize =
    (crate::oil_engine::fuel::LIQUID_STORED / crate::liquid::BUCKET) as usize;

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

/// Oil in a bar: its sheen on top, darkening down.
const OIL_FILL: [Rgba; 6] = [OIL[4], OIL[3], OIL[2], OIL[2], OIL[1], OIL[0]];

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("gui/tank", track(PUMP_BUCKETS)),
        ("gui/tank_fill", fill(&FILL)),
        ("gui/oil_engine_oil", track(ENGINE_BUCKETS)),
        ("gui/oil_engine_oil_fill", fill(&OIL_FILL)),
        ("gui/oil_engine_water", track(ENGINE_BUCKETS)),
        ("gui/oil_engine_water_fill", fill(&FILL)),
    ]
}

/// The empty track, marked at every one of `buckets`.
fn track(buckets: usize) -> Picture {
    let mut picture = Picture::sized(WIDTH, HEIGHT, RIM);
    picture.rect((1, 1), (WIDTH - 1, HEIGHT - 1), INNER);
    picture.rect((2, 2), (WIDTH - 1, HEIGHT - 2), FACE);
    picture.rect((2, HEIGHT - 2), (WIDTH - 1, HEIGHT - 1), LIP);
    for bucket in 1..buckets {
        let x = bucket * WIDTH / buckets;
        picture.rect((x, 2), (x + 1, HEIGHT - 2), MARK);
    }
    picture
}

/// The fill of `colors`, row by row from the top, inside the track's rim.
fn fill(colors: &[Rgba; 6]) -> Picture {
    let mut picture = Picture::sized(WIDTH, HEIGHT, CLEAR);
    for (row, color) in colors.iter().enumerate() {
        picture.rect((1, row + 1), (WIDTH - 1, row + 2), *color);
    }
    picture
}
