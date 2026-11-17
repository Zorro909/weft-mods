//! Pictures of the steel age's ore work: muddy water, still and in its bucket; the flotation
//! cell's pulp, its bubbling froth and its rotor; the clarifier's settling water and its rake;
//! and the core drill's crank and rod. The moving parts' textures follow their Blockbench models'
//! boxes (`Unwrap`).

use super::quarry::PLANK;
use super::steel_items::bucket_of;
use super::{CLEAR, IRON, Picture, Rgba, STEEL, Unwrap, noise};

/// Muddy water: a murky grey-brown, dark to its sheen.
const MUDDY: [Rgba; 5] = [
    [0x2e, 0x26, 0x1a, 255],
    [0x44, 0x38, 0x26, 255],
    [0x58, 0x4a, 0x34, 255],
    [0x6e, 0x60, 0x48, 255],
    [0x8a, 0x7c, 0x62, 255],
];
/// How opaque muddy water is: murkier than water.
const MUDDY_ALPHA: u8 = 0xd8;
/// Froth: pale grey bubbles, specked dark with the ore they carry.
const FROTH: [Rgba; 4] = [
    [0x5a, 0x5c, 0x60, 255],
    [0xa4, 0xa8, 0xac, 255],
    [0xd0, 0xd4, 0xd6, 255],
    [0xf2, 0xf4, 0xf4, 255],
];
/// Clear water as it settles out over the mud: blue, dark to light.
const CLEAR_WATER: [Rgba; 3] = [
    [0x2a, 0x4c, 0x8c, 255],
    [0x36, 0x60, 0xa8, 255],
    [0x4c, 0x7c, 0xc4, 255],
];
/// Frames of muddy water's swirl and of the froth's bubbling, each shown for its `.anim`'s
/// frametime.
const FRAMES: usize = 8;

/// The flotation rotor's boxes: shaft, drive cap, blades and the collar above the froth.
const ROTOR_SHAFT: Unwrap = Unwrap::new((0, 0), (2, 15, 2));
const ROTOR_CAP: Unwrap = Unwrap::new((8, 0), (6, 2, 6));
const ROTOR_BLADE: Unwrap = Unwrap::new((8, 8), (8, 3, 1));
const ROTOR_COLLAR: Unwrap = Unwrap::new((0, 17), (4, 1, 4));
/// The clarifier rake's boxes: the arm across the tank, the drive head and a tine.
const RAKE_ARM: Unwrap = Unwrap::new((0, 0), (14, 1, 2));
const RAKE_HEAD: Unwrap = Unwrap::new((0, 3), (4, 2, 4));
const RAKE_TINE: Unwrap = Unwrap::new((16, 3), (1, 4, 1));
/// The core drill's turning boxes: the rod, the crank's shaft, arm and handle, and a collar on
/// the rod.
const ROD: Unwrap = Unwrap::new((0, 0), (2, 16, 2));
const CRANK_SHAFT: Unwrap = Unwrap::new((8, 0), (2, 2, 2));
const CRANK_ARM: Unwrap = Unwrap::new((8, 4), (6, 1, 2));
const CRANK_HANDLE: Unwrap = Unwrap::new((8, 7), (1, 3, 1));
const ROD_COLLAR: Unwrap = Unwrap::new((12, 7), (4, 1, 4));

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/muddy_water_still", muddy_still()),
        ("item/muddy_water_bucket", bucket_of(&MUDDY)),
        ("block/flotation_pulp", pulp()),
        ("block/flotation_froth", froth()),
        ("block/clarifier_water", settling()),
        ("entity/flotation_rotor", rotor()),
        ("entity/clarifier_rake", rake()),
        ("entity/core_drill_rod", rod()),
    ]
}

/// Muddy water: murky and a little see-through, a lighter swirl drifting slowly across it.
fn muddy_still() -> Picture {
    let frames: Vec<Picture> = (0..FRAMES)
        .map(|frame| {
            let mut picture = Picture::surface(&MUDDY[1..3], 709);
            picture.each(|x, y, color| {
                let swirl = (2 * x + y + 16 - 2 * frame % 16) % 16;
                let [r, g, b, _] = match (swirl, noise(x, y, 719) % 4) {
                    (0, 0) => MUDDY[4],
                    (0 | 1, _) => MUDDY[3],
                    _ => color,
                };
                [r, g, b, MUDDY_ALPHA]
            });
            picture
        })
        .collect();
    Picture::stacked(&frames)
}

/// The pulp in an idle cell: grey slurry, still.
fn pulp() -> Picture {
    Picture::surface(&[FROTH[0], MUDDY[2], MUDDY[3]], 727)
}

/// Froth: bubbles swelling and bursting from frame to frame, specked with ore.
fn froth() -> Picture {
    let frames: Vec<Picture> = (0..FRAMES).map(froth_frame).collect();
    Picture::stacked(&frames)
}

/// Bubbles on the froth, and the frame each bursts on its own.
const BUBBLES: usize = 14;

/// One frame of the froth: bubbles scattered over a pale foam, each swelling from frame to frame,
/// white in a grey rim, until it bursts and leaves a dark speck of ore. They wrap
/// round the edges, so the froth tiles.
fn froth_frame(frame: usize) -> Picture {
    let mut picture = Picture::filled(FROTH[2]);
    picture.each(|x, y, _| {
        let speck = noise(x, y, 743).is_multiple_of(7);
        let mut color = if speck { FROTH[1] } else { FROTH[2] };
        for bubble in 0..BUBBLES {
            let at = |axis: usize| noise(bubble, axis, 733) as usize % 16;
            let age = (noise(bubble, 2, 733) as usize + frame) % FRAMES;
            let (dx, dy) = (wrapped(x, at(0)), wrapped(y, at(1)));
            let distance = dx * dx + dy * dy;
            let radius = 1 + age / 3;
            color = match age {
                0 if distance == 0 => FROTH[0],
                0 => continue,
                _ if distance > radius * radius + radius => continue,
                _ if distance + 1 >= radius * radius => FROTH[1],
                _ => FROTH[3],
            };
        }
        color
    });
    picture
}

/// How far apart `a` and `b` are on a 16 texel picture that wraps round.
fn wrapped(a: usize, b: usize) -> usize {
    let apart = a.abs_diff(b);
    apart.min(16 - apart)
}

/// The clarifier's water: mud settling dark in the middle, clear blue toward the rim.
fn settling() -> Picture {
    let mut picture = Picture::filled(CLEAR_WATER[1]);
    picture.each(|x, y, _| {
        let (dx, dy) = (x.abs_diff(8), y.abs_diff(8));
        let from_middle = dx * dx + dy * dy;
        let speck = noise(x, y, 739) % 5;
        match from_middle {
            0..=9 => MUDDY[(speck % 2) as usize + 1],
            10..=25 if speck < 3 => MUDDY[3],
            10..=25 => CLEAR_WATER[0],
            _ if speck == 0 => CLEAR_WATER[2],
            _ => CLEAR_WATER[1],
        }
    });
    picture
}

/// The flotation rotor: a steel shaft, a riveted drive cap, iron blades and a collar.
fn rotor() -> Picture {
    let mut picture = Picture::sized(32, 32, CLEAR);
    ROTOR_SHAFT.paint(
        &mut picture,
        |x, _| if x % 2 == 0 { STEEL[4] } else { STEEL[3] },
    );
    ROTOR_CAP.paint(&mut picture, |x, y| {
        if (x + y) % 3 == 0 { IRON[3] } else { STEEL[2] }
    });
    ROTOR_BLADE.paint(&mut picture, |_, y| if y == 8 { IRON[3] } else { IRON[1] });
    ROTOR_COLLAR.paint(&mut picture, |_, _| STEEL[1]);
    picture
}

/// The clarifier rake: an iron arm and drive head, dark tines.
fn rake() -> Picture {
    let mut picture = Picture::sized(32, 16, CLEAR);
    RAKE_ARM.paint(
        &mut picture,
        |x, _| if x % 4 == 0 { IRON[1] } else { IRON[2] },
    );
    RAKE_HEAD.paint(&mut picture, |x, y| {
        if (x + y) % 4 == 0 { IRON[3] } else { STEEL[2] }
    });
    RAKE_TINE.paint(&mut picture, |_, _| IRON[0]);
    picture
}

/// The core drill's rod in steel, an iron crank with a wooden handle, and a collar.
fn rod() -> Picture {
    let mut picture = Picture::sized(32, 32, CLEAR);
    ROD.paint(
        &mut picture,
        |_, y| if y % 4 == 0 { STEEL[2] } else { STEEL[4] },
    );
    CRANK_SHAFT.paint(&mut picture, |_, _| IRON[1]);
    CRANK_ARM.paint(&mut picture, |_, y| if y == 4 { IRON[3] } else { IRON[2] });
    CRANK_HANDLE.paint(&mut picture, |x, _| PLANK[1 + x % 2]);
    ROD_COLLAR.paint(&mut picture, |_, _| STEEL[1]);
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn froth_bubbles_burst_and_swell_from_frame_to_frame() {
        let (first, next) = (froth_frame(0), froth_frame(1));
        let differs = (0..16).any(|y| (0..16).any(|x| first.get(x, y) != next.get(x, y)));
        assert!(differs, "it moves");
        let specked = (0..FRAMES)
            .map(froth_frame)
            .any(|frame| (0..16).any(|y| (0..16).any(|x| frame.get(x, y) == FROTH[0])));
        assert!(specked, "some burst, specked with ore");
    }

    #[test]
    fn mud_settles_in_the_middle_and_clear_water_at_the_rim() {
        let water = settling();
        assert!(MUDDY.contains(&water.get(8, 8)), "mud in the middle");
        assert!(CLEAR_WATER.contains(&water.get(0, 0)), "clear at the rim");
    }

    #[test]
    fn the_moving_parts_fill_their_unwraps() {
        assert_ne!(rotor().get(15, 0)[3], 0, "the cap's top");
        assert_ne!(rake().get(2, 0)[3], 0, "the arm's top");
        assert_ne!(rod().get(1, 17)[3], 0, "the rod's last row");
    }
}
