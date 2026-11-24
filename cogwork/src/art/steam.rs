//! Pictures of the steam works: steam itself, a white haze drifting up; the firebox's iron door
//! with its grate, the fire glowing through it while it burns; the boiler's riveted plate, its
//! brass bands and its gauge; and the moving parts of the steam engine and the trip hammer.

use super::steel::{FIRE_BRICK, FIRE_FRAMES, FIRE_MORTAR, bricks, fire, flickering, glow_of};
use super::{BRASS, CLEAR, IRON, Picture, Rgba, STEEL, Unwrap, noise};

/// Steam, from its thin haze to its brightest wisps, see-through.
const STEAM: [Rgba; 3] = [
    [0xe2, 0xe8, 0xee, 0x70],
    [0xee, 0xf2, 0xf6, 0x98],
    [0xfa, 0xfc, 0xff, 0xc0],
];
/// Frames of steam's drift, one every `steam_still.anim`'s frametime.
const STEAM_FRAMES: usize = 8;
/// The grate's slits, before they are filled with fire or embers.
const SLIT: Rgba = [1, 1, 1, 255];
const EMBERS: Rgba = [0x22, 0x16, 0x10, 255];
/// The gauge's white face and red mark.
const DIAL: Rgba = [0xec, 0xe8, 0xdc, 255];
const MARK: Rgba = [0xc0, 0x30, 0x20, 255];

/// The steam engine rig's texture and where its model's unwrap puts each box.
const RIG_SIZE: (usize, usize) = (64, 32);
const SPOKE_UPRIGHT: Unwrap = Unwrap::new((0, 0), (2, 10, 4));
const SPOKE_ACROSS: Unwrap = Unwrap::new((12, 0), (2, 4, 10));
const STUB: Unwrap = Unwrap::new((36, 0), (3, 1, 1));
const PIN: Unwrap = Unwrap::new((36, 2), (6, 1, 1));
const ROD: Unwrap = Unwrap::new((36, 4), (1, 1, 6));
const CROSSHEAD: Unwrap = Unwrap::new((50, 0), (3, 2, 2));
const CONROD: Unwrap = Unwrap::new((0, 14), (2, 1, 4));
/// The trip hammer's texture and its boxes.
const HAMMER_SIZE: (usize, usize) = (64, 32);
const HELVE: Unwrap = Unwrap::new((0, 0), (2, 3, 13));
const HEAD: Unwrap = Unwrap::new((30, 0), (5, 4, 4));
const HUB: Unwrap = Unwrap::new((30, 8), (3, 4, 3));

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/steam_still", steam_still()),
        ("block/firebox_front", firebox_door(None)),
        (
            "block/firebox_front_lit",
            flickering(|frame| firebox_door(Some(frame))),
        ),
        (
            "block/firebox_front_lit_e",
            flickering(|frame| glow_of(&firebox_door(Some(frame)))),
        ),
        ("block/boiler_plate", boiler_plate()),
        ("block/boiler_band", boiler_band()),
        ("block/boiler_gauge", boiler_gauge()),
        ("entity/steam_engine_rig", rig()),
        ("entity/trip_hammer_hammer", hammer()),
    ]
}

/// Steam: a thin white haze with brighter wisps drifting up through it.
fn steam_still() -> Picture {
    let frames: Vec<Picture> = (0..STEAM_FRAMES)
        .map(|frame| {
            let mut picture = Picture::filled(STEAM[0]);
            picture.each(|x, y, color| {
                let risen = (y + 2 * frame) % 16;
                let wisp = (x + noise(0, risen / 4, 503) as usize) % 8;
                match (wisp, noise(x, risen, 509) % 4) {
                    (0, 0) => STEAM[2],
                    (0 | 1, _) => STEAM[1],
                    _ => color,
                }
            });
            picture
        })
        .collect();
    Picture::stacked(&frames)
}

/// The firebox's front: fire bricks round an iron door with a grate, dark embers behind it when
/// cold and the fire of frame `lit` when it burns.
fn firebox_door(lit: Option<usize>) -> Picture {
    let mut picture = bricks(&FIRE_BRICK, FIRE_MORTAR, 409);
    let door = Picture::drawn(
        &[
            "dmmmmmmmmmmd",
            "mLLLLLLLLLLm",
            "mL.b.b.b.bLm",
            "mL.b.b.b.bLm",
            "mL.b.b.b.bLm",
            "mL.b.b.b.bLm",
            "mL.b.b.b.bLm",
            "mLLLLLLLLLhm",
            "dmmmmmmmmmmd",
        ],
        &[
            ('d', IRON[0]),
            ('m', IRON[1]),
            ('L', IRON[2]),
            ('h', IRON[3]),
            ('b', STEEL[3]),
            ('.', SLIT),
        ],
    );
    picture.overlay(&door, (2, 5));
    picture.each(|x, y, color| match (color == SLIT, lit) {
        (true, Some(frame)) => fire(x, y, frame),
        (true, None) => EMBERS,
        _ => color,
    });
    picture
}

/// The boiler's plate: smooth iron sheets riveted along their seams, a row of rivets over each.
fn boiler_plate() -> Picture {
    let mut picture = Picture::filled(IRON[2]);
    picture.each(|x, y, color| match (y % 8, x % 3) {
        (7, _) => IRON[1],
        (0, _) => IRON[3],
        (5, 1) => IRON[3],
        (6, 1) => IRON[1],
        _ if noise(x, y, 521).is_multiple_of(7) => IRON[1],
        _ => color,
    });
    picture
}

/// The boiler's bands: brass, lit along the top, riveted every four texels.
fn boiler_band() -> Picture {
    let mut picture = Picture::surface(&BRASS[1..4], 523);
    picture.each(|x, y, color| match (y % 4, x % 2) {
        (0, _) => BRASS[3],
        (1, 0) => BRASS[4],
        (3, _) => BRASS[0],
        _ => color,
    });
    picture
}

/// The boiler's gauge: a white dial in a brass rim with its needle and a red mark, in the middle
/// of the plate.
fn boiler_gauge() -> Picture {
    let mut picture = boiler_plate();
    let dial = Picture::drawn(
        &[".bbbb.", "bWWWRb", "bWWnWb", "bWnWWb", "bWWWWb", ".bbbb."],
        &[('b', BRASS[1]), ('W', DIAL), ('R', MARK), ('n', STEEL[0])],
    );
    picture.overlay(&dial, (5, 5));
    picture
}

/// Dark steel, flecked.
fn dark_steel(x: usize, y: usize) -> Rgba {
    STEEL[1 + (noise(x, y, 527) % 3) as usize]
}

/// Polished iron, flecked.
fn bright_iron(x: usize, y: usize) -> Rgba {
    if noise(x, y, 529).is_multiple_of(4) {
        IRON[2]
    } else {
        IRON[3]
    }
}

/// Iron, flecked darker: the flywheel's spokes.
fn spoke_iron(x: usize, y: usize) -> Rgba {
    if noise(x, y, 535).is_multiple_of(3) {
        IRON[1]
    } else {
        IRON[2]
    }
}

/// The steam engine's moving parts: a flywheel of iron spokes whose ends are brass, so that the
/// turning spokes show a brass rim from every side; polished iron rods and pins, and a brass
/// crosshead.
fn rig() -> Picture {
    let mut picture = Picture::sized(RIG_SIZE.0, RIG_SIZE.1, CLEAR);
    let rim = |_, _, _| BRASS[3];
    SPOKE_UPRIGHT.paint(&mut picture, spoke_iron);
    SPOKE_UPRIGHT.paint_caps(&mut picture, rim);
    SPOKE_ACROSS.paint(&mut picture, spoke_iron);
    SPOKE_ACROSS.paint_ends(&mut picture, rim);
    brass_ends(&mut picture);
    for part in [STUB, PIN, ROD, CONROD] {
        part.paint(&mut picture, bright_iron);
    }
    CROSSHEAD.paint(&mut picture, |x, y| {
        BRASS[2 + (noise(x, y, 531) % 2) as usize]
    });
    picture
}

/// The texel at each end of the spokes' sides, brass: the top and bottom rows of the upright
/// spoke's sides, and the first and last columns of the side faces of the one across.
fn brass_ends(picture: &mut Picture) {
    let ((u, v), (w, h, d)) = (SPOKE_UPRIGHT.at, SPOKE_UPRIGHT.size);
    for x in u..u + 2 * (d + w) {
        picture.set(x, v + d, BRASS[2]);
        picture.set(x, v + d + h - 1, BRASS[2]);
    }
    let ((u, v), (w, h, d)) = (SPOKE_ACROSS.at, SPOKE_ACROSS.size);
    for y in v + d..v + d + h {
        for x in [u, u + d - 1, u + d + w, u + 2 * d + w - 1] {
            picture.set(x, y, BRASS[2]);
        }
    }
}

/// The trip hammer's hammer: a helve of dark steel, a polished head with a darker face, on a
/// brass hub.
fn hammer() -> Picture {
    let mut picture = Picture::sized(HAMMER_SIZE.0, HAMMER_SIZE.1, CLEAR);
    HELVE.paint(&mut picture, dark_steel);
    HEAD.paint(&mut picture, bright_iron);
    HEAD.paint_caps(
        &mut picture,
        |_, _, edge| if edge { IRON[1] } else { STEEL[2] },
    );
    HUB.paint(&mut picture, |x, y| {
        BRASS[2 + (noise(x, y, 533) % 2) as usize]
    });
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_firebox_glows_only_through_its_grate() {
        let door = firebox_door(Some(0));
        let mask = glow_of(&door);
        let glowing = (0..16)
            .flat_map(|y| (0..16).map(move |x| (x, y)))
            .filter(|&(x, y)| mask.get(x, y) != [0, 0, 0, 255])
            .count();
        assert_eq!(glowing, 20, "four slits five high");
        assert_eq!(door.get(0, 0)[3], 255, "the bricks are solid");
        assert_eq!(firebox_door(None).get(4, 7), EMBERS, "embers when cold");
        assert_eq!(FIRE_FRAMES, 4, "frames as the other fires");
    }

    #[test]
    fn steam_is_see_through() {
        let steam = steam_still();
        let solid = (0..16 * STEAM_FRAMES).any(|y| (0..16).any(|x| steam.get(x, y)[3] == 255));
        assert!(!solid, "no texel of it hides what is behind");
    }

    #[test]
    fn every_box_of_the_moving_parts_is_painted() {
        let rig = rig();
        for (x, y, what) in [(5, 2, "a spoke"), (20, 12, "the other"), (40, 1, "a stub")] {
            assert_ne!(rig.get(x, y)[3], 0, "{what}");
        }
        assert_ne!(rig.get(42, 6)[3], 0, "the piston rod");
        assert_ne!(rig.get(4, 16)[3], 0, "the connecting rod");
        let hammer = hammer();
        assert_ne!(hammer.get(10, 14)[3], 0, "the helve");
        assert_ne!(hammer.get(36, 6)[3], 0, "the head");
    }
}
