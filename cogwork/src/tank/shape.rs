//! The shape of a tank block, for each set of sides joined to more of the tank (`boxes`).
//!
//! The outside of a tank is drawn, the inside where blocks join is not: glass on its outer
//! sides, a slim frame along the edges where two outer sides meet, brass caps on its corners,
//! and a riveted base around its bottom layer. Nothing reaches out of the block: the caps stand
//! out of the frame and base because those lie a little in from its outer sides.

use weft_sdk::Dir;

use crate::boxes::Piece;

/// Texels in from an outer side its glass lies, and its lid.
const GLASS: f32 = 1.0;
const LID: f32 = 15.0;
/// How thick the frame is, and how far its corner caps reach.
const FRAME: f32 = 2.0;
const CAP: f32 = 3.0;
/// How far in from an outer side the frame and base lie, so that no face of theirs is in the
/// plane of a cap's.
const INSET: f32 = 0.25;
/// How high the base of the bottom layer is.
const BASE: f32 = 4.0;

/// The pieces of a tank block joined toward the sides `joined` says. None for one joined all
/// round, inside the tank.
pub fn pieces(joined: impl Fn(Dir) -> bool) -> Vec<Piece> {
    let outer = |dir: Dir| !joined(dir);
    let mut pieces: Vec<Piece> = glass(&outer).into_iter().collect();
    if outer(Dir::Down) {
        pieces.push(base(&outer));
    }
    pieces.extend(frame(&outer));
    pieces.extend(caps(&outer));
    pieces
}

/// The glass: a pane on each outer side, its lid on top if that is outer too, standing on the
/// base where there is one. None in the middle of a tank, with no outer side but its bottom.
fn glass(outer: &impl Fn(Dir) -> bool) -> Option<Piece> {
    let sides = [Dir::West, Dir::East, Dir::Up, Dir::North, Dir::South];
    if !sides.into_iter().any(outer) {
        return None;
    }
    let low = |dir| if outer(dir) { GLASS } else { 0.0 };
    let high = |dir| if outer(dir) { 16.0 - GLASS } else { 16.0 };
    let floor = if outer(Dir::Down) { BASE } else { 0.0 };
    let lid = if outer(Dir::Up) { LID } else { 16.0 };
    let corners = (
        [low(Dir::West), floor, low(Dir::North)],
        [high(Dir::East), lid, high(Dir::South)],
    );
    // Its bottom is the base's top, or the glass of the block below.
    let face = |dir: Dir| (dir != Dir::Down && outer(dir)).then_some("liquid_tank_glass");
    Some(Piece::new(corners, face))
}

/// The base around the bottom layer: riveted on its outer sides, its floor showing through the
/// glass above.
fn base(outer: &impl Fn(Dir) -> bool) -> Piece {
    let face = |dir: Dir| match dir {
        Dir::Up => Some("liquid_tank_floor"),
        Dir::Down => Some("liquid_tank_base"),
        _ => outer(dir).then_some("liquid_tank_base"),
    };
    let (x, z) = (along(Dir::West, outer), along(Dir::North, outer));
    Piece::new(([x.0, INSET, z.0], [x.1, BASE, z.1]), face)
}

/// A bar along every edge where two outer sides meet: the four upright edges from the base up,
/// and the four of the top. The base is the edge of the bottom.
fn frame(outer: &impl Fn(Dir) -> bool) -> Vec<Piece> {
    let floor = if outer(Dir::Down) { BASE } else { 0.0 };
    let (x, y, z) = (
        along(Dir::West, outer),
        along(Dir::Down, outer),
        along(Dir::North, outer),
    );
    let mut bars = Vec::new();
    for x_side in [Dir::West, Dir::East] {
        for z_side in [Dir::North, Dir::South] {
            if outer(x_side) && outer(z_side) {
                let (x, z) = (span(x_side, -INSET, FRAME), span(z_side, -INSET, FRAME));
                bars.push(([x.0, floor, z.0], [x.1, y.1, z.1]));
            }
        }
    }
    if outer(Dir::Up) {
        let top = span(Dir::Up, -INSET, FRAME);
        for side in [Dir::West, Dir::East]
            .into_iter()
            .filter(|&side| outer(side))
        {
            let x = span(side, -INSET, FRAME);
            bars.push(([x.0, top.0, z.0], [x.1, top.1, z.1]));
        }
        for side in [Dir::North, Dir::South]
            .into_iter()
            .filter(|&side| outer(side))
        {
            let z = span(side, -INSET, FRAME);
            bars.push(([x.0, top.0, z.0], [x.1, top.1, z.1]));
        }
    }
    let bar = |corners| Piece::new(corners, |_| Some("liquid_tank_frame"));
    bars.into_iter().map(bar).collect()
}

/// A brass cap on each corner where three outer sides meet, out to the block's sides.
fn caps(outer: &impl Fn(Dir) -> bool) -> Vec<Piece> {
    let mut caps = Vec::new();
    for x_side in [Dir::West, Dir::East] {
        for y_side in [Dir::Down, Dir::Up] {
            for z_side in [Dir::North, Dir::South] {
                if [x_side, y_side, z_side].into_iter().all(outer) {
                    let [x, y, z] = [x_side, y_side, z_side].map(|dir| span(dir, 0.0, CAP));
                    let corners = ([x.0, y.0, z.0], [x.1, y.1, z.1]);
                    caps.push(Piece::new(corners, |_| Some("liquid_tank_cap")));
                }
            }
        }
    }
    caps
}

/// From and to, along the axis of `low`, the low side of it, of a piece reaching across the
/// block: `INSET` short of each outer side, on to the edge where more of the tank goes on.
fn along(low: Dir, outer: &impl Fn(Dir) -> bool) -> (f32, f32) {
    let from = if outer(low) { INSET } else { 0.0 };
    let to = if outer(low.opposite()) {
        16.0 - INSET
    } else {
        16.0
    };
    (from, to)
}

/// From and to, along `dir`'s axis, of a box `thick` deep at the side `dir`, standing `out`
/// beyond the block (in from it, for less than nothing).
fn span(dir: Dir, out: f32, thick: f32) -> (f32, f32) {
    // Even faces are the low side of their axis.
    if dir.face().is_multiple_of(2) {
        (-out, thick)
    } else {
        (16.0 - thick, 16.0 + out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many of `pieces` show `texture` all round.
    fn count(pieces: &[Piece], texture: &str) -> usize {
        let all = |piece: &&Piece| {
            piece
                .faces
                .iter()
                .all(|face| face.as_deref() == Some(texture))
        };
        pieces.iter().filter(all).count()
    }

    #[test]
    fn a_lone_tank_has_its_whole_frame() {
        let alone = pieces(|_| false);
        assert_eq!(
            count(&alone, "liquid_tank_frame"),
            8,
            "four upright, four on top"
        );
        assert_eq!(count(&alone, "liquid_tank_cap"), 8, "a cap on every corner");
        let middle = pieces(|dir| dir != Dir::North);
        assert_eq!(
            count(&middle, "liquid_tank_cap"),
            0,
            "no corner of the tank"
        );
    }

    #[test]
    fn no_piece_reaches_out_of_its_block() {
        for sides in 0..64u32 {
            let joined = |dir: Dir| sides & 1 << dir.face() != 0;
            for piece in pieces(joined) {
                let (from, to) = piece.corners;
                let inside = from.iter().chain(&to).all(|&at| (0.0..=16.0).contains(&at));
                assert!(inside, "{:?} joined {sides:06b}", piece.corners);
            }
        }
    }

    #[test]
    fn a_block_inside_the_tank_has_no_pieces() {
        assert!(pieces(|_| true).is_empty(), "nothing to see");
    }
}
