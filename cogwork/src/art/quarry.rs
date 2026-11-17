//! Pictures of the Timber Quarry: its block of planks on a cobblestone footing between log posts,
//! its lashed frame of log poles bound with string, its rig in the same wood, and the stone and
//! iron drill bits, as items and as the rig shows them. The rig's painters take a `RigPaint`, so
//! the Iron Quarry's rig (`iron`) is the same pieces in iron.

use super::{CLEAR, IRON, Picture, Rgba, Unwrap, noise};

/// Bark, dark to light.
pub const BARK: [Rgba; 4] = [
    [0x3a, 0x2a, 0x18, 255],
    [0x4e, 0x3a, 0x22, 255],
    [0x62, 0x4a, 0x2c, 255],
    [0x74, 0x5a, 0x36, 255],
];
/// Planks, dark to light, oak as base's.
pub const PLANK: [Rgba; 4] = [
    [0x6a, 0x4c, 0x28, 255],
    [0x94, 0x70, 0x3f, 255],
    [0xad, 0x85, 0x50, 255],
    [0xc0, 0x98, 0x60, 255],
];
/// String, its shade and its light.
pub const STRING: [Rgba; 2] = [[0x9c, 0x90, 0x74, 255], [0xcc, 0xc0, 0x9c, 255]];
/// Knapped stone for the stone bit, a warm grey, dark to light.
const FLINT: [Rgba; 4] = [
    [0x3e, 0x38, 0x34, 255],
    [0x5e, 0x56, 0x4e, 255],
    [0x7c, 0x72, 0x66, 255],
    [0x98, 0x8e, 0x80, 255],
];
/// Cobblestone, dark to light.
const COBBLE: [Rgba; 4] = [
    [0x4c, 0x4c, 0x4e, 255],
    [0x6c, 0x6c, 0x6e, 255],
    [0x84, 0x84, 0x86, 255],
    [0x9c, 0x9c, 0x9c, 255],
];

/// The rig's pieces, as the Gantry's models unwrap them.
const CARRIAGE: Unwrap = Unwrap::new((0, 0), (14, 8, 14));
const SHAFT: Unwrap = Unwrap::new((0, 0), (4, 64, 4));
const BEAM: Unwrap = Unwrap::new((0, 0), (64, 4, 4));
const BIT: Unwrap = Unwrap::new((0, 0), (8, 12, 8));
const TIP: Unwrap = Unwrap::new((32, 0), (4, 4, 4));
/// What a rig is painted with, by where a pixel is: the carriage's faces and their rims, the
/// poles of its shaft and beams and the bands round them once a block, and the cut ends of the
/// poles and their rims.
pub struct RigPaint {
    pub plate: fn(usize, usize) -> Rgba,
    pub rim: fn(usize, usize) -> Rgba,
    pub pole: fn(usize, usize) -> Rgba,
    pub band: fn(usize, usize) -> Rgba,
    pub cut: Rgba,
    pub cut_rim: Rgba,
}

/// The Timber Quarry's rig: planks rimmed in bark, log poles bound with string.
const TIMBER: RigPaint = RigPaint {
    plate: plank,
    rim: bark,
    pole: bark,
    band: twine,
    cut: PLANK[2],
    cut_rim: BARK[0],
};

/// Texels between the bands round a pole, and how wide a band is.
const BAND_EVERY: usize = 16;
const BAND: usize = 3;

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/timber_quarry_side", side(None)),
        ("block/timber_quarry_front", side(Some(false))),
        ("block/timber_quarry_front_on", side(Some(true))),
        ("block/timber_quarry_top", top()),
        ("block/lashed_frame_pole", Picture::surface(&BARK, 211)),
        ("block/lashed_frame_knot", knot()),
        ("block/lashed_frame_end", log_end()),
        ("item/stone_bit", bit_icon(&FLINT)),
        ("item/iron_bit", bit_icon(&IRON)),
        ("entity/timber_carriage", carriage(&TIMBER)),
        ("entity/timber_shaft", shaft(&TIMBER)),
        ("entity/timber_beam", beam(&TIMBER)),
        ("entity/stone_drill_bit", drill_bit(&FLINT)),
        ("entity/iron_drill_bit", drill_bit(&IRON)),
    ]
}

/// Plank grain: streaks along a row, a dark seam every fourth row.
pub fn plank(x: usize, y: usize) -> Rgba {
    if y % 4 == 3 {
        return PLANK[0];
    }
    PLANK[1 + (noise(x / 3, y, 223) % 3) as usize]
}

/// Bark grain: streaks along a column.
fn bark(x: usize, y: usize) -> Rgba {
    BARK[(noise(x, y / 3, 227) % 4) as usize]
}

/// Twine wound round a pole: light strands slanting across, shade between.
fn twine(x: usize, y: usize) -> Rgba {
    STRING[usize::from(!(x + y).is_multiple_of(3))]
}

/// The quarry's sides: planks over a cobblestone footing, log posts at both edges lashed with
/// string. A front shows a wooden cog, its spokes turned a little further while it works.
fn side(cog: Option<bool>) -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| match (x, y) {
        (0 | 1 | 14 | 15, 3 | 4 | 10 | 11) => twine(x, y),
        (0 | 1 | 14 | 15, _) => bark(x, y),
        (_, 12..) => COBBLE[(noise(x, y, 229) % 4) as usize],
        _ => plank(x, y),
    });
    if let Some(working) = cog {
        let rows = if working { COG_TURNED } else { COG };
        let cog = Picture::drawn(&rows, &[('d', PLANK[0]), ('l', PLANK[3]), ('h', BARK[0])]);
        picture.overlay(&cog, (4, 2));
    }
    picture
}

/// A wooden cog, 8 by 8, and the same a tooth further round.
pub const COG: [&str; 8] = [
    "...ll...", ".d.dd.d.", "..dddd..", "ldddhddl", "ldhdddl.", "..dddd..", ".d.dd.d.", "...ll...",
];
pub const COG_TURNED: [&str; 8] = [
    ".l....l.", "..dddd..", ".ddddddl", ".ddhdddd", "dddhddd.", "lddddd..", "..dddd..", ".l....l.",
];

/// The top: planks inside a log rim, the shaft's hole lashed round in the middle.
fn top() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let rim = !(2..=13).contains(&x) || !(2..=13).contains(&y);
        let hole = (6..10).contains(&x) && (6..10).contains(&y);
        let ring = (5..11).contains(&x) && (5..11).contains(&y);
        match (rim, hole, ring) {
            (true, ..) => bark(x, y),
            (_, true, _) => BARK[0],
            (_, _, true) => twine(x, y),
            _ => plank(x, y),
        }
    });
    picture
}

/// Where poles meet: twine wound crosswise over the bark.
fn knot() -> Picture {
    let mut picture = Picture::surface(&BARK, 211);
    picture.each(|x, y, color| {
        let middle = (5..11).contains(&x) && (5..11).contains(&y);
        let strand = !(x + y).is_multiple_of(3) || (x + 16 - y).is_multiple_of(4);
        if middle && strand { twine(x, y) } else { color }
    });
    picture
}

/// A sawn log's end: rings round a dark heart inside the bark.
fn log_end() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let (dx, dy) = (x.abs_diff(8), y.abs_diff(8));
        match dx.max(dy) {
            0..=1 => PLANK[0],
            2 | 4 => PLANK[1],
            3 | 5 => PLANK[3],
            6 => PLANK[2],
            _ => bark(x, y),
        }
    });
    picture
}

/// A drill bit in the hand: a spiralled cone of `metal` on a wooden shank bound with string.
fn bit_icon(metal: &[Rgba; 4]) -> Picture {
    const ROWS: [&str; 16] = [
        "................",
        "...........ws...",
        "..........wsw...",
        ".........sww....",
        "........ws......",
        ".......ttt......",
        "......tdt.......",
        ".....ldtl.......",
        "....lmdml.......",
        "...lmdmd........",
        "..lmdmdl........",
        "..mdmdl.........",
        ".mdmml..........",
        ".dmd............",
        ".dl.............",
        "................",
    ];
    let key = [
        ('w', PLANK[2]),
        ('s', PLANK[0]),
        ('t', STRING[1]),
        ('d', metal[0]),
        ('m', metal[2]),
        ('l', metal[3]),
    ];
    Picture::drawn(&ROWS, &key)
}

/// The carriage: plates, each face rimmed.
pub fn carriage(paint: &RigPaint) -> Picture {
    let mut picture = Picture::sized(64, 32, CLEAR);
    CARRIAGE.paint(&mut picture, paint.plate);
    let rim = |x, y, edge| {
        if edge {
            (paint.rim)(x, y)
        } else {
            (paint.plate)(x, y)
        }
    };
    CARRIAGE.paint_caps(&mut picture, rim);
    CARRIAGE.paint_ends(&mut picture, rim);
    picture
}

/// A shaft piece: a pole, banded every block.
pub fn shaft(paint: &RigPaint) -> Picture {
    let mut picture = Picture::sized(32, 80, CLEAR);
    let top_rows = SHAFT.size.2;
    SHAFT.paint(&mut picture, |x, y| {
        let along = y.saturating_sub(top_rows);
        if y >= top_rows && along % BAND_EVERY < BAND {
            (paint.band)(x, y)
        } else {
            (paint.pole)(x, y)
        }
    });
    let cut = |_, _, edge| if edge { paint.cut_rim } else { paint.cut };
    SHAFT.paint_caps(&mut picture, cut);
    picture
}

/// A beam piece: a pole lying along the gantry, banded every block.
pub fn beam(paint: &RigPaint) -> Picture {
    let mut picture = Picture::sized(144, 16, CLEAR);
    let (w, d) = (BEAM.size.0, BEAM.size.2);
    BEAM.paint(&mut picture, |x, y| {
        let along = match (y < d, x) {
            (true, _) => (x - d) % w,
            (false, x) if x < d => return paint.cut,
            (false, x) if x < d + w => x - d,
            (false, x) if x < 2 * d + w => return paint.cut,
            (false, x) => x - 2 * d - w,
        };
        if (along + BAND_EVERY / 2) % BAND_EVERY < BAND {
            (paint.band)(x, y)
        } else {
            (paint.pole)(x, y)
        }
    });
    picture
}

/// A bit on the rig: a spiral of `metal`, dark grooves winding round a lighter edge, and its tip.
fn drill_bit(metal: &[Rgba; 4]) -> Picture {
    let mut picture = Picture::sized(64, 32, CLEAR);
    let spiral = |x: usize, y: usize| match (x + y) % 8 {
        0 => metal[3],
        1 | 2 => metal[2],
        3..=5 => metal[1],
        _ => metal[0],
    };
    BIT.paint(&mut picture, spiral);
    TIP.paint(&mut picture, spiral);
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rig_pieces_fill_their_unwraps() {
        let carriage = carriage(&TIMBER);
        assert_ne!(carriage.get(14, 0)[3], 0, "the carriage's top");
        let beam = beam(&TIMBER);
        assert_ne!(beam.get(4, 0)[3], 0, "the beam's top");
        assert_ne!(beam.get(135, 7)[3], 0, "its far side");
        let shaft = shaft(&TIMBER);
        assert_ne!(shaft.get(15, 67)[3], 0, "the shaft's last row");
    }

    #[test]
    fn poles_are_banded_with_string_once_a_block() {
        let shaft = shaft(&TIMBER);
        let banded = (4..68)
            .filter(|&y| STRING.contains(&shaft.get(1, y)))
            .count();
        assert_eq!(banded, 4 * BAND, "four blocks, a band each");
    }
}
