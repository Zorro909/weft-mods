//! Block models put together in code from boxes, for blocks with too many shapes to keep a model
//! file for each: hubs with arms (`arms`) and tanks (`tank::shape`). Every face shows its texture
//! where the box lies in the block, as a model file's face does when Blockbench sets its UV, so a
//! picture drawn over the whole block goes on across boxes.

use weft_sdk::*;

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
