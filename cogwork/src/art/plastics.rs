//! Pictures of the plastic works: the cream composite plate the plastic age's machines share, the
//! Gantry's frame of composite panels on steel, the sealed liquid tube and the pressure tube, the
//! polymerizer's sight glass, the press's ram and the Bit Magazine's drum, and the items: the
//! catalyst, pellets, sheets, gears, pipes, composite panels and the molds.

use super::liquids::{TUBE_EDGES, liquid_tube, liquid_tube_hub};
use super::{CLEAR, GLINT, Picture, Rgba, STEEL, Unwrap, noise};

/// Cream plastic, darkest to lightest.
const PLASTIC: [Rgba; 4] = [
    [0x8a, 0x84, 0x70, 255],
    [0xb8, 0xb0, 0x98, 255],
    [0xd8, 0xd0, 0xb8, 255],
    [0xf2, 0xee, 0xde, 255],
];
/// The pressure tube's dark slate plastic, darkest to lightest, and its warning band.
const SLATE: [Rgba; 3] = [
    [0x26, 0x2e, 0x38, 255],
    [0x3a, 0x46, 0x54, 255],
    [0x52, 0x60, 0x70, 255],
];
const WARNING: Rgba = [0xe0, 0xb0, 0x20, 255];
/// A rivet's head, lighter than any steel.
const RIVET: Rgba = [0x8a, 0x92, 0x9c, 255];
/// Amber resin and its bubbles, as the polymerizer shows through its glass; and the catalyst.
const RESIN: [Rgba; 3] = [
    [0x8a, 0x4a, 0x10, 255],
    [0xc8, 0x7a, 0x1c, 255],
    [0xf4, 0xc0, 0x50, 255],
];
const CATALYST: [Rgba; 3] = [
    [0x7a, 0x1e, 0x18, 255],
    [0xd0, 0x8a, 0x20, 255],
    [0xfa, 0xe0, 0x70, 255],
];
/// Texels a steel girder of the composite frame is wide.
const GIRDER: usize = 3;

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/composite_plate", plate()),
        ("block/composite_frame", frame()),
        ("block/sealed_liquid_tube", sleeved(liquid_tube())),
        ("block/sealed_liquid_tube_hub", sleeved(liquid_tube_hub())),
        ("block/pressure_tube", pressure_tube(false)),
        ("block/pressure_tube_hub", pressure_tube(true)),
        ("block/polymerizer_glass", sight_glass()),
        ("entity/molding_press_ram", ram()),
        ("entity/bit_magazine_drum", drum()),
        ("item/catalyst", beads(&CATALYST)),
        ("item/plastic_pellet", beads(&PLASTIC[..3])),
        ("item/plastic_sheet", sheet(false)),
        ("item/composite_panel", sheet(true)),
        ("item/plastic_gear", gear(&PLASTIC)),
        ("item/plastic_pipe", pipe()),
        ("item/sheet_mold", mold(Imprint::Sheet)),
        ("item/gear_mold", mold(Imprint::Gear)),
        ("item/pipe_mold", mold(Imprint::Pipe)),
        ("item/panel_mold", mold(Imprint::Panel)),
    ]
}

/// Whether a pixel of a 16 by 16 plate is a rivet: one in from each corner.
fn rivet(x: usize, y: usize) -> bool {
    (x == 1 || x == 14) && (y == 1 || y == 14)
}

/// Cream composite in a dark steel rim, riveted at its corners, with a faint grain.
fn plate() -> Picture {
    let mut picture = Picture::filled(PLASTIC[2]);
    picture.each(|x, y, _| {
        let rim = x == 0 || y == 0 || x == 15 || y == 15;
        match (rim, rivet(x, y)) {
            (true, _) => STEEL[1],
            (_, true) => RIVET,
            _ if x == 1 || y == 1 => PLASTIC[3],
            _ if x == 14 || y == 14 => PLASTIC[1],
            _ if noise(x, y, 811).is_multiple_of(7) => PLASTIC[1],
            _ => PLASTIC[2],
        }
    });
    picture
}

/// A face of the Gantry's frame: steel girders round its edge, riveted at the corners, holding a
/// composite panel with a round hole cut out of its middle, open through.
fn frame() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let girder = |at: usize| !(GIRDER..16 - GIRDER).contains(&at);
        let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
        let from_middle = (dx * dx + dy * dy).sqrt();
        match (girder(x) || girder(y), from_middle) {
            _ if (x == 1 || x == 14) && (y == 1 || y == 14) => RIVET,
            (true, _) if x == 0 || y == 0 || x == 15 || y == 15 => STEEL[0],
            (true, _) => STEEL[3],
            (false, r) if r < 3.0 => CLEAR,
            (false, r) if r < 4.0 => PLASTIC[1],
            _ if x == GIRDER || y == GIRDER => PLASTIC[3],
            _ => PLASTIC[2],
        }
    });
    picture
}

/// A liquid tube's glass with a cream plastic sleeve round each arm, near its end.
fn sleeved(mut picture: Picture) -> Picture {
    let [low, high] = TUBE_EDGES;
    let across = |at: usize| (low..=high).contains(&at);
    let band = |at: usize| (1..3).contains(&at) || (13..15).contains(&at);
    picture.each(|x, y, color| {
        let sleeve = (band(x) && across(y)) || (band(y) && across(x));
        let edge = [low, high].contains(&x) || [low, high].contains(&y);
        match (sleeve, edge) {
            (true, true) => PLASTIC[0],
            (true, false) if x == 1 || y == 1 || x == 13 || y == 13 => PLASTIC[3],
            (true, false) => PLASTIC[2],
            _ => color,
        }
    });
    picture
}

/// The pressure tube: dark slate plastic with a warning band near each arm's end, and a clear
/// window in the hub's faces to see how full it is.
fn pressure_tube(hub: bool) -> Picture {
    let mut picture = Picture::filled(SLATE[1]);
    picture.each(|x, y, _| {
        let window = hub && (6..10).contains(&x) && (6..10).contains(&y);
        let near_end = |at: usize| (1..3).contains(&at) || (13..15).contains(&at);
        let band = !hub && (near_end(x) || near_end(y));
        let edge = TUBE_EDGES.contains(&x) || TUBE_EDGES.contains(&y);
        match (window, edge, band) {
            (true, ..) if (x, y) == (6, 6) => GLINT,
            (true, ..) => CLEAR,
            (_, true, _) => SLATE[0],
            (_, _, true) => WARNING,
            _ if x == 5 || y == 5 => SLATE[2],
            _ => SLATE[1],
        }
    });
    picture
}

/// Amber resin behind the polymerizer's glass, bubbles rising through it, in a steel rim.
fn sight_glass() -> Picture {
    let mut picture = Picture::filled(RESIN[1]);
    picture.each(|x, y, _| {
        let rim = x == 0 || y == 0 || x == 15 || y == 15;
        let bubble = noise(x / 2, y / 2, 97).is_multiple_of(5) && (x + y).is_multiple_of(2);
        match (rim, bubble) {
            (true, _) => STEEL[1],
            (_, true) => RESIN[2],
            _ if y < 4 => RESIN[0],
            _ if x == 2 && y < 12 => GLINT,
            _ => RESIN[1],
        }
    });
    picture
}

/// The press's ram, a 64 by 32 texture: a steel platen polished on its face, and a bright rod.
fn ram() -> Picture {
    let mut picture = Picture::sized(64, 32, CLEAR);
    Unwrap::new((0, 0), (8, 2, 8)).paint(&mut picture, |x, y| {
        if noise(x, y, 5).is_multiple_of(5) {
            STEEL[2]
        } else {
            STEEL[3]
        }
    });
    Unwrap::new((0, 0), (8, 2, 8)).paint_caps(
        &mut picture,
        |_, _, edge| {
            if edge { STEEL[1] } else { STEEL[4] }
        },
    );
    Unwrap::new((32, 0), (2, 4, 2)).paint(
        &mut picture,
        |x, _| {
            if x % 2 == 0 { RIVET } else { STEEL[4] }
        },
    );
    picture
}

/// The Bit Magazine's drum, a 64 by 32 texture: two cream discs on a steel hub, and the steel
/// bits standing in it with cyan tips.
fn drum() -> Picture {
    let mut picture = Picture::sized(64, 32, CLEAR);
    for at in [(0, 0), (0, 11)] {
        let disc = Unwrap::new(at, (10, 1, 10));
        disc.paint(&mut picture, |_, _| PLASTIC[1]);
        disc.paint_caps(
            &mut picture,
            |_, _, edge| {
                if edge { STEEL[1] } else { PLASTIC[2] }
            },
        );
    }
    Unwrap::new((40, 0), (2, 7, 2)).paint(&mut picture, |_, _| STEEL[2]);
    Unwrap::new((48, 0), (2, 3, 2)).paint(&mut picture, |_, y| {
        if y < 3 {
            [0x62, 0xde, 0xdc, 255]
        } else {
            STEEL[3]
        }
    });
    picture
}

/// A heap of little round beads in `colors`, darkest to lightest.
fn beads(colors: &[Rgba]) -> Picture {
    let mut picture = Picture::filled(CLEAR);
    for (x, y) in [(5, 4), (9, 5), (2, 8), (7, 9), (11, 10), (4, 12)] {
        bead(&mut picture, (x, y), colors);
    }
    picture
}

/// A round bead four texels across with its top left corner at `(x, y)`: lit at the top left.
fn bead(picture: &mut Picture, (x, y): (usize, usize), colors: &[Rgba]) {
    let [dark, mid, light] = [colors[0], colors[1], colors[colors.len() - 1]];
    let rows = [
        [CLEAR, light, light, CLEAR],
        [light, mid, mid, dark],
        [mid, mid, dark, dark],
        [CLEAR, dark, dark, CLEAR],
    ];
    for (dy, row) in rows.iter().enumerate() {
        for (dx, &color) in row.iter().enumerate() {
            if color[3] > 0 {
                picture.set(x + dx, y + dy, color);
            }
        }
    }
}

/// A flat sheet of plastic seen a little from above; a composite panel shows its steel backing
/// under it, riveted.
fn sheet(on_steel: bool) -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.rect((2, 4), (14, 11), PLASTIC[2]);
    picture.rect((2, 4), (14, 5), PLASTIC[3]);
    picture.rect((13, 4), (14, 11), PLASTIC[1]);
    if on_steel {
        picture.rect((2, 11), (14, 13), STEEL[2]);
        picture.rect((2, 12), (14, 13), STEEL[0]);
        for x in [3, 12] {
            picture.set(x, 5, RIVET);
            picture.set(x, 9, RIVET);
        }
    } else {
        picture.rect((2, 11), (14, 12), PLASTIC[0]);
    }
    picture
}

/// A gear of `colors`, darkest to lightest: eight teeth round a hub with a hole, lit from above.
fn gear(colors: &[Rgba; 4]) -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
        let r = (dx * dx + dy * dy).sqrt();
        let sector = ((dy.atan2(dx) / std::f32::consts::TAU * 16.0).floor() as i32).rem_euclid(2);
        let solid = (1.6..5.2).contains(&r) || ((5.2..7.2).contains(&r) && sector == 0);
        match (solid, dy < -2.0, dy > 2.0) {
            (false, ..) => CLEAR,
            _ if r < 2.6 => colors[0],
            (true, true, _) => colors[3],
            (true, _, true) => colors[1],
            _ => colors[2],
        }
    });
    picture
}

/// A short length of plastic pipe lying from bottom left to top right, its ends open.
fn pipe() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let along = x + y;
        let across = x as i32 - y as i32;
        if !(8..=22).contains(&along) || across.abs() > 3 {
            return CLEAR;
        }
        match (along, across) {
            (8 | 22, -1..=1) => PLASTIC[0],
            (8 | 22, _) => PLASTIC[1],
            (_, -3 | -2) => PLASTIC[3],
            (_, 2 | 3) => PLASTIC[1],
            _ => PLASTIC[2],
        }
    });
    picture
}

/// What a mold's face is cut to.
#[derive(Clone, Copy)]
enum Imprint {
    Sheet,
    Gear,
    Pipe,
    Panel,
}

/// A steel mold block with the shape of its part cut into its face.
fn mold(imprint: Imprint) -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.rect((1, 2), (15, 14), STEEL[3]);
    picture.rect((1, 2), (15, 3), STEEL[4]);
    picture.rect((1, 13), (15, 14), STEEL[0]);
    picture.rect((14, 2), (15, 14), STEEL[1]);
    let cut = STEEL[0];
    match imprint {
        Imprint::Sheet => picture.rect((4, 5), (12, 11), cut),
        Imprint::Panel => {
            picture.rect((4, 5), (12, 11), cut);
            for (x, y) in [(5, 6), (10, 6), (5, 9), (10, 9)] {
                picture.set(x, y, RIVET);
            }
        }
        Imprint::Pipe => picture.rect((3, 7), (13, 10), cut),
        Imprint::Gear => {
            let shape = gear(&[cut, cut, cut, cut]);
            let mut small = Picture::filled(CLEAR);
            small.each(|x, y, _| shape.get((x * 16 / 10).min(15), (y * 16 / 10).min(15)));
            picture.overlay(&small, (3, 3));
        }
    }
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_is_open_through_its_middle() {
        let frame = frame();
        assert_eq!(frame.get(7, 7)[3], 0, "a hole through the panel");
        assert_eq!(frame.get(0, 7), STEEL[0], "a girder at its edge");
        assert_eq!(frame.get(4, 11), PLASTIC[2], "the panel round the hole");
    }

    #[test]
    fn the_pressure_tube_shows_its_fill_through_its_hub_only() {
        assert_eq!(pressure_tube(true).get(8, 8)[3], 0, "a window in the hub");
        assert_eq!(pressure_tube(false).get(8, 8)[3], 255, "the arms are solid");
    }

    #[test]
    fn the_sealed_tube_stays_glass_between_its_sleeves() {
        let tube = sleeved(liquid_tube());
        assert_eq!(
            tube.get(8, 8),
            liquid_tube().get(8, 8),
            "glass in the middle"
        );
        assert_eq!(tube.get(2, 8), PLASTIC[2], "a sleeve near the end");
    }
}
