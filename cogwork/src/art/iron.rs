//! Pictures of the Iron Quarry: its block of riveted iron plates, its frame of thin iron bars
//! riveted where they meet, and its rig, the Timber Quarry's pieces (`quarry`) in iron with
//! riveted collars once a block.

use super::quarry::{COG, COG_TURNED, RigPaint, beam, carriage, shaft};
use super::{IRON, Picture, Rgba, noise};

/// The Iron Quarry's rig: plates rimmed dark, bars with riveted collars.
const IRON_RIG: RigPaint = RigPaint {
    plate: iron,
    rim: |_, _| IRON[0],
    pole: iron,
    band: collar,
    cut: IRON[1],
    cut_rim: IRON[0],
};
/// Where the bar runs through a 16 by 16 frame texture: the texels its 4-wide faces show.
const BAR: std::ops::Range<usize> = 6..10;

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/iron_quarry_side", plate(None)),
        ("block/iron_quarry_front", plate(Some(false))),
        ("block/iron_quarry_front_on", plate(Some(true))),
        ("block/iron_quarry_top", top()),
        ("block/iron_frame_bar", bar()),
        ("block/iron_frame_joint", joint()),
        ("block/iron_frame_end", bar_end()),
        ("entity/iron_carriage", carriage(&IRON_RIG)),
        ("entity/iron_shaft", shaft(&IRON_RIG)),
        ("entity/iron_beam", beam(&IRON_RIG)),
    ]
}

/// A shade between the iron's middle tones, for the grain of a plate.
const GRAIN: Rgba = [0x6c, 0x6c, 0x70, 255];

/// Plate iron: even, now and then a little darker.
fn iron(x: usize, y: usize) -> Rgba {
    if noise(x, y, 307).is_multiple_of(5) {
        GRAIN
    } else {
        IRON[2]
    }
}

/// A riveted collar round a bar: dark iron, a rivet head every third texel.
fn collar(x: usize, y: usize) -> Rgba {
    if (x + y).is_multiple_of(3) {
        IRON[3]
    } else {
        IRON[0]
    }
}

/// Whether a pixel is a rivet of a 16 by 16 plate: two in from each corner, and halfway down
/// each side.
fn rivet(x: usize, y: usize) -> bool {
    let near = |at: usize| at == 2 || at == 13;
    (near(x) && (near(y) || y == 8)) || (near(y) && x == 8)
}

/// A side of the quarry: a riveted iron plate in a dark rim. A front shows an iron cog, its
/// teeth turned a little further while it works.
fn plate(cog: Option<bool>) -> Picture {
    let mut picture = Picture::filled(IRON[2]);
    picture.each(|x, y, _| {
        let rim = x == 0 || y == 0 || x == 15 || y == 15;
        match (rim, rivet(x, y), y > 0 && rivet(x, y - 1)) {
            (true, ..) => IRON[0],
            (_, true, _) => IRON[3],
            // Each rivet's shadow under it.
            (_, _, true) => IRON[1],
            _ => iron(x, y),
        }
    });
    if let Some(working) = cog {
        let rows = if working { COG_TURNED } else { COG };
        let cog = Picture::drawn(&rows, &[('d', IRON[0]), ('l', IRON[3]), ('h', IRON[1])]);
        picture.overlay(&cog, (4, 4));
    }
    picture
}

/// The top: the riveted plate with the shaft's hole in the middle, a collar round it.
fn top() -> Picture {
    let mut picture = plate(None);
    picture.each(|x, y, color| {
        let (dx, dy) = (x.abs_diff(8), y.abs_diff(8));
        let hole = (6..10).contains(&x) && (6..10).contains(&y);
        match (hole, dx.max(dy) <= 3 && !hole) {
            (true, _) => IRON[0],
            (_, true) => collar(x, y),
            _ => color,
        }
    });
    picture
}

/// A bar's sides: plain iron, lit along one edge.
fn bar() -> Picture {
    let mut picture = Picture::filled(IRON[2]);
    picture.each(|x, y, _| {
        if x == BAR.start || y == BAR.start {
            IRON[3]
        } else {
            iron(x, y)
        }
    });
    picture
}

/// Where bars meet: a dark plate with a rivet in each corner.
fn joint() -> Picture {
    let mut picture = bar();
    picture.each(|x, y, color| {
        let corner = |at: usize| at == BAR.start || at == BAR.end - 1;
        match (BAR.contains(&x) && BAR.contains(&y), corner(x) && corner(y)) {
            (true, true) => IRON[3],
            (true, false) => IRON[1],
            _ => color,
        }
    });
    picture
}

/// A bar's sawn end: a dark rim round its cut face.
fn bar_end() -> Picture {
    let mut picture = bar();
    picture.each(|x, y, color| {
        let edge = |at: usize| at == BAR.start || at == BAR.end - 1;
        match (BAR.contains(&x) && BAR.contains(&y), edge(x) || edge(y)) {
            (true, true) => IRON[0],
            (true, false) => IRON[1],
            _ => color,
        }
    });
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_joint_is_riveted_where_the_bar_shows() {
        let joint = joint();
        for (x, y) in [(6, 6), (9, 6), (6, 9), (9, 9)] {
            assert_eq!(joint.get(x, y), IRON[3], "a rivet at {x}, {y}");
        }
        assert_eq!(joint.get(7, 7), IRON[1], "a plate between them");
    }

    #[test]
    fn the_rig_is_iron_banded_with_collars() {
        let shaft = shaft(&IRON_RIG);
        let collared = (4..20).filter(|&y| shaft.get(1, y) == IRON[0]).count();
        assert!(collared > 0, "a collar on the shaft's first block");
    }
}
