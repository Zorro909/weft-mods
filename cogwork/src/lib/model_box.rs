//! A box of a model, in texels: 16 to a block. A block's model and hitbox lie in its cell, from 0
//! to 16 along each axis; an entity's model lies around its feet. The SDK takes a box as its two
//! corners (`model_box`, `hitbox`) or as a corner and a size (`ModelPart::cuboid`).

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

    /// The box from the corner `lo`, `size` across.
    pub const fn sized(lo: [f32; 3], size: [f32; 3]) -> ModelBox {
        let hi = [lo[0] + size[0], lo[1] + size[1], lo[2] + size[2]];
        ModelBox { lo, hi }
    }

    pub fn size(self) -> [f32; 3] {
        [0, 1, 2].map(|i| self.hi[i] - self.lo[i])
    }

    /// The box as thick as this one that reaches from its face toward `d` out to the edge of
    /// the block: a tube's or a shaft's arm off its hub.
    pub fn arm(self, d: Dir) -> ModelBox {
        let axis = d.face() as usize / 2;
        let (mut lo, mut hi) = (self.lo, self.hi);
        // Even faces are the low side of their axis.
        if d.face().is_multiple_of(2) {
            (lo[axis], hi[axis]) = (0.0, self.lo[axis]);
        } else {
            (lo[axis], hi[axis]) = (self.hi[axis], 16.0);
        }
        ModelBox { lo, hi }
    }

    /// The entity model part `name` made of this box alone, its texture at `uv`.
    pub fn part(self, name: &str, uv: [f32; 2]) -> ModelPart {
        ModelPart::new(name, [0.0; 3]).cuboid(self.lo, self.size(), uv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tube's hub: 8 texels wide in the middle of the block.
    const HUB: ModelBox = ModelBox::new([4.0; 3], [12.0; 3]);

    #[test]
    fn a_box_from_its_corner_and_size() {
        let plate = ModelBox::sized([-8.0, -2.0, -8.0], [16.0, 4.0, 16.0]);
        assert_eq!(plate.hi, [8.0, 2.0, 8.0], "the far corner");
        assert_eq!(plate.size(), [16.0, 4.0, 16.0], "and back");
        assert_eq!(ModelBox::BLOCK.size(), [16.0; 3], "a block is 16 across");
    }

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
