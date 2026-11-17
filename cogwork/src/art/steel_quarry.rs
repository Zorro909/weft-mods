//! Pictures of the Steel Excavator: its block of riveted dark steel plates, the steel plate the
//! steel age's machines share, its frame of steel lattice, its rig, the Timber Quarry's pieces
//! (`quarry`) in steel, and the diamond-tipped drill bit in the hand and on the rig.

use super::quarry::{COG, COG_TURNED, RigPaint, beam, bit_icon, carriage, drill_bit, shaft};
use super::{CLEAR, Picture, Rgba, STEEL, noise};

/// The Steel Excavator's rig: plates rimmed darkest, poles with riveted bands.
const STEEL_RIG: RigPaint = RigPaint {
    plate: steel,
    rim: |_, _| STEEL[0],
    pole: steel,
    band,
    cut: STEEL[2],
    cut_rim: STEEL[0],
};
/// A rivet's head, lighter than any plate.
const RIVET: Rgba = [0x8a, 0x92, 0x9c, 255];
/// Diamond, dark to light, as the tip of a bit.
const DIAMOND: [Rgba; 4] = [
    [0x14, 0x5e, 0x66, 255],
    [0x2a, 0xa8, 0xb0, 255],
    [0x62, 0xde, 0xdc, 255],
    [0xc0, 0xfa, 0xf4, 255],
];
/// Texels a lattice girder is wide.
const GIRDER: usize = 3;

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/steel_plate", plate(None)),
        ("block/steel_excavator_side", plate(None)),
        ("block/steel_excavator_front", plate(Some(false))),
        ("block/steel_excavator_front_on", plate(Some(true))),
        ("block/steel_excavator_top", top()),
        ("block/steel_frame", lattice()),
        ("entity/steel_carriage", carriage(&STEEL_RIG)),
        ("entity/steel_shaft", shaft(&STEEL_RIG)),
        ("entity/steel_beam", beam(&STEEL_RIG)),
        ("item/diamond_bit", bit_icon(&DIAMOND)),
        ("entity/diamond_drill_bit", drill_bit(&DIAMOND)),
    ]
}

/// Plate steel: even, now and then a darker grain.
fn steel(x: usize, y: usize) -> Rgba {
    if noise(x, y, 613).is_multiple_of(6) {
        STEEL[2]
    } else {
        STEEL[3]
    }
}

/// A riveted band round a pole: dark steel, a rivet every fourth texel.
fn band(x: usize, y: usize) -> Rgba {
    if (x + y).is_multiple_of(4) {
        RIVET
    } else {
        STEEL[1]
    }
}

/// Whether a pixel is a rivet of a 16 by 16 plate: one in from each corner, and every fourth
/// texel along its rim.
fn rivet(x: usize, y: usize) -> bool {
    let edge = |at: usize| at == 1 || at == 14;
    let along = |at: usize| at.is_multiple_of(4) && (2..14).contains(&at);
    (edge(x) && (edge(y) || along(y))) || (edge(y) && along(x))
}

/// A steel plate in a dark rim, riveted round. A front shows a steel cog, its teeth turned a
/// little further while it works.
fn plate(cog: Option<bool>) -> Picture {
    let mut picture = Picture::filled(STEEL[3]);
    picture.each(|x, y, _| {
        let rim = x == 0 || y == 0 || x == 15 || y == 15;
        match (rim, rivet(x, y), y > 0 && rivet(x, y - 1)) {
            (true, ..) => STEEL[0],
            (_, true, _) => RIVET,
            // Each rivet's shadow under it.
            (_, _, true) => STEEL[1],
            _ => steel(x, y),
        }
    });
    if let Some(working) = cog {
        let rows = if working { COG_TURNED } else { COG };
        let cog = Picture::drawn(&rows, &[('d', STEEL[0]), ('l', RIVET), ('h', STEEL[4])]);
        picture.overlay(&cog, (4, 4));
    }
    picture
}

/// The top: the riveted plate with the shaft's hole in the middle, a band round it.
fn top() -> Picture {
    let mut picture = plate(None);
    picture.each(|x, y, color| {
        let (dx, dy) = (x.abs_diff(8), y.abs_diff(8));
        let hole = (6..10).contains(&x) && (6..10).contains(&y);
        match (hole, dx.max(dy) <= 3 && !hole) {
            (true, _) => STEEL[0],
            (_, true) => band(x, y),
            _ => color,
        }
    });
    picture
}

/// A face of steel lattice: girders round its edge, riveted where they cross, and two braces
/// from corner to corner; open between.
fn lattice() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let girder = |at: usize| !(GIRDER..16 - GIRDER).contains(&at);
        let brace = x.abs_diff(y) <= 1 || (x + y).abs_diff(15) <= 1;
        let corner = girder(x) && girder(y);
        match (girder(x) || girder(y), brace) {
            _ if corner && (x == 1 || x == 14) && (y == 1 || y == 14) => RIVET,
            (true, _) if x == 0 || y == 0 || x == 15 || y == 15 => STEEL[0],
            (true, _) => steel(x, y),
            (false, true) if x.abs_diff(y) == 1 || (x + y).abs_diff(15) == 1 => STEEL[1],
            (false, true) => STEEL[2],
            _ => CLEAR,
        }
    });
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_lattice_is_open_between_its_girders_and_braces() {
        let lattice = lattice();
        assert_eq!(lattice.get(0, 7)[3], 255, "a girder at its edge");
        assert_eq!(lattice.get(8, 8)[3], 255, "the braces cross in the middle");
        assert_eq!(lattice.get(8, 4)[3], 0, "open between them");
        assert_eq!(lattice.get(1, 1), RIVET, "riveted at a corner");
    }

    #[test]
    fn a_steel_plate_is_riveted_round() {
        let plate = plate(None);
        assert_eq!(plate.get(1, 1), RIVET, "a corner rivet");
        assert_eq!(plate.get(1, 2), STEEL[1], "its shadow");
        assert_eq!(plate.get(0, 8), STEEL[0], "the rim");
    }
}
