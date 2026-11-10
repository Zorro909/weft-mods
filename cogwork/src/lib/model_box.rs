//! A box of a block's model, in texels: 16 to a block. The model and the hitbox lie in the
//! block's cell, from 0 to 16 along each axis. The SDK takes a box as its two corners
//! (`model_box`, `hitbox`).

use weft_sdk::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ModelBox {
    /// The lowest corner.
    pub lo: [f32; 3],
    /// The highest corner.
    pub hi: [f32; 3],
}

impl ModelBox {
    /// The whole cell of a block.
    pub const BLOCK: ModelBox = ModelBox::new([0.0; 3], [16.0; 3]);

    pub const fn new(lo: [f32; 3], hi: [f32; 3]) -> ModelBox {
        ModelBox { lo, hi }
    }

    /// The box as thick as this one that reaches from its face toward `dir` out to the edge of
    /// the block: a tube's or a shaft's arm off its hub.
    pub fn arm(self, dir: Dir) -> ModelBox {
        let axis = dir.face() as usize / 2;
        let (mut lo, mut hi) = (self.lo, self.hi);
        // Even faces are the low side of their axis.
        if dir.face().is_multiple_of(2) {
            (lo[axis], hi[axis]) = (0.0, self.lo[axis]);
        } else {
            (lo[axis], hi[axis]) = (self.hi[axis], 16.0);
        }
        ModelBox { lo, hi }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tube's hub: 8 texels wide in the middle of the block.
    const HUB: ModelBox = ModelBox::new([4.0; 3], [12.0; 3]);

    #[test]
    fn arms_reach_from_the_hub_to_the_edge() {
        assert_eq!(
            HUB.arm(Dir::West),
            ModelBox::new([0.0, 4.0, 4.0], [4.0, 12.0, 12.0]),
            "west"
        );
        assert_eq!(
            HUB.arm(Dir::Up),
            ModelBox::new([4.0, 12.0, 4.0], [12.0, 16.0, 12.0]),
            "up"
        );
        assert_eq!(
            HUB.arm(Dir::South),
            ModelBox::new([4.0, 4.0, 12.0], [12.0, 12.0, 16.0]),
            "south"
        );
    }
}
