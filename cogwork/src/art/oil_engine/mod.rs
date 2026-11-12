//! Pictures of the oil engine: its parts in pale riveted iron, copper and dark steel, the ports
//! drawn on its formed panel, the gauges and lamp on its controller, the panel itself (`panel`)
//! and its rig (`rig`).

mod panel;
mod rig;

use super::{BRASS, CLEAR, COPPER, IRON, OIL, Picture, Rgba, STEEL};

/// The engine's pale iron plate, its most common shade first.
const PLATE: [Rgba; 3] = [
    [0x8e, 0x90, 0x94, 255],
    [0x86, 0x88, 0x8c, 255],
    [0x98, 0x9a, 0x9e, 255],
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

pub fn all() -> Vec<(&'static str, Picture)> {
    let mut all = vec![
        ("block/oil_engine_casing", casing()),
        ("block/combustion_chamber", chamber()),
        ("block/oil_intake", on_casing(intake_port())),
        ("block/oil_intake_port", intake_port()),
        ("block/oil_engine_drive", on_casing(drive_port())),
        ("block/oil_engine_drive_port", drive_port()),
        ("block/oil_engine_controller_front", controller_front()),
        ("block/oil_engine_lamp", lamp(LAMP_OUT)),
        ("block/oil_engine_lamp_on", lamp(LAMP_LIT)),
        ("entity/oil_engine_rig", rig::picture()),
    ];
    all.extend(panel::tiles());
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
    panel::plate()
}

/// A port drawn on casing: an intake or drive before its engine is built.
fn on_casing(port: Picture) -> Picture {
    let mut picture = casing();
    picture.overlay(&port, (0, 0));
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

/// An intake: a copper flange round a dark mouth that liquid tubes join. Clear round it, to go
/// on casing or on the formed panel.
fn intake_port() -> Picture {
    let mut picture = Picture::filled(CLEAR);
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

/// A drive: a brass coupling, the shaft's own brass, round the dark socket a torque shaft goes
/// into. Clear round it, as an intake.
fn drive_port() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, color| {
        let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
        let distance = dx.abs().max(dy.abs());
        let lit = dx + dy < 0.0;
        match distance {
            d if d < 1.0 => STEEL[0],
            d if d < 2.0 => STEEL[1],
            d if d < 3.0 => STEEL[3],
            d if d < 5.0 && lit => BRASS[3],
            d if d < 5.0 => BRASS[1],
            d if d < 6.0 => BRASS[0],
            _ => color,
        }
    });
    for at in [(3, 3), (11, 3), (3, 11), (11, 11)] {
        rivet(&mut picture, at, BRASS[4], BRASS[0]);
    }
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
}
