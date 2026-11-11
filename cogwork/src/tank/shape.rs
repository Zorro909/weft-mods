//! The shape of a tank block, for each set of sides joined to more of the tank (`boxes`).
//!
//! The outside of a tank is drawn, the inside where blocks join is not: glass on its outer
//! sides, a slim frame along the edges where two outer sides meet, brass caps on its corners,
//! and a riveted base around its bottom layer. The caps reach the block's sides; the frame and
//! base lie a texel in from them and the glass inside the frame, so the caps stand out. Every
//! face lies on a whole texel, as the engine draws them, and none lies in another's plane facing
//! the same way, where the two would flicker.

use weft_sdk::Dir;

use crate::boxes::Piece;

/// Texels in from an outer side its glass lies: inside the frame.
pub const GLASS: f32 = 2.0;
/// How far in from an outer side the frame and base lie, and how thick the frame is.
const INSET: f32 = 1.0;
const FRAME: f32 = 2.0;
/// How far a corner cap reaches along each edge.
const CAP: f32 = 3.0;
/// How high the base of the bottom layer is.
pub const BASE: f32 = 5.0;

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
    let (x, z) = (
        across(Dir::West, outer, GLASS),
        across(Dir::North, outer, GLASS),
    );
    let floor = if outer(Dir::Down) { BASE } else { 0.0 };
    let lid = across(Dir::Down, outer, GLASS).1;
    let corners = ([x.0, floor, z.0], [x.1, lid, z.1]);
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
    let (x, z) = (
        across(Dir::West, outer, INSET),
        across(Dir::North, outer, INSET),
    );
    Piece::new(([x.0, 0.0, z.0], [x.1, BASE, z.1]), face)
}

/// A bar along every edge where two outer sides meet, from cap to cap: the four upright edges
/// from the base up, and the four of the top. The base is the edge of the bottom.
fn frame(outer: &impl Fn(Dir) -> bool) -> Vec<Piece> {
    let mut bars = Vec::new();
    for x_side in [Dir::West, Dir::East] {
        for z_side in [Dir::North, Dir::South] {
            if outer(x_side) && outer(z_side) {
                let floor = if outer(Dir::Down) { BASE } else { 0.0 };
                let top = across(Dir::Down, outer, CAP).1;
                let (x, z) = (edge(x_side), edge(z_side));
                bars.push(([x.0, floor, z.0], [x.1, top, z.1]));
            }
        }
    }
    if outer(Dir::Up) {
        let y = edge(Dir::Up);
        for side in [Dir::West, Dir::East]
            .into_iter()
            .filter(|&side| outer(side))
        {
            let (x, z) = (edge(side), across(Dir::North, outer, CAP));
            bars.push(([x.0, y.0, z.0], [x.1, y.1, z.1]));
        }
        for side in [Dir::North, Dir::South]
            .into_iter()
            .filter(|&side| outer(side))
        {
            let (x, z) = (across(Dir::West, outer, CAP), edge(side));
            bars.push(([x.0, y.0, z.0], [x.1, y.1, z.1]));
        }
    }
    let bar = |corners| Piece::new(corners, |_| Some("liquid_tank_frame"));
    bars.into_iter().map(bar).collect()
}

/// A brass cap on each corner where three outer sides meet. A bottom one has no face below,
/// where the base's is.
fn caps(outer: &impl Fn(Dir) -> bool) -> Vec<Piece> {
    let mut caps = Vec::new();
    for x_side in [Dir::West, Dir::East] {
        for y_side in [Dir::Down, Dir::Up] {
            for z_side in [Dir::North, Dir::South] {
                if [x_side, y_side, z_side].into_iter().all(outer) {
                    let [x, y, z] = [x_side, y_side, z_side].map(|dir| at_side(dir, 0.0, CAP));
                    let corners = ([x.0, y.0, z.0], [x.1, y.1, z.1]);
                    let face = |dir: Dir| (dir != Dir::Down).then_some("liquid_tank_cap");
                    caps.push(Piece::new(corners, face));
                }
            }
        }
    }
    caps
}

/// From and to, along the axis of `low`, the low side of it, of a piece reaching across the
/// block: `by` short of each outer side, on to the edge where more of the tank goes on.
fn across(low: Dir, outer: &impl Fn(Dir) -> bool, by: f32) -> (f32, f32) {
    let from = if outer(low) { by } else { 0.0 };
    let to = if outer(low.opposite()) {
        16.0 - by
    } else {
        16.0
    };
    (from, to)
}

/// From and to, along `dir`'s axis, of a frame bar on the side `dir`.
fn edge(dir: Dir) -> (f32, f32) {
    at_side(dir, INSET, FRAME)
}

/// From and to, along `dir`'s axis, of a box `thick` deep, `inset` in from the side `dir`.
fn at_side(dir: Dir, inset: f32, thick: f32) -> (f32, f32) {
    // Even faces are the low side of their axis.
    if dir.face().is_multiple_of(2) {
        (inset, inset + thick)
    } else {
        (16.0 - inset - thick, 16.0 - inset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many of `pieces` show `texture` on every face they have.
    fn count(pieces: &[Piece], texture: &str) -> usize {
        let all = |piece: &&Piece| piece.faces.iter().flatten().all(|face| face == texture);
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

    /// Whether `a`'s and `b`'s faces toward `dir` lie in one plane, overlapping.
    fn flicker(a: &Piece, b: &Piece, dir: Dir) -> bool {
        let face = dir.face() as usize;
        if a.faces[face].is_none() || b.faces[face].is_none() {
            return false;
        }
        let axis = face / 2;
        let plane = |piece: &Piece| {
            if face.is_multiple_of(2) {
                piece.corners.0[axis]
            } else {
                piece.corners.1[axis]
            }
        };
        let overlap = (0..3).filter(|&other| other != axis).all(|other| {
            a.corners.0[other] < b.corners.1[other] && b.corners.0[other] < a.corners.1[other]
        });
        plane(a) == plane(b) && overlap
    }

    #[test]
    fn no_two_faces_flicker_in_one_plane() {
        for sides in 0..64u32 {
            let pieces = pieces(|dir: Dir| sides & 1 << dir.face() != 0);
            for (i, a) in pieces.iter().enumerate() {
                for b in &pieces[i + 1..] {
                    for dir in Dir::ALL {
                        assert!(
                            !flicker(a, b, dir),
                            "{:?} and {:?} toward {dir:?}, joined {sides:06b}",
                            a.corners,
                            b.corners
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn every_face_lies_on_a_whole_texel() {
        for sides in 0..64u32 {
            for piece in pieces(|dir: Dir| sides & 1 << dir.face() != 0) {
                let (from, to) = piece.corners;
                let whole = from.iter().chain(&to).all(|at| at.fract() == 0.0);
                assert!(whole, "{:?}: the engine rounds it", piece.corners);
            }
        }
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
