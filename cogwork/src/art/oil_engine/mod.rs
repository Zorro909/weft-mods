//! Pictures of the oil engine: its parts in pale riveted iron, copper and dark steel, the tiles
//! its formed casing joins into one framed panel a face with (`compact` connected textures:
//! alone, joined all round, joined above and below, joined left and right, an inner corner),
//! the gauges and lamp on its controller, and its rig (`rig`).

mod rig;

use super::{BRASS, CLEAR, COPPER, IRON, OIL, Picture, Rgba, STEEL};

/// The engine's pale iron plate, its most common shade first.
const PLATE: [Rgba; 3] = [
    [0x8e, 0x90, 0x94, 255],
    [0x86, 0x88, 0x8c, 255],
    [0x98, 0x9a, 0x9e, 255],
];
/// A drive's blue, as a cool engine trunk's, dark to light.
const BLUE: [Rgba; 3] = [
    [0x24, 0x4c, 0x8c, 255],
    [0x34, 0x6c, 0xc0, 255],
    [0x5c, 0x98, 0xe4, 255],
];
/// Water in a gauge, dark to its surface.
const WATER: [Rgba; 3] = [
    [0x2c, 0x5c, 0xb4, 255],
    [0x3c, 0x78, 0xd4, 255],
    [0x8c, 0xc0, 0xf4, 255],
];
/// The lamp, out and lit: its face and its glint.
const LAMP_OUT: [Rgba; 2] = [[0x1e, 0x36, 0x22, 255], [0x2e, 0x4c, 0x32, 255]];
const LAMP_LIT: [Rgba; 2] = [[0x5c, 0xd8, 0x4c, 255], [0xd4, 0xff, 0xc0, 255]];
/// A gauge's empty glass.
const GAUGE_GLASS: Rgba = [0x26, 0x2c, 0x34, 255];

/// Where the gauges sit on the controller's front, as their model's faces show them: the left
/// texel of each, and the rows from the top they span.
const OIL_GAUGE_AT: usize = 3;
const WATER_GAUGE_AT: usize = 9;
const GAUGE_ROWS: (usize, usize) = (5, 14);
/// Rows of liquid a gauge shows at each step, of the 7 inside its frame.
const GAUGE_FILL: [usize; 5] = [0, 2, 4, 5, 7];
/// The lamp's top left texel on the controller's front.
const LAMP_AT: (usize, usize) = (7, 2);

/// Texels wide a formed panel's frame is: a dark line and two of copper.
const FRAME: usize = 3;

pub fn all() -> Vec<(&'static str, Picture)> {
    let mut all = vec![
        ("block/oil_engine_casing", casing()),
        ("block/combustion_chamber", chamber()),
        ("block/oil_intake", intake()),
        ("block/oil_engine_drive", drive()),
        ("block/oil_engine_controller_front", controller_front()),
        ("block/oil_engine_lamp", lamp(LAMP_OUT)),
        ("block/oil_engine_lamp_on", lamp(LAMP_LIT)),
        ("block/oil_engine_panel/0", panel(Edges::ALL, false)),
        ("block/oil_engine_panel/1", panel(Edges::NONE, false)),
        ("block/oil_engine_panel/2", panel(Edges::SIDES, false)),
        ("block/oil_engine_panel/3", panel(Edges::ENDS, false)),
        ("block/oil_engine_panel/4", panel(Edges::NONE, true)),
        ("entity/oil_engine_rig", rig::picture()),
    ];
    let gauges = [
        "block/oil_engine_gauge_oil_0",
        "block/oil_engine_gauge_oil_1",
        "block/oil_engine_gauge_oil_2",
        "block/oil_engine_gauge_oil_3",
        "block/oil_engine_gauge_oil_4",
        "block/oil_engine_gauge_water_0",
        "block/oil_engine_gauge_water_1",
        "block/oil_engine_gauge_water_2",
        "block/oil_engine_gauge_water_3",
        "block/oil_engine_gauge_water_4",
    ];
    for (index, name) in gauges.into_iter().enumerate() {
        let (at, liquid) = if index < 5 {
            (OIL_GAUGE_AT, [OIL[1], OIL[2], OIL[4]])
        } else {
            (WATER_GAUGE_AT, WATER)
        };
        all.push((name, gauge(at, GAUGE_FILL[index % 5], liquid)));
    }
    all
}

/// The pale iron surface every plate of the engine starts from.
fn plate(salt: u32) -> Picture {
    Picture::surface(&PLATE, salt)
}

/// A plate's edge: lit on its top and left, shaded on its bottom and right.
fn bevel(picture: &mut Picture, light: Rgba, shade: Rgba) {
    picture.each(|x, y, color| match (x, y) {
        (_, 0) | (0, _) => light,
        (_, 15) | (15, _) => shade,
        _ => color,
    });
}

/// A rivet at (`x`, `y`): a lit head with a shadow below right.
fn rivet(picture: &mut Picture, (x, y): (usize, usize), head: Rgba, shadow: Rgba) {
    picture.set(x, y, head);
    picture.set(x + 1, y + 1, shadow);
}

/// The four rivets near a plate's corners, `inset` texels in.
fn corner_rivets(picture: &mut Picture, inset: usize, head: Rgba, shadow: Rgba) {
    let far = 15 - inset - 1;
    for at in [(inset, inset), (far, inset), (inset, far), (far, far)] {
        rivet(picture, at, head, shadow);
    }
}

/// Casing, not yet part of a built engine: a riveted plate.
fn casing() -> Picture {
    let mut picture = plate(83);
    bevel(&mut picture, IRON[3], IRON[1]);
    corner_rivets(&mut picture, 2, IRON[3], IRON[0]);
    picture
}

/// The combustion chamber: a copper firebox with three vents, bolted with brass.
fn chamber() -> Picture {
    let mut picture = Picture::surface(&COPPER[2..4], 89);
    bevel(&mut picture, COPPER[4], COPPER[1]);
    for top in [4, 7, 10] {
        picture.rect((4, top), (12, top + 2), STEEL[0]);
        picture.rect((4, top + 2), (12, top + 3), COPPER[4]);
    }
    corner_rivets(&mut picture, 1, BRASS[3], BRASS[0]);
    picture
}

/// An intake: a copper flange round a dark mouth that liquid tubes join.
fn intake() -> Picture {
    let mut picture = casing();
    picture.each(|x, y, color| {
        let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
        let distance = (dx * dx + dy * dy).sqrt();
        let lit = dx + dy < 0.0;
        match distance {
            d if d < 2.6 => STEEL[0],
            d if d < 3.6 => STEEL[3],
            d if d < 5.8 && lit => COPPER[3],
            d if d < 5.8 => COPPER[1],
            d if d < 6.6 => COPPER[0],
            _ => color,
        }
    });
    for at in [(7, 3), (11, 7), (7, 11), (3, 7)] {
        picture.set(at.0, at.1, BRASS[3]);
    }
    picture
}

/// A drive: dark steel with a blue-rimmed socket for a shaft.
fn drive() -> Picture {
    let mut picture = Picture::surface(&STEEL[2..4], 97);
    bevel(&mut picture, STEEL[4], STEEL[0]);
    picture.rect((3, 3), (13, 13), BLUE[1]);
    picture.rect((3, 3), (13, 4), BLUE[2]);
    picture.rect((3, 3), (4, 13), BLUE[2]);
    picture.rect((5, 5), (11, 11), STEEL[3]);
    picture.rect((6, 6), (10, 10), STEEL[0]);
    picture.rect((12, 4), (13, 13), BLUE[0]);
    picture.rect((4, 12), (13, 13), BLUE[0]);
    corner_rivets(&mut picture, 1, BRASS[3], BRASS[0]);
    picture
}

/// The controller's front: a riveted plate with a dark panel behind its gauges and lamp.
fn controller_front() -> Picture {
    let mut picture = casing();
    picture.rect((2, 1), (14, 15), STEEL[0]);
    picture.rect((3, 2), (13, 14), STEEL[2]);
    for at in [(2, 1), (13, 1), (2, 14), (13, 14)] {
        picture.set(at.0, at.1, BRASS[3]);
    }
    picture
}

/// A gauge at texel `left` of the front showing `rows` of `liquid` (dark, light, its surface)
/// in brass-framed glass. Clear elsewhere: the model shows only the gauge's part.
fn gauge(left: usize, rows: usize, liquid: [Rgba; 3]) -> Picture {
    let mut picture = Picture::filled(CLEAR);
    let (top, bottom) = GAUGE_ROWS;
    picture.rect((left, top), (left + 4, bottom), BRASS[1]);
    picture.rect((left, top), (left + 4, top + 1), BRASS[3]);
    picture.rect((left, top), (left + 1, bottom), BRASS[3]);
    let inside = (top + 1, bottom - 1);
    picture.rect((left + 1, inside.0), (left + 3, inside.1), GAUGE_GLASS);
    for row in 0..rows {
        let y = inside.1 - 1 - row;
        let color = if row + 1 == rows {
            liquid[2]
        } else {
            liquid[row % 2]
        };
        picture.rect((left + 1, y), (left + 3, y + 1), color);
    }
    if rows < 7 {
        picture.set(left + 1, inside.0, IRON[2]);
    }
    picture
}

/// The lamp: a 2 by 2 light, its glint at the top left. Clear elsewhere.
fn lamp([face, glint]: [Rgba; 2]) -> Picture {
    let mut picture = Picture::filled(CLEAR);
    let (x, y) = LAMP_AT;
    picture.rect((x, y), (x + 2, y + 2), face);
    picture.set(x, y, glint);
    picture
}

/// Which edges of a panel tile carry the frame.
#[derive(Clone, Copy)]
struct Edges {
    left_right: bool,
    top_bottom: bool,
}

impl Edges {
    const ALL: Edges = Edges {
        left_right: true,
        top_bottom: true,
    };
    const NONE: Edges = Edges {
        left_right: false,
        top_bottom: false,
    };
    /// Joined above and below: the frame on its left and right.
    const SIDES: Edges = Edges {
        left_right: true,
        top_bottom: false,
    };
    /// Joined left and right: the frame on its top and bottom.
    const ENDS: Edges = Edges {
        left_right: false,
        top_bottom: true,
    };
}

/// A tile of a formed panel: the plate, with the frame along `edges`, and with `corners` the
/// frame's corner only, where two joined sides meet round an unjoined one.
fn panel(edges: Edges, corners: bool) -> Picture {
    let mut picture = plate(101);
    picture.each(|x, y, color| {
        let across = |at: usize| at.min(15 - at);
        let (from_side, from_end) = (across(x), across(y));
        let on_side = edges.left_right && from_side < FRAME;
        let on_end = edges.top_bottom && from_end < FRAME;
        let in_corner = corners && from_side < FRAME && from_end < FRAME;
        if !(on_side || on_end || in_corner) {
            return color;
        }
        // An inner corner is the corner of a frame round a block beside: its line touches that
        // block at one point only.
        let depth = match (on_side, on_end) {
            (true, true) => from_side.min(from_end),
            (true, false) => from_side,
            (false, true) => from_end,
            (false, false) => from_side.max(from_end),
        };
        let lit = x + y < 15;
        match depth {
            0 => STEEL[1],
            _ if lit => COPPER[3],
            _ => COPPER[2],
        }
    });
    if edges.left_right && edges.top_bottom {
        corner_rivets(&mut picture, 1, BRASS[3], BRASS[0]);
    }
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_gauges_sit_where_the_model_shows_them() {
        let model = include_str!("../../../assets/models/block/oil_engine_controller.json");
        let (top, bottom) = GAUGE_ROWS;
        for left in [OIL_GAUGE_AT, WATER_GAUGE_AT] {
            let face = format!("\"uv\": [{left}, {top}, {}, {bottom}]", left + 4);
            assert!(model.contains(&face), "a gauge face {face}");
        }
        let (x, y) = LAMP_AT;
        let lamp = format!("\"uv\": [{x}, {y}, {}, {}]", x + 2, y + 2);
        assert!(model.contains(&lamp), "the lamp's face {lamp}");
    }

    #[test]
    fn a_panel_tile_alone_is_framed_all_round() {
        let alone = panel(Edges::ALL, false);
        let joined = panel(Edges::NONE, false);
        assert_eq!(alone.get(7, 0), STEEL[1], "its frame's line at the top");
        assert_eq!(alone.get(0, 7), STEEL[1], "and at the left");
        assert_ne!(joined.get(7, 0), STEEL[1], "joined all round, no frame");
        let corner = panel(Edges::NONE, true);
        assert_eq!(
            corner.get(0, 0),
            STEEL[1],
            "an inner corner's line touches its corner"
        );
        assert_eq!(corner.get(2, 0), COPPER[3], "and goes no further");
        assert_ne!(corner.get(7, 0), STEEL[1], "no edge");
    }
}
