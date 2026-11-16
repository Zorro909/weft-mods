//! Whether the blocks around a stall register make a stall: 3 wide, 2 deep and 3 high. In front,
//! where buyers stand, the register between two counters; behind them three crates; on the
//! corners a post each; over it all six pieces of awning.
//!
//! ```text
//!  top     A A A     middle   P . P     ground  C C C   (behind)
//!          A A A              P . P             c R c   (front: buyers stand below this row)
//! ```
//!
//! The world comes in as a closure saying which part a block is, so tests reach it.

use weft_sdk::Dir;
use weftkit::Pos;

/// What a stall is built of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Register,
    Counter,
    Crate,
    Post,
    Awning,
}

/// Every block of the stall whose register stands at `register` facing `front`, with the part
/// that belongs there; the register first.
pub fn layout(register: Pos, front: Dir) -> Vec<(Pos, Part)> {
    let side = front.clockwise();
    let at = |across: i32, back: i32, up: i32| {
        register
            .toward_by(side, across)
            .toward_by(front, -back)
            .add_y(up)
    };
    let mut blocks = vec![
        (register, Part::Register),
        (at(-1, 0, 0), Part::Counter),
        (at(1, 0, 0), Part::Counter),
    ];
    for across in -1..=1 {
        blocks.push((at(across, 1, 0), Part::Crate));
        for back in 0..=1 {
            blocks.push((at(across, back, 2), Part::Awning));
        }
    }
    for across in [-1, 1] {
        for back in 0..=1 {
            blocks.push((at(across, back, 1), Part::Post));
        }
    }
    blocks
}

/// Whether the stall of the register at `register` facing `front` stands whole; else the first
/// part missing and where it belongs. `part_at` says which part a block is.
pub fn check(
    register: Pos,
    front: Dir,
    part_at: impl Fn(Pos) -> Option<Part>,
) -> Result<(), (Part, Pos)> {
    match layout(register, front)
        .into_iter()
        .find(|(pos, part)| part_at(*pos) != Some(*part))
    {
        Some((pos, part)) => Err((part, pos)),
        None => Ok(()),
    }
}

/// Where the register of a stall holding the block `pos` may stand: within one block across,
/// one behind or in front, and two below.
pub fn registers_near(pos: Pos) -> impl Iterator<Item = Pos> {
    (-2..=0).flat_map(move |dy| {
        (-1..=1).flat_map(move |dx| (-1..=1).map(move |dz| pos.add_x(dx).add_y(dy).add_z(dz)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTER: Pos = Pos::new(10, 5, 10);

    /// The part `layout` puts at `pos`, as if the world held a whole stall.
    fn whole(front: Dir) -> impl Fn(Pos) -> Option<Part> {
        move |pos| {
            layout(REGISTER, front)
                .into_iter()
                .find(|(at, _)| *at == pos)
                .map(|(_, part)| part)
        }
    }

    #[test]
    fn a_whole_stall_stands_whichever_way_it_faces() {
        for front in [Dir::North, Dir::East, Dir::South, Dir::West] {
            assert_eq!(check(REGISTER, front, whole(front)), Ok(()), "{front:?}");
        }
    }

    #[test]
    fn the_crates_stand_behind_the_front() {
        let crates: Vec<Pos> = layout(REGISTER, Dir::North)
            .into_iter()
            .filter(|(_, part)| *part == Part::Crate)
            .map(|(pos, _)| pos)
            .collect();
        assert!(
            crates.iter().all(|pos| pos.z == REGISTER.z + 1),
            "facing north, behind is south: {crates:?}"
        );
    }

    #[test]
    fn a_missing_part_is_named_with_its_place() {
        let front = Dir::North;
        let gap = REGISTER.add_y(2);
        let without = |pos: Pos| (pos != gap).then(|| whole(front)(pos)).flatten();
        assert_eq!(
            check(REGISTER, front, without),
            Err((Part::Awning, gap)),
            "the awning over the register"
        );
    }

    #[test]
    fn every_part_finds_its_register() {
        for (pos, _) in layout(REGISTER, Dir::East) {
            assert!(
                registers_near(pos).any(|near| near == REGISTER),
                "from {pos:?}"
            );
        }
    }

    #[test]
    fn a_stall_has_one_register_and_six_pieces_of_awning() {
        let parts: Vec<Part> = layout(REGISTER, Dir::South)
            .into_iter()
            .map(|(_, part)| part)
            .collect();
        let count = |of: Part| parts.iter().filter(|part| **part == of).count();
        assert_eq!(
            [
                count(Part::Register),
                count(Part::Counter),
                count(Part::Crate),
                count(Part::Post),
                count(Part::Awning)
            ],
            [1, 2, 3, 4, 6],
            "the plan's parts"
        );
    }
}
