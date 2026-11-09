//! A building the film goes into, where seed 42's settlement put it: points in the building as
//! drawn are turned with it into the world.

use structgen_api::{Pos, Turn};

pub struct Building {
    /// Its lowest corner in the world.
    pub corner: Pos,
    /// Along x and z as drawn, before it was turned.
    pub size: [i32; 2],
    pub turn: Turn,
}

impl Building {
    /// Where `[x, y, z]` of the building as drawn is in the world: x along its front, y up from
    /// its floor, z in from its front, the side with its door.
    pub fn at(&self, [x, y, z]: [f64; 3]) -> [f64; 3] {
        let [sx, sz] = self.size.map(f64::from);
        let (dx, dz) = match self.turn {
            Turn::None => (x, z),
            Turn::Right => (sz - z, x),
            Turn::Back => (sx - x, sz - z),
            Turn::Left => (z, sx - x),
        };
        let [cx, cy, cz]: [f64; 3] = self.corner.into();
        [cx + dx, cy + y, cz + dz]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_turn_as_the_blocks_do() {
        for turn in Turn::ALL {
            let building = Building {
                corner: Pos::new(100, 64, -50),
                size: [5, 3],
                turn,
            };
            let block = [4, 2, 1];
            let Pos { x, y, z } = turn.pos(Pos::from(block), [5, 9, 3]);
            let at = building.at(block.map(|v| v as f64 + 0.5));
            let expected = [100 + x, 64 + y, -50 + z].map(|v| v as f64 + 0.5);
            assert_eq!(
                at, expected,
                "the middle of the same block, turned {turn:?}"
            );
        }
    }
}
