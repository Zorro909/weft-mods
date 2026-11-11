//! Pictures of the timber liquid tube: its glass arms framed in wood, its wooden pump body with a
//! window on each side, and the paddle wheel turning in it. The wood is the timber extraction
//! tube's (`tube_timber.png`).

use super::{BRASS, CLEAR, GLASS_METAL, GLINT, Picture, Rgba, noise};

/// The timber tube's wood, dark to light.
const WOOD: [Rgba; 6] = [
    [0x46, 0x2e, 0x18, 255],
    [0x5c, 0x3e, 0x20, 255],
    [0x76, 0x54, 0x2e, 255],
    [0x88, 0x64, 0x38, 255],
    [0xa4, 0x7c, 0x48, 255],
    [0xba, 0x90, 0x58, 255],
];
/// Where the arms' wooden edges lie: the arms are 4 to 12.
const ARM_EDGES: [usize; 2] = [4, 11];
/// Where the body's frame lies, two texels wide: the body is 3 to 13, its window inside the
/// frame.
const FRAME: [usize; 2] = [3, 12];
const FRAME_WIDTH: usize = 2;
/// The wheel's texture: its size, and where the model's unwrap puts each box (Blockbench's
/// Bedrock layout of a box `w` wide, `h` high and `d` deep: a row of its top and bottom `d` high,
/// then a row of its four sides `h` high).
const WHEEL_SIZE: (usize, usize) = (64, 32);
const UPRIGHT_PADDLE: Unwrap = Unwrap {
    at: (0, 0),
    size: (2, 8, 8),
};
const LEVEL_PADDLE: Unwrap = Unwrap {
    at: (0, 16),
    size: (8, 2, 8),
};
const AXLE: Unwrap = Unwrap {
    at: (32, 0),
    size: (3, 3, 9),
};

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/timber_liquid_tube", arm()),
        ("block/timber_liquid_tube_window", body()),
        ("entity/timber_pump_wheel", wheel()),
    ]
}

/// The arms: clear glass between two wooden edges, a brass band where each meets the next
/// tube, and a streak of sheen across each.
fn arm() -> Picture {
    let across = |at: usize| (ARM_EDGES[0]..=ARM_EDGES[1]).contains(&at);
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let along_arm = across(x) || across(y);
        let at_end = [0, 15].contains(&x) || [0, 15].contains(&y);
        match (along_arm, at_end) {
            (false, _) => CLEAR,
            (true, true) => BRASS[2],
            (true, false) if ARM_EDGES.contains(&x) || ARM_EDGES.contains(&y) => WOOD[1],
            _ => CLEAR,
        }
    });
    for (x, y) in [
        (1, 8),
        (2, 7),
        (13, 8),
        (14, 7),
        (7, 2),
        (8, 1),
        (7, 14),
        (8, 13),
    ] {
        picture.set(x, y, GLASS_METAL);
    }
    for (x, y) in [(2, 6), (14, 6), (9, 1), (9, 13)] {
        picture.set(x, y, GLINT);
    }
    picture
}

/// The pump body's sides: planks all round (the flanges round the arms show them too), a frame
/// lit on its top and left, brass at its corners, and a window of clear glass in it.
fn body() -> Picture {
    let [low, high] = FRAME;
    let on_bar = |at: usize| at < low + FRAME_WIDTH || at > high - FRAME_WIDTH;
    let mut picture = Picture::surface(&WOOD[2..5], 61);
    picture.each(|x, y, color| {
        let in_frame = (low..=high).contains(&x) && (low..=high).contains(&y);
        let on_frame = in_frame && (on_bar(x) || on_bar(y));
        match (in_frame, on_frame) {
            (true, true) if on_bar(x) && on_bar(y) => BRASS[3],
            (true, true) if x == low || y == low => WOOD[5],
            (true, true) if x == high || y == high => WOOD[1],
            (true, true) => WOOD[3],
            (true, false) => CLEAR,
            // A seam between planks every four rows.
            (false, _) if y % 4 == 3 => WOOD[1],
            (false, _) => color,
        }
    });
    // A rivet in each brass corner.
    for (x, y) in [(low, low), (high, low), (low, high), (high, high)] {
        picture.set(x, y, BRASS[1]);
    }
    for (x, y) in [(6, 9), (7, 8), (8, 7)] {
        picture.set(x, y, GLASS_METAL);
    }
    picture.set(9, 6, GLINT);
    picture
}

/// The paddle wheel: two crossed paddles of wood, their ends lighter where they show through the
/// windows, on a brass axle.
fn wheel() -> Picture {
    let (width, height) = WHEEL_SIZE;
    let mut picture = Picture::sized(width, height, CLEAR);
    for paddle in [UPRIGHT_PADDLE, LEVEL_PADDLE] {
        paddle.paint(&mut picture, |x, y| WOOD[2 + noise(x, y, 67) as usize % 2]);
        paddle.paint_ends(
            &mut picture,
            |_, _, edge| if edge { WOOD[2] } else { WOOD[5] },
        );
    }
    AXLE.paint(&mut picture, |x, y| BRASS[1 + noise(x, y, 71) as usize % 2]);
    AXLE.paint_ends(
        &mut picture,
        |_, _, edge| if edge { BRASS[2] } else { BRASS[4] },
    );
    picture
}

/// Where a box of the wheel model lies in its texture: the top left corner of its unwrap, and the
/// box's width, height and depth in texels.
#[derive(Clone, Copy)]
struct Unwrap {
    at: (usize, usize),
    size: (usize, usize, usize),
}

impl Unwrap {
    /// Paints every face of the box with `color` of where a pixel is.
    fn paint(self, picture: &mut Picture, color: impl Fn(usize, usize) -> Rgba) {
        let ((u, v), (w, h, d)) = (self.at, self.size);
        let top_row = [(u + d, v, w, d), (u + d + w, v, w, d)];
        let side_row = [(u, v + d, 2 * (d + w), h)];
        for (x0, y0, across, down) in top_row.into_iter().chain(side_row) {
            for y in y0..y0 + down {
                for x in x0..x0 + across {
                    picture.set(x, y, color(x, y));
                }
            }
        }
    }

    /// Paints the box's two ends along its depth, the faces the windows show, with `color` of
    /// where a pixel is and whether it is on the face's edge.
    fn paint_ends(self, picture: &mut Picture, color: impl Fn(usize, usize, bool) -> Rgba) {
        let ((u, v), (w, h, d)) = (self.at, self.size);
        for x0 in [u + d, u + 2 * d + w] {
            let y0 = v + d;
            for y in y0..y0 + h {
                for x in x0..x0 + w {
                    let edge = x == x0 || x == x0 + w - 1 || y == y0 || y == y0 + h - 1;
                    picture.set(x, y, color(x, y, edge));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheel_texture_matches_its_model() {
        let model = include_str!("../../assets/models/timber_pump_wheel.geo.json");
        for unwrap in [UPRIGHT_PADDLE, LEVEL_PADDLE, AXLE] {
            let (w, h, d) = unwrap.size;
            let (u, v) = unwrap.at;
            let cube = format!("\"size\": [{w}, {h}, {d}], \"uv\": [{u}, {v}]");
            assert!(model.contains(&cube), "a box {cube}");
        }
        let (width, height) = WHEEL_SIZE;
        assert!(
            model.contains(&format!("\"texture_width\": {width}")),
            "as wide"
        );
        assert!(
            model.contains(&format!("\"texture_height\": {height}")),
            "as high"
        );
    }

    #[test]
    fn the_window_is_clear_inside_its_frame_only() {
        let body = body();
        let [low, high] = FRAME;
        for y in 0..16 {
            for x in 0..16 {
                let window = low + FRAME_WIDTH..=high - FRAME_WIDTH;
                let inside = window.contains(&x) && window.contains(&y);
                assert!(inside || body.get(x, y) != CLEAR, "wood at {x}, {y}");
            }
        }
        assert_eq!(body.get(5, 5), CLEAR, "glass you see through");
    }
}
