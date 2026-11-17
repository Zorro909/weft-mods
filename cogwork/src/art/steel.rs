//! Pictures of the steel works' blocks: the coke oven's sooty red bricks, its iron door with the
//! fire glowing through its slits, its drain; the blast furnace's pale fire bricks and its tap
//! hole, white-hot while it smelts; the block of steel, and the forge anvil's iron and the hot
//! bloom on it. What glows has a glow mask beside it (`<name>_e`, Weft `docs/textures.md`), and a
//! fire flickers through `FIRE_FRAMES` frames (`<name>.anim`).

use super::{IRON, Picture, Rgba, STEEL, noise};

/// Coke oven bricks, sooty red, dark to light, and their mortar.
const OVEN_BRICK: [Rgba; 4] = [
    [0x4a, 0x22, 0x1a, 255],
    [0x5e, 0x2c, 0x22, 255],
    [0x70, 0x36, 0x28, 255],
    [0x82, 0x42, 0x30, 255],
];
const OVEN_MORTAR: Rgba = [0x2a, 0x24, 0x22, 255];
/// Fire bricks, pale buff, dark to light, and their mortar.
pub const FIRE_BRICK: [Rgba; 4] = [
    [0xb0, 0x98, 0x70, 255],
    [0xc4, 0xac, 0x80, 255],
    [0xd2, 0xbc, 0x90, 255],
    [0xde, 0xca, 0xa0, 255],
];
pub const FIRE_MORTAR: Rgba = [0x8a, 0x80, 0x74, 255];
/// Fire, from deep red to white-hot.
pub const FIRE: [Rgba; 5] = [
    [0x8a, 0x1c, 0x08, 255],
    [0xd0, 0x40, 0x0c, 255],
    [0xf0, 0x7c, 0x18, 255],
    [0xfc, 0xbc, 0x40, 255],
    [0xff, 0xf0, 0xa8, 255],
];
/// A glow mask's full glow and none.
const GLOWS: Rgba = [255, 0, 0, 255];
const DARK: Rgba = [0, 0, 0, 255];
/// Frames of a flickering fire.
pub const FIRE_FRAMES: usize = 4;

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        (
            "block/coke_oven_bricks",
            bricks(&OVEN_BRICK, OVEN_MORTAR, 401),
        ),
        ("block/coke_oven_drain", drain()),
        ("block/coke_oven_door", oven_door(None)),
        (
            "block/coke_oven_door_lit",
            flickering(|frame| oven_door(Some(frame))),
        ),
        (
            "block/coke_oven_door_lit_e",
            flickering(|frame| glow_of(&oven_door(Some(frame)))),
        ),
        ("block/fire_bricks", bricks(&FIRE_BRICK, FIRE_MORTAR, 409)),
        ("block/blast_furnace_front", tap_hole(None)),
        (
            "block/blast_furnace_front_lit",
            flickering(|frame| tap_hole(Some(frame))),
        ),
        (
            "block/blast_furnace_front_lit_e",
            flickering(|frame| glow_of(&tap_hole(Some(frame)))),
        ),
        ("block/steel_block", steel_block()),
        ("block/forge_anvil", anvil_iron()),
        ("block/steel_bloom_hot", hot_bloom()),
        ("block/steel_bloom_hot_e", glow_of(&hot_bloom())),
    ]
}

/// Bricks of `brick` laid in `mortar`: courses four texels high, each brick eight long, every
/// other course shifted half a brick, each brick its own shade with flecks.
pub fn bricks(brick: &[Rgba; 4], mortar: Rgba, salt: u32) -> Picture {
    let mut picture = Picture::filled(mortar);
    picture.each(|x, y, _| {
        let course = y / 4;
        let shift = if course.is_multiple_of(2) { 0 } else { 4 };
        let along = (x + shift) % 16;
        if y % 4 == 3 || along % 8 == 7 {
            return mortar;
        }
        let shade = 1 + (noise((x + shift) / 8, course, salt) % 3) as usize;
        let fleck = noise(x, y, salt + 1).is_multiple_of(7);
        // Light from above: a brick's top row lighter, its bottom row darker.
        match (y % 4, fleck) {
            (_, true) => brick[shade - 1],
            (0, _) => brick[(shade + 1).min(3)],
            (2, _) => brick[shade - 1],
            _ => brick[shade],
        }
    });
    picture
}

/// The coke oven's drain: its bricks round an iron plate with a dark outlet.
fn drain() -> Picture {
    let mut picture = bricks(&OVEN_BRICK, OVEN_MORTAR, 401);
    let plate = Picture::drawn(
        &[
            "dmmmmmmd", "mLLLLLLm", "mLdddd.m", "mLdooddm", "mLdooddm", "mLddddLm", "mLLLLLLm",
            "dmmmmmmd",
        ],
        &[
            ('d', IRON[0]),
            ('m', IRON[1]),
            ('L', IRON[2]),
            ('o', [0x14, 0x0c, 0x08, 255]),
        ],
    );
    picture.overlay(&plate, (4, 4));
    picture
}

/// `frames` frames of a picture `frame` draws, one under the other: a flickering fire.
pub fn flickering(frame: impl Fn(usize) -> Picture) -> Picture {
    let frames: Vec<Picture> = (0..FIRE_FRAMES).map(frame).collect();
    Picture::stacked(&frames)
}

/// The fire's color at a texel of frame `frame`: hotter low down, flickering.
pub fn fire(x: usize, y: usize, frame: usize) -> Rgba {
    let flicker = (noise(x, y + frame * 16, 419) % 3) as usize;
    let heat = (16 - y.min(15)) / 5;
    FIRE[(heat + flicker).clamp(1, 4)]
}

/// The coke oven's door: an iron door in an iron frame with five slits, dark when cold and
/// glowing with the fire of frame `lit`.
fn oven_door(lit: Option<usize>) -> Picture {
    let mut picture = bricks(&OVEN_BRICK, OVEN_MORTAR, 401);
    let door = Picture::drawn(
        &[
            "ffffffffffff",
            "fDDDDDDDDDDf",
            "fDrDDDDDDrDf",
            "fDDDDDDDDDDf",
            "fDsDsDsDsDDf",
            "fDsDsDsDsDDf",
            "fDsDsDsDsDDf",
            "fDDDDDDDDDhf",
            "fDDDDDDDDDhf",
            "fDsDsDsDsDDf",
            "fDsDsDsDsDDf",
            "fDsDsDsDsDDf",
            "fDDDDDDDDDDf",
            "fDrDDDDDDrDf",
        ],
        &[
            ('f', IRON[0]),
            ('D', STEEL[3]),
            ('r', IRON[2]),
            ('h', IRON[3]),
            ('s', [1, 1, 1, 255]),
        ],
    );
    picture.overlay(&door, (2, 2));
    let slit = [1, 1, 1, 255];
    picture.each(|x, y, color| match (color == slit, lit) {
        (true, Some(frame)) => fire(x, y, frame),
        (true, None) => [0x18, 0x10, 0x0c, 255],
        _ => color,
    });
    picture
}

/// The blast furnace's front: fire bricks round an iron-framed tap hole, dark when cold and
/// white-hot with the fire of frame `lit`.
fn tap_hole(lit: Option<usize>) -> Picture {
    let mut picture = bricks(&FIRE_BRICK, FIRE_MORTAR, 409);
    let frame = Picture::drawn(
        &[
            "...dmmmmd...",
            "..dmLLLLmd..",
            ".dmLhhhhLmd.",
            "dmLh....hLmd",
            "mLh......hLm",
            "mL........Lm",
            "mL........Lm",
            "mL........Lm",
            "mL........Lm",
            "mLLLLLLLLLLm",
            "dmmmmmmmmmmd",
        ],
        &[
            ('d', IRON[0]),
            ('m', IRON[1]),
            ('L', IRON[2]),
            ('h', STEEL[1]),
            ('.', [1, 1, 1, 255]),
        ],
    );
    picture.overlay(&frame, (2, 4));
    let hole = [1, 1, 1, 255];
    picture.each(|x, y, color| match (color == hole, lit) {
        (true, Some(frame)) => fire(x, y + 3, frame),
        (true, None) => [0x1c, 0x16, 0x12, 255],
        _ => color,
    });
    picture
}

/// The glow mask of `picture`: full where it shows fire, none elsewhere.
pub fn glow_of(picture: &Picture) -> Picture {
    let mut mask = picture.clone();
    mask.each(|_, _, color| if FIRE.contains(&color) { GLOWS } else { DARK });
    mask
}

/// A block of steel: four riveted plates of dark steel.
pub fn steel_block() -> Picture {
    let mut picture = Picture::surface(&STEEL[2..], 433);
    picture.each(|x, y, color| {
        let seam = x % 8 == 7 || y % 8 == 7;
        let rivet = matches!((x % 8, y % 8), (1, 1) | (5, 1) | (1, 5) | (5, 5));
        let lit = x % 8 == 0 || y % 8 == 0;
        match (seam, rivet, lit) {
            (true, ..) => STEEL[0],
            (_, true, _) => IRON[3],
            (_, _, true) => STEEL[4],
            _ => color,
        }
    });
    picture
}

/// The anvil's iron: dark cast iron, smooth, flecked lighter and darker here and there.
fn anvil_iron() -> Picture {
    let mut picture = Picture::filled(STEEL[1]);
    picture.each(|x, y, color| match noise(x, y, 443) % 12 {
        0 => STEEL[2],
        1 => STEEL[0],
        _ => color,
    });
    picture
}

/// A hot steel bloom: orange-hot, its pits darker, its crust glowing yellow.
fn hot_bloom() -> Picture {
    let mut picture = Picture::filled(FIRE[2]);
    picture.each(|x, y, _| match noise(x, y, 449) % 6 {
        0 => FIRE[1],
        1 => FIRE[3],
        2 if noise(x, y, 457).is_multiple_of(2) => FIRE[4],
        _ => FIRE[2],
    });
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_fire_glows() {
        let door = oven_door(Some(0));
        let mask = glow_of(&door);
        assert_eq!(mask.get(0, 0), DARK, "the bricks don't");
        let slit = (0..16).flat_map(|y| (0..16).map(move |x| (x, y)));
        let glowing = slit.filter(|&(x, y)| mask.get(x, y) == GLOWS).count();
        assert!(glowing >= 20, "its slits do: {glowing}");
        let cold = glow_of(&oven_door(None));
        let any = (0..16).any(|y| (0..16).any(|x| cold.get(x, y) == GLOWS));
        assert!(!any, "a cold door has no fire");
    }

    #[test]
    fn a_flickering_fire_is_a_strip_of_square_frames() {
        let strip = flickering(|frame| tap_hole(Some(frame)));
        assert_eq!(
            strip.get(0, 16 * FIRE_FRAMES - 1)[3],
            255,
            "the last frame drawn"
        );
    }

    #[test]
    fn bricks_are_laid_in_mortar() {
        let wall = bricks(&FIRE_BRICK, FIRE_MORTAR, 1);
        assert_eq!(wall.get(3, 3), FIRE_MORTAR, "between courses");
        assert_eq!(wall.get(7, 1), FIRE_MORTAR, "between bricks");
        assert_ne!(wall.get(2, 1), FIRE_MORTAR, "a brick");
    }
}
