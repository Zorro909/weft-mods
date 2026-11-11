//! Block models put together in code from boxes, for blocks with too many shapes to keep a model
//! file for each: hubs with arms (`arms`) and tanks (`tank::shape`). Every face shows its texture
//! where the box lies in the block, as a model file's face does when Blockbench sets its UV, so a
//! picture drawn over the whole block goes on across boxes.

use weft_sdk::{Dir, State, abi};

/// The two corners of a box, in texels: 16 to a block.
pub type Corners = ([f32; 3], [f32; 3]);

/// One box of a model: where it lies, and the texture of its face toward each side, by
/// `Dir::face`, or none where it has no face.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub corners: Corners,
    pub faces: [Option<String>; 6],
}

impl Piece {
    /// A box at `corners` with the texture `texture` gives toward each side, or no face there.
    pub fn new<'a>(corners: Corners, texture: impl Fn(Dir) -> Option<&'a str>) -> Piece {
        let faces = Dir::ALL.map(|dir| texture(dir).map(str::to_string));
        Piece { corners, faces }
    }
}

/// Leaves out the faces where two of `pieces` meet face to face, as a hub and its arm do: inside
/// the shape, they would show through a see-through texture as a pane across it.
pub fn hide_shared_faces(pieces: &mut [Piece]) {
    for a in 0..pieces.len() {
        for b in a + 1..pieces.len() {
            for dir in Dir::ALL {
                if meet(&pieces[a], &pieces[b], dir) {
                    pieces[a].faces[dir.face() as usize] = None;
                    pieces[b].faces[dir.opposite().face() as usize] = None;
                }
            }
        }
    }
}

/// Whether `a`'s face toward `dir` and `b`'s face back are the same rectangle.
fn meet(a: &Piece, b: &Piece, dir: Dir) -> bool {
    let axis = dir.face() as usize / 2;
    // Even faces are the low side of their axis.
    let side = |piece: &Piece, low: bool| {
        if low {
            piece.corners.0[axis]
        } else {
            piece.corners.1[axis]
        }
    };
    let toward_low = dir.face().is_multiple_of(2);
    let same_plane = side(a, toward_low) == side(b, !toward_low);
    let same_rectangle = (0..3).filter(|&other| other != axis).all(|other| {
        a.corners.0[other] == b.corners.0[other] && a.corners.1[other] == b.corners.1[other]
    });
    same_plane && same_rectangle
}

/// `state` drawn as `pieces`, which are also what it collides with. A piece without a face is
/// left out, as Blockbench leaves it out of a file.
pub fn draw<'s>(state: State<'s>, pieces: &[Piece]) -> State<'s> {
    pieces.iter().fold(state, |state, piece| {
        let Some(any) = piece.faces.iter().flatten().next() else {
            return state;
        };
        // A face left out still names a texture: one the piece has.
        let faces = std::array::from_fn(|face| match &piece.faces[face] {
            Some(texture) => (texture.as_str(), abi::UV_FROM_BOX),
            None => (any.as_str(), abi::UV_NO_FACE),
        });
        let (from, to) = piece.corners;
        state.model_box_faces(from, to, faces)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glass(corners: Corners) -> Piece {
        Piece::new(corners, |_| Some("glass"))
    }

    fn has(piece: &Piece, dir: Dir) -> bool {
        piece.faces[dir.face() as usize].is_some()
    }

    #[test]
    fn where_two_pieces_meet_neither_shows_a_face() {
        let hub = glass(([4.0; 3], [12.0; 3]));
        let arm = glass(([4.0, 4.0, 0.0], [12.0, 12.0, 4.0]));
        let mut pieces = [hub, arm];
        hide_shared_faces(&mut pieces);
        let [hub, arm] = &pieces;
        assert!(
            !has(hub, Dir::North) && !has(arm, Dir::South),
            "both faces between them gone"
        );
        assert!(
            has(hub, Dir::South) && has(arm, Dir::North),
            "their outer ends stay"
        );
        assert!(has(arm, Dir::West), "and their sides");
    }

    #[test]
    fn a_face_only_partly_covered_stays() {
        let hub = glass(([4.0; 3], [12.0; 3]));
        let rod = glass(([6.0, 6.0, 0.0], [10.0, 10.0, 4.0]));
        let mut pieces = [hub, rod];
        hide_shared_faces(&mut pieces);
        assert!(has(&pieces[0], Dir::North), "the hub's face around the rod");
    }
}
