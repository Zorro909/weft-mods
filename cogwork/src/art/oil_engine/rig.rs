//! The oil engine's rig's picture: dark steel frame, chimney and crankshaft, iron cylinders banded
//! with copper, bright piston rods, brass connecting rods on copper crank webs, and a flywheel
//! whose spokes show it turning.

use super::PLATE;
use crate::art::{BRASS, CLEAR, COPPER, IRON, Picture, Rgba, STEEL, Unwrap, noise};

/// The rig's texture, and where its model's unwrap puts each box.
const RIG_SIZE: (usize, usize) = (128, 64);
const PLINTH: Unwrap = box_at(0, 0, (36, 1, 12));
const CYLINDER: Unwrap = box_at(0, 13, (8, 7, 8));
const POST: Unwrap = box_at(32, 13, (3, 18, 4));
const CHIMNEY: Unwrap = box_at(46, 13, (4, 16, 4));
const CHIMNEY_CAP: Unwrap = box_at(62, 13, (6, 2, 6));
const PISTON_ROD: Unwrap = box_at(86, 13, (2, 6, 2));
const CROSSHEAD: Unwrap = box_at(94, 13, (4, 2, 4));
const CONNECTING_ROD: Unwrap = box_at(110, 13, (2, 5, 2));
/// A rod's big end, round its crank pin.
const BIG_END: Unwrap = box_at(86, 23, (4, 2, 2));
/// The crankshaft: its end through the left post, and the lengths between the webs and to
/// the flywheel.
const SHAFT_END: Unwrap = box_at(0, 37, (6, 2, 2));
const SHAFT: Unwrap = box_at(16, 37, (8, 2, 2));
/// A crank web, one either side of each pin.
const CRANK_WEB: Unwrap = box_at(68, 37, (1, 4, 2));
const FLYWHEEL_HUB: Unwrap = box_at(52, 45, (4, 2, 2));
/// The flywheel: side by side, the boxes that make its stepped round, each with the height and
/// across-place of its bottom corner in the model's texels (the axle at 18 and 0).
const FLYWHEEL: [(Unwrap, i32, i32); 7] = [
    (box_at(0, 45, (2, 12, 4)), 12, -2),
    (box_at(12, 45, (2, 10, 2)), 13, 2),
    (box_at(20, 45, (2, 10, 2)), 13, -4),
    (box_at(28, 45, (2, 8, 1)), 14, 4),
    (box_at(34, 45, (2, 8, 1)), 14, -5),
    (box_at(40, 45, (2, 4, 1)), 16, 5),
    (box_at(46, 45, (2, 4, 1)), 16, -6),
];
/// Where the flywheel's axle is, and the radius of the brass rim inside its edge, in texels.
const AXLE: (f32, f32) = (18.0, 0.0);
const RIM: f32 = 4.6;
/// Every box of the rig but the flywheel's, to check against its model.
const RIG_BOXES: [Unwrap; 13] = [
    PLINTH,
    CYLINDER,
    POST,
    CHIMNEY,
    CHIMNEY_CAP,
    PISTON_ROD,
    CROSSHEAD,
    CONNECTING_ROD,
    BIG_END,
    SHAFT_END,
    SHAFT,
    CRANK_WEB,
    FLYWHEEL_HUB,
];

const fn box_at(u: usize, v: usize, size: (usize, usize, usize)) -> Unwrap {
    Unwrap { at: (u, v), size }
}

/// The rig: dark steel frame and chimney, iron cylinders banded with copper, bright rods, a
/// copper crank, a steel flywheel rimmed in brass.
pub fn picture() -> Picture {
    let (width, height) = RIG_SIZE;
    let mut picture = Picture::sized(width, height, CLEAR);
    let steel = |x, y| STEEL[1 + noise(x, y, 103) as usize % 2];
    let iron = |x, y| PLATE[noise(x, y, 107) as usize % 3];
    let edged =
        |inside: Rgba, edge: Rgba| move |_, _, on_edge: bool| if on_edge { edge } else { inside };
    PLINTH.paint(&mut picture, steel);
    POST.paint(&mut picture, |x, y| {
        STEEL[3 + noise(x, y, 131) as usize % 2]
    });
    POST.paint_ends(&mut picture, edged(STEEL[3], STEEL[1]));
    CHIMNEY.paint(&mut picture, |x, y| STEEL[noise(x, y, 109) as usize % 2]);
    CHIMNEY.paint_caps(
        &mut picture,
        |_, _, edge| if edge { STEEL[1] } else { STEEL[0] },
    );
    CHIMNEY_CAP.paint(&mut picture, |x, y| {
        COPPER[1 + noise(x, y, 113) as usize % 2]
    });
    CHIMNEY_CAP.paint_caps(
        &mut picture,
        |_, _, edge| if edge { COPPER[3] } else { STEEL[0] },
    );
    cylinder(&mut picture, iron);
    PISTON_ROD.paint(&mut picture, |_, _| IRON[3]);
    CROSSHEAD.paint(&mut picture, |_, _| COPPER[3]);
    CROSSHEAD.paint_caps(&mut picture, edged(COPPER[4], COPPER[2]));
    CONNECTING_ROD.paint(&mut picture, |_, _| BRASS[2]);
    let shaft = |x: usize, _| {
        if x.is_multiple_of(4) {
            STEEL[2]
        } else {
            STEEL[4]
        }
    };
    SHAFT_END.paint(&mut picture, shaft);
    SHAFT.paint(&mut picture, shaft);
    BIG_END.paint(&mut picture, |_, _| BRASS[1]);
    BIG_END.paint_ends(&mut picture, edged(BRASS[3], BRASS[0]));
    CRANK_WEB.paint(&mut picture, |_, _| COPPER[2]);
    CRANK_WEB.paint_ends(&mut picture, edged(COPPER[3], COPPER[1]));
    flywheel(&mut picture);
    FLYWHEEL_HUB.paint(&mut picture, |_, _| BRASS[1]);
    picture
}

/// The flywheel: a brass rim round dark steel crossed by two spokes, so its turning shows. Each
/// box's sides are painted by where each texel lies on the wheel.
fn flywheel(picture: &mut Picture) {
    for (unwrap, bottom, side) in FLYWHEEL {
        let ((u, v), (w, h, d)) = (unwrap.at, unwrap.size);
        unwrap.paint(picture, |_, _| BRASS[1]);
        let faces = [u, u + d + w];
        for left in faces {
            for row in 0..h {
                for column in 0..d {
                    let y = (bottom + (h - 1 - row) as i32) as f32 + 0.5;
                    let z = (side + column as i32) as f32 + 0.5;
                    picture.set(left + column, v + d + row, wheel_at(y - AXLE.0, z - AXLE.1));
                }
            }
        }
    }
}

/// The flywheel's face `up` and `across` texels from its axle.
fn wheel_at(up: f32, across: f32) -> Rgba {
    let radius = (up * up + across * across).sqrt();
    match radius {
        r if r < 1.5 => BRASS[3],
        r if r > RIM + 1.0 => BRASS[1],
        r if r > RIM => BRASS[2],
        _ if up.abs() < 1.0 || across.abs() < 1.0 => IRON[2],
        _ => STEEL[0],
    }
}

/// The cylinders: iron, a copper band round their top and foot, dark where the rod goes in.
fn cylinder(picture: &mut Picture, iron: impl Fn(usize, usize) -> Rgba) {
    let (_, v) = CYLINDER.at;
    let (_, h, d) = CYLINDER.size;
    let sides = v + d;
    CYLINDER.paint(picture, |x, y| {
        let row = y.wrapping_sub(sides);
        match row {
            0 | 1 => COPPER[3],
            row if row + 2 == h => COPPER[2],
            row if row + 1 == h => COPPER[1],
            _ => iron(x, y),
        }
    });
    CYLINDER.paint_caps(picture, |x, y, edge| match edge {
        true => COPPER[2],
        false if noise(x, y, 127).is_multiple_of(3) => STEEL[1],
        false => STEEL[0],
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rig_texture_matches_its_model() {
        let model = include_str!("../../../assets/models/oil_engine_rig.geo.json");
        let flywheel = FLYWHEEL.map(|(unwrap, ..)| unwrap);
        for unwrap in RIG_BOXES.into_iter().chain(flywheel) {
            let (w, h, d) = unwrap.size;
            let (u, v) = unwrap.at;
            let cube = format!("\"size\": [{w}, {h}, {d}], \"uv\": [{u}, {v}]");
            assert!(model.contains(&cube), "a box {cube}");
        }
        let (width, height) = RIG_SIZE;
        assert!(
            model.contains(&format!("\"texture_width\": {width}")),
            "as wide"
        );
        assert!(
            model.contains(&format!("\"texture_height\": {height}")),
            "as high"
        );
    }
}
