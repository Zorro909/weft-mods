//! Pictures of the water wheel and the wooden axle it turns: the wheel's planks, spokes and
//! paddles, its icon, and the axle's rod and bearing. The wood is the Timber Quarry's (`quarry`).

use super::quarry::{BARK, PLANK, plank};
use super::{CLEAR, IRON, Picture, Rgba, Unwrap, noise};

/// The wheel's texture, and where its model's unwrap puts each box.
const WHEEL_SIZE: (usize, usize) = (64, 48);
const SPOKE: Unwrap = Unwrap {
    at: (0, 0),
    size: (2, 36, 2),
};
const HUB: Unwrap = Unwrap {
    at: (8, 0),
    size: (8, 8, 6),
};
const PADDLE: Unwrap = Unwrap {
    at: (36, 0),
    size: (2, 6, 8),
};
const RIM: Unwrap = Unwrap {
    at: (8, 14),
    size: (14, 2, 4),
};

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("entity/water_wheel", wheel()),
        ("item/water_wheel", icon()),
        ("block/wooden_axle", axle()),
        ("block/wooden_axle_bearing", bearing()),
    ]
}

/// Weathered wood, a shade darker than planks: what the water keeps wet.
fn wet(x: usize, y: usize) -> Rgba {
    [PLANK[0], PLANK[1], BARK[3]][(noise(x, y, 233) % 3) as usize]
}

/// The wheel: plank spokes and rim, wet paddles with light edges, a dark hub bound in iron.
fn wheel() -> Picture {
    let (width, height) = WHEEL_SIZE;
    let mut picture = Picture::sized(width, height, CLEAR);
    SPOKE.paint(&mut picture, plank);
    RIM.paint(&mut picture, plank);
    PADDLE.paint(&mut picture, wet);
    PADDLE.paint_ends(
        &mut picture,
        |x, y, edge| {
            if edge { PLANK[3] } else { wet(x, y) }
        },
    );
    HUB.paint(&mut picture, |x, y| {
        BARK[1 + (noise(x, y, 239) % 2) as usize]
    });
    HUB.paint_ends(
        &mut picture,
        |_, _, edge| if edge { IRON[1] } else { BARK[0] },
    );
    picture
}

/// The wheel in the hand: an eight-spoked wheel with paddles round its rim.
fn icon() -> Picture {
    const ROWS: [&str; 16] = [
        "......pppp......",
        "...p..rrrr..p...",
        "..pprr.ss.rrpp..",
        "...r.s.ss.s.r...",
        ".pr...sss.s..rp.",
        ".pr.s..ss..s.rp.",
        "pr...s.ss.s...rp",
        "prssssshhsssssrp",
        "prssssshhsssssrp",
        "pr...s.ss.s...rp",
        ".pr.s..ss..s.rp.",
        ".pr.s..sss...rp.",
        "...r.s.ss.s.r...",
        "..pprr.ss.rrpp..",
        "...p..rrrr..p...",
        "......pppp......",
    ];
    let key = [
        ('p', PLANK[0]),
        ('r', PLANK[2]),
        ('s', PLANK[1]),
        ('h', IRON[1]),
    ];
    Picture::drawn(&ROWS, &key)
}

/// The axle's rod: smooth wood with a light streak along it.
fn axle() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let streak = noise(x / 4, y / 4, 241) % 3;
        PLANK[1 + streak as usize]
    });
    picture
}

/// The bearing: a block of dark wood with an iron corner at each corner of its faces.
fn bearing() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let corner = [5, 10].contains(&x) && [5, 10].contains(&y);
        let edge = [5, 10].contains(&x) || [5, 10].contains(&y);
        match (corner, edge) {
            (true, _) => IRON[2],
            (_, true) => BARK[0],
            _ => BARK[2 + (noise(x, y, 251) % 2) as usize],
        }
    });
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheel_texture_matches_its_model() {
        let model = include_str!("../../assets/models/water_wheel.geo.json");
        let (width, height) = WHEEL_SIZE;
        assert!(
            model.contains(&format!("\"texture_width\": {width}")),
            "as wide"
        );
        assert!(
            model.contains(&format!("\"texture_height\": {height}")),
            "as high"
        );
        for part in [SPOKE, HUB, PADDLE, RIM] {
            let (w, h, d) = part.size;
            let (u, v) = part.at;
            let cube = format!("\"size\": [{w}, {h}, {d}], \"uv\": [{u}, {v}]");
            assert!(model.contains(&cube), "a box {cube}");
        }
    }
}
