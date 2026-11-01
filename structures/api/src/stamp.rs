//! A site's blocks that fall into one cube: the structure itself, foundation pillars from its
//! bottom layer down to the ground, and the ground that rises above it cut away. Each cube
//! writes only its own share.

use crate::site::Site;

/// A structure ready to place: block states, known once the game has loaded.
#[derive(Debug)]
pub struct Resolved {
    pub size: [i32; 3],
    /// The state of every block, x fastest, then z, then y.
    pub states: Vec<u32>,
    pub air: u32,
    /// Filled under the bottom layer down to the ground; none leaves the gap.
    pub foundation: Option<u32>,
}

impl Resolved {
    fn state(&self, x: i32, y: i32, z: i32) -> u32 {
        let [sx, _, sz] = self.size;
        self.states[((y * sz + z) * sx + x) as usize]
    }
}

/// Calls `set` with the position inside the cube at `origin` and the state of everything
/// `site` puts there.
pub fn stamp(
    resolved: &Resolved,
    site: &Site,
    origin: [i32; 3],
    set: &mut dyn FnMut([usize; 3], u32),
) {
    let [sx, sy, sz] = resolved.size;
    let [cx, cy, cz] = site.corner;
    for (dz, z) in overlap(cz, sz, origin[2]) {
        for (dx, x) in overlap(cx, sx, origin[0]) {
            let ground = site.ground[(dz * sx + dx) as usize];
            for (dy, y) in overlap(cy, sy, origin[1]) {
                set([x, y, z], resolved.state(dx, dy, dz));
            }
            if let Some(foundation) = resolved
                .foundation
                .filter(|_| resolved.state(dx, 0, dz) != resolved.air)
            {
                for (_, y) in overlap(ground + 1, cy - ground - 1, origin[1]) {
                    set([x, y, z], foundation);
                }
            }
            for (_, y) in overlap(cy + sy, ground - cy - sy + 1, origin[1]) {
                set([x, y, z], resolved.air);
            }
        }
    }
}

/// Where `len` blocks from `start` meet the cube starting at `origin`, along one axis: each
/// as (offset from `start`, position in the cube).
fn overlap(start: i32, len: i32, origin: i32) -> impl Iterator<Item = (i32, usize)> {
    let from = start.max(origin);
    let to = (start + len).min(origin + 16);
    (from..to).map(move |p| (p - start, (p - origin) as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    const AIR: u32 = 0;
    const FLOOR: u32 = 1;
    const WALL: u32 = 2;
    const STONE: u32 = 9;

    /// 2 x 2 x 2: a floor with one corner missing, and walls above.
    fn resolved() -> Resolved {
        Resolved {
            size: [2, 2, 2],
            states: vec![FLOOR, FLOOR, FLOOR, AIR, WALL, WALL, WALL, WALL],
            air: AIR,
            foundation: Some(STONE),
        }
    }

    fn placed(site: &Site, origin: [i32; 3]) -> Vec<([usize; 3], u32)> {
        let mut out = Vec::new();
        stamp(&resolved(), site, origin, &mut |p, s| out.push((p, s)));
        out.sort();
        out
    }

    #[test]
    fn flat_ground_gets_just_the_structure() {
        let site = Site {
            corner: [3, 5, 3],
            ground: vec![4; 4],
        };
        let blocks = placed(&site, [0, 0, 0]);
        assert_eq!(blocks.len(), 8, "no foundation and nothing cut");
        assert!(blocks.contains(&([4, 5, 4], AIR)), "its air clears too");
    }

    #[test]
    fn foundations_reach_down_to_the_ground() {
        let site = Site {
            corner: [3, 5, 3],
            ground: vec![1, 4, 4, 1],
        };
        let blocks = placed(&site, [0, 0, 0]);
        let pillar: Vec<usize> = blocks
            .iter()
            .filter(|(p, s)| p[0] == 3 && p[2] == 3 && *s == STONE)
            .map(|(p, _)| p[1])
            .collect();
        assert_eq!(pillar, [2, 3, 4], "from above the ground up to the floor");
        assert!(
            !blocks
                .iter()
                .any(|(p, s)| p[0] == 4 && p[2] == 4 && *s == STONE),
            "none under a gap in the floor"
        );
    }

    #[test]
    fn ground_above_the_structure_is_cut() {
        let site = Site {
            corner: [3, 5, 3],
            ground: vec![9, 4, 4, 4],
        };
        let cut: Vec<usize> = placed(&site, [0, 0, 0])
            .iter()
            .filter(|(p, s)| p[0] == 3 && p[2] == 3 && p[1] >= 7 && *s == AIR)
            .map(|(p, _)| p[1])
            .collect();
        assert_eq!(
            cut,
            [7, 8, 9],
            "from the top of the structure up to the ground"
        );
    }

    #[test]
    fn a_cube_gets_only_its_share() {
        let site = Site {
            corner: [15, 5, 3],
            ground: vec![4; 4],
        };
        let here = placed(&site, [0, 0, 0]);
        let next = placed(&site, [16, 0, 0]);
        assert!(
            here.iter().all(|(p, _)| p[0] == 15),
            "the first column here"
        );
        assert!(
            next.iter().all(|(p, _)| p[0] == 0),
            "the second in the next cube"
        );
        assert_eq!(here.len() + next.len(), 8, "every block once");
    }
}
