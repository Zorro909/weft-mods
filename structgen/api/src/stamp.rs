//! A part's blocks that fall into one cube: a template with its foundation pillars and the
//! ground cut away above it, or a box of blocks. Each cube writes only its own share.

use weftkit::Pos;

use crate::part::{Foundation, Part, Piece};
use crate::templates::Resolved;

/// The block the ground has on top of column (`x`, `z`), 0 where there is none.
pub(crate) type Surface<'a> = &'a dyn Fn(i32, i32) -> u32;

/// Calls `set` with the position inside the cube at `origin` and the state of everything
/// `part` puts there.
pub(crate) fn stamp(
    part: &Part,
    origin: Pos,
    surface: Surface,
    set: &mut dyn FnMut([usize; 3], u32),
) {
    match part {
        Part::Piece(piece) => {
            if let Some(resolved) = piece.template.resolved() {
                stamp_piece(piece, resolved, origin, surface, set);
            }
        }
        Part::Fill { area, state } => {
            let ([sx, sy, sz], lo) = (area.size(), area.lo);
            for (_, z) in overlap(lo.z, sz, origin.z) {
                for (_, y) in overlap(lo.y, sy, origin.y) {
                    for (_, x) in overlap(lo.x, sx, origin.x) {
                        set([x, y, z], *state);
                    }
                }
            }
        }
    }
}

fn stamp_piece(
    piece: &Piece,
    resolved: &Resolved,
    origin: Pos,
    surface: Surface,
    set: &mut dyn FnMut([usize; 3], u32),
) {
    let [sx, sy, sz] = piece.size();
    let Pos {
        x: cx,
        y: cy,
        z: cz,
    } = piece.corner;
    for (dz, z) in overlap(cz, sz, origin.z) {
        for (dx, x) in overlap(cx, sx, origin.x) {
            for (dy, y) in overlap(cy, sy, origin.y) {
                set([x, y, z], resolved.state(piece.turn, Pos::new(dx, dy, dz)));
            }
            let ground = piece.ground.get((dz * sx + dx) as usize).copied().flatten();
            let Some(ground) = ground else {
                continue;
            };
            let floor = resolved.state(piece.turn, Pos::new(dx, 0, dz));
            let world = [origin.x + x as i32, origin.z + z as i32];
            if let Some(state) = foundation(piece, resolved, floor, surface, world) {
                for (_, y) in overlap(ground + 1, cy - ground - 1, origin.y) {
                    set([x, y, z], state);
                }
            }
            if piece.clear_above {
                for (_, y) in overlap(cy + sy, ground - cy - sy + 1, origin.y) {
                    set([x, y, z], resolved.air);
                }
            }
        }
    }
}

/// What fills under `floor`, the bottom block of world column (`x`, `z`): nothing under air,
/// and nothing where the ground has no top block or the block is unknown.
fn foundation(
    piece: &Piece,
    resolved: &Resolved,
    floor: u32,
    surface: Surface,
    [x, z]: [i32; 2],
) -> Option<u32> {
    let state = match piece.foundation.filter(|_| floor != resolved.air)? {
        Foundation::Ground => surface(x, z),
        Foundation::Block(block) => block.id(),
    };
    (state != 0 && state != u32::MAX).then_some(state)
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
    use weftkit::Area;

    use super::*;
    use crate::format::Structure;
    use crate::templates;
    use crate::turn::Turn;

    const AIR: u32 = 0;
    const FLOOR: u32 = 1;
    const WALL: u32 = 2;
    const STONE: u32 = 9;

    /// 2 x 2 x 2: a floor with one corner missing, and walls above.
    fn resolved() -> Resolved {
        Resolved {
            size: [2, 2, 2],
            indices: vec![1, 1, 1, 0, 2, 2, 2, 2],
            palettes: [0, 1, 2, 3].map(|_| vec![AIR, FLOOR, WALL]),
            air: AIR,
        }
    }

    fn piece(corner: Pos, ground: Vec<i32>) -> Piece {
        let template = Structure::from_names([2, 2, 2], ["core:air"; 8]).expect("8 names");
        Piece {
            template: templates::add("test:stamp", template),
            corner,
            turn: Turn::None,
            foundation: Some(Foundation::Ground),
            clear_above: true,
            ground: ground.into_iter().map(Some).collect(),
        }
    }

    fn placed(piece: &Piece, origin: Pos) -> Vec<([usize; 3], u32)> {
        let mut out = Vec::new();
        stamp_piece(piece, &resolved(), origin, &|_, _| STONE, &mut |p, s| {
            out.push((p, s))
        });
        out.sort();
        out
    }

    #[test]
    fn flat_ground_gets_just_the_structure() {
        let blocks = placed(&piece(Pos::new(3, 5, 3), vec![4; 4]), Pos::default());
        assert_eq!(blocks.len(), 8, "no foundation and nothing cut");
        assert!(blocks.contains(&([4, 5, 4], AIR)), "its air clears too");
    }

    #[test]
    fn foundations_reach_down_to_the_ground() {
        let blocks = placed(&piece(Pos::new(3, 5, 3), vec![1, 4, 4, 1]), Pos::default());
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
    fn a_ground_foundation_takes_each_columns_top_block() {
        let piece = piece(Pos::new(3, 5, 3), vec![1, 4, 4, 1]);
        let mut blocks = Vec::new();
        let sand_at_x3 = |x: i32, _: i32| if x == 3 { 12 } else { STONE };
        stamp_piece(
            &piece,
            &resolved(),
            Pos::default(),
            &sand_at_x3,
            &mut |p, s| blocks.push((p, s)),
        );
        let under = |x: usize, z: usize| {
            blocks
                .iter()
                .filter(move |(p, _)| p[0] == x && p[2] == z && p[1] < 5)
        };
        assert!(
            under(3, 3).all(|(_, s)| *s == 12),
            "the top block of its own column"
        );
        let nothing = |_: i32, _: i32| 0;
        let mut set = 0;
        stamp_piece(
            &piece,
            &resolved(),
            Pos::default(),
            &nothing,
            &mut |p, _| set += (p[1] < 5) as u32,
        );
        assert_eq!(set, 0, "no foundation where the ground has no top block");
    }

    #[test]
    fn ground_above_the_structure_is_cut() {
        let cut: Vec<usize> = placed(&piece(Pos::new(3, 5, 3), vec![9, 4, 4, 4]), Pos::default())
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
    fn without_modifiers_only_the_structure_is_set() {
        let mut bare = piece(Pos::new(3, 5, 3), vec![1, 9, 1, 9]);
        bare.foundation = None;
        bare.clear_above = false;
        assert_eq!(
            placed(&bare, Pos::default()).len(),
            8,
            "the ground is left as it is"
        );
    }

    #[test]
    fn a_cube_gets_only_its_share() {
        let piece = piece(Pos::new(15, 5, 3), vec![4; 4]);
        let here = placed(&piece, Pos::default());
        let next = placed(&piece, Pos::new(16, 0, 0));
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

    #[test]
    fn a_fill_is_cut_at_the_cube() {
        let fill = Part::Fill {
            area: Area::between(Pos::new(14, 0, 0), Pos::new(17, 0, 0)),
            state: STONE,
        };
        let mut here = 0;
        stamp(&fill, Pos::default(), &|_, _| 0, &mut |_, _| here += 1);
        assert_eq!(here, 2, "two of its four blocks are in this cube");
    }
}
