//! Pictures of what a refinery makes: tar, lubricant, diesel and gas, still and in their buckets,
//! each drawn as oil is, a band of sheen drifting over it; and asphalt, the road block made of tar.

use super::steel_items::bucket_of;
use super::{Picture, Rgba, noise};

/// Tar: black and thick, a dull brown sheen.
const TAR: [Rgba; 5] = [
    [0x08, 0x07, 0x06, 255],
    [0x10, 0x0e, 0x0c, 255],
    [0x18, 0x15, 0x12, 255],
    [0x2c, 0x26, 0x20, 255],
    [0x44, 0x3a, 0x30, 255],
];
/// Lubricant: a deep golden oil.
const LUBRICANT: [Rgba; 5] = [
    [0x4a, 0x34, 0x08, 255],
    [0x6a, 0x4c, 0x10, 255],
    [0x84, 0x62, 0x18, 255],
    [0xb0, 0x8a, 0x2c, 255],
    [0xe0, 0xc0, 0x5c, 255],
];
/// Diesel: a pale amber, lighter than lubricant.
const DIESEL: [Rgba; 5] = [
    [0x7a, 0x5a, 0x1a, 255],
    [0x9a, 0x76, 0x26, 255],
    [0xb4, 0x8e, 0x34, 255],
    [0xd2, 0xae, 0x52, 255],
    [0xf0, 0xd8, 0x8a, 255],
];
/// Gas, as a liquid: a pale straw, nearly white.
const GAS: [Rgba; 5] = [
    [0xa8, 0xa4, 0x7e, 255],
    [0xbc, 0xb8, 0x92, 255],
    [0xcc, 0xc8, 0xa2, 255],
    [0xe2, 0xde, 0xbc, 255],
    [0xf6, 0xf4, 0xe0, 255],
];
/// Asphalt: near-black binder, and the grey grit of its aggregate.
const BINDER: [Rgba; 3] = [
    [0x26, 0x26, 0x28, 255],
    [0x2e, 0x2e, 0x30, 255],
    [0x22, 0x22, 0x24, 255],
];
const GRIT: [Rgba; 2] = [[0x5a, 0x58, 0x56, 255], [0x78, 0x76, 0x72, 255]];
/// Frames of a liquid's sheen, each shown for its `.anim`'s frametime.
const FRAMES: usize = 8;

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/tar_still", still(&TAR, 811)),
        ("item/tar_bucket", bucket_of(&TAR)),
        ("block/lubricant_still", still(&LUBRICANT, 821)),
        ("item/lubricant_bucket", bucket_of(&LUBRICANT)),
        ("block/diesel_still", still(&DIESEL, 823)),
        ("item/diesel_bucket", bucket_of(&DIESEL)),
        ("block/gas_still", still(&GAS, 827)),
        ("item/gas_bucket", bucket_of(&GAS)),
        ("block/asphalt", asphalt()),
    ]
}

/// A liquid of `colors`, dark to its sheen: a band of sheen drifting across it frame by frame.
fn still(colors: &[Rgba; 5], salt: u32) -> Picture {
    let frames: Vec<Picture> = (0..FRAMES)
        .map(|frame| {
            let mut picture = Picture::surface(&colors[..3], salt);
            picture.each(|x, y, color| {
                let band = (x + 2 * y + 16 - 2 * frame % 16) % 16;
                match (band, noise(x, y, salt + 1) % 3) {
                    (0, 0) => colors[4],
                    (0 | 1, _) => colors[3],
                    _ => color,
                }
            });
            picture
        })
        .collect();
    Picture::stacked(&frames)
}

/// Asphalt: dark binder flecked with grey grit, a few lighter stones.
fn asphalt() -> Picture {
    let mut picture = Picture::surface(&BINDER, 829);
    picture.each(|x, y, color| match noise(x, y, 839) % 16 {
        0 => GRIT[0],
        1 => GRIT[1],
        _ => color,
    });
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_liquid_drifts_over_its_frames() {
        let picture = still(&DIESEL, 1);
        assert_eq!(picture.height, 16 * FRAMES, "a strip of frames");
        let first = (0..16).map(|y| picture.get(0, y)).collect::<Vec<_>>();
        let second = (16..32).map(|y| picture.get(0, y)).collect::<Vec<_>>();
        assert_ne!(first, second, "the sheen moves");
    }

    #[test]
    fn asphalt_is_mostly_binder() {
        let picture = asphalt();
        let texels = (0..16).flat_map(|y| (0..16).map(move |x| (x, y)));
        let binder = texels
            .filter(|&(x, y)| BINDER.contains(&picture.get(x, y)))
            .count();
        assert!(binder > 160, "{binder} of 256 texels dark");
    }
}
