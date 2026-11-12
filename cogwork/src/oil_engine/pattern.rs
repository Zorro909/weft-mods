//! Whether the blocks around an oil engine's controller make the engine: a 3 by 3 by 3 cube
//! behind its front, the combustion chamber in the middle, the controller in the middle of the
//! front face, and the rest of the shell casing, intakes and drives, at least one of each of the
//! last two. Intakes and drives go anywhere on the shell but the front face and the top layer,
//! where the rig stands.
//!
//! The world comes in as a closure saying which part a block is, so tests reach it.

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::Dir;

use crate::lib::{Area, Pos};

/// What an oil engine is built of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Casing,
    Chamber,
    Controller,
    Intake,
    Drive,
}

/// An engine's cube, from the middle of its front face and the way that faces.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cube {
    pub middle: Pos,
    pub front: Dir,
}

impl Cube {
    /// The cube behind a controller at `pos` whose front faces `front`.
    pub fn behind(pos: Pos, front: Dir) -> Cube {
        Cube {
            middle: pos.toward(front.opposite()),
            front,
        }
    }

    /// Its 27 blocks.
    pub fn area(self) -> Area {
        let corner = |step: i32| self.middle.add_x(step).add_y(step).add_z(step);
        Area::between(corner(-1), corner(1))
    }

    /// Whether `pos`, one of its blocks, is on its face toward `dir`.
    pub fn on_face(self, pos: Pos, dir: Dir) -> bool {
        let step = dir.offset();
        let (x, y, z) = (
            pos.x - self.middle.x,
            pos.y - self.middle.y,
            pos.z - self.middle.z,
        );
        x * step.x + y * step.y + z * step.z == 1
    }

    /// The sides `pos`, one of its blocks, faces out of the cube on.
    pub fn outward(self, pos: Pos) -> impl Iterator<Item = Dir> {
        Dir::ALL
            .into_iter()
            .filter(move |&dir| self.on_face(pos, dir))
    }

    /// Whether an intake or a drive may be at `pos`, one of its shell blocks: not on its front
    /// face nor on top.
    fn takes_a_port(self, pos: Pos) -> bool {
        !self.on_face(pos, self.front) && !self.on_face(pos, Dir::Up)
    }
}

/// The blocks of a built engine that do something: where its liquid comes in and its energy
/// goes out, and the casing that shows it formed.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Layout {
    pub casings: Vec<Pos>,
    pub intakes: Vec<Pos>,
    pub drives: Vec<Pos>,
}

/// What keeps an engine from being built; its screen says `cogwork.oil_engine.<name>`. Kept in
/// `OilEngine::fault` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Fault {
    /// The block there is no part of an engine.
    #[default]
    NotPart,
    /// The middle holds no combustion chamber.
    NoChamber,
    /// A combustion chamber or a second controller in the shell.
    Stray,
    /// An intake or a drive on the front face or on top.
    Misplaced,
    NoIntake,
    NoDrive,
}

/// What keeps an engine from being built, and the block where: the middle of the cube when it is
/// none in particular.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Problem {
    pub fault: Fault,
    pub at: Pos,
}

/// The layout of the engine whose controller is at `controller` facing `front`, `part_at`
/// telling what each block is; or the first thing wrong with it.
pub fn check(
    controller: Pos,
    front: Dir,
    part_at: impl Fn(Pos) -> Option<Part>,
) -> Result<Layout, Problem> {
    let cube = Cube::behind(controller, front);
    let mut layout = Layout::default();
    let wrong = |fault: Fault, at: Pos| Err(Problem { fault, at });
    for pos in cube.area().blocks() {
        let part = part_at(pos);
        if pos == cube.middle {
            if part != Some(Part::Chamber) {
                return wrong(Fault::NoChamber, pos);
            }
            continue;
        }
        match part {
            None => return wrong(Fault::NotPart, pos),
            Some(Part::Controller) if pos == controller => {}
            Some(Part::Controller | Part::Chamber) => return wrong(Fault::Stray, pos),
            Some(Part::Casing) => layout.casings.push(pos),
            Some(Part::Intake | Part::Drive) if !cube.takes_a_port(pos) => {
                return wrong(Fault::Misplaced, pos);
            }
            Some(Part::Intake) => layout.intakes.push(pos),
            Some(Part::Drive) => layout.drives.push(pos),
        }
    }
    if layout.intakes.is_empty() {
        return wrong(Fault::NoIntake, cube.middle);
    }
    if layout.drives.is_empty() {
        return wrong(Fault::NoDrive, cube.middle);
    }
    Ok(layout)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// A controller at the origin facing north, so the cube lies south of it.
    const CONTROLLER: Pos = Pos::new(0, 0, 0);
    const MIDDLE: Pos = Pos::new(0, 0, 1);

    /// A built engine: an intake on its west face, a drive on its east face.
    fn built() -> HashMap<Pos, Part> {
        let cube = Cube::behind(CONTROLLER, Dir::North);
        let mut parts: HashMap<Pos, Part> = cube
            .area()
            .blocks()
            .map(|pos| (pos, Part::Casing))
            .collect();
        parts.insert(MIDDLE, Part::Chamber);
        parts.insert(CONTROLLER, Part::Controller);
        parts.insert(Pos::new(-1, 0, 1), Part::Intake);
        parts.insert(Pos::new(1, 0, 1), Part::Drive);
        parts
    }

    fn checked(parts: &HashMap<Pos, Part>) -> Result<Layout, Problem> {
        check(CONTROLLER, Dir::North, |pos| parts.get(&pos).copied())
    }

    fn fault(parts: &HashMap<Pos, Part>) -> Option<(Fault, Pos)> {
        checked(parts)
            .err()
            .map(|problem| (problem.fault, problem.at))
    }

    #[test]
    fn a_whole_engine_is_built() {
        let layout = checked(&built()).expect("built");
        assert_eq!(layout.intakes, [Pos::new(-1, 0, 1)], "its intake");
        assert_eq!(layout.drives, [Pos::new(1, 0, 1)], "its drive");
        assert_eq!(layout.casings.len(), 27 - 4, "and the casing around");
    }

    #[test]
    fn ports_go_anywhere_but_the_front_and_the_top() {
        let mut parts = built();
        parts.insert(Pos::new(1, -1, 2), Part::Intake);
        parts.insert(Pos::new(0, -1, 1), Part::Drive);
        assert!(checked(&parts).is_ok(), "on a bottom corner and underneath");
        let front = Pos::new(1, -1, 0);
        parts.insert(front, Part::Drive);
        assert_eq!(
            fault(&parts),
            Some((Fault::Misplaced, front)),
            "on the front face"
        );
        parts.insert(front, Part::Casing);
        let top = Pos::new(-1, 1, 1);
        parts.insert(top, Part::Intake);
        assert_eq!(fault(&parts), Some((Fault::Misplaced, top)), "on top");
    }

    #[test]
    fn what_is_wrong_is_said_with_where() {
        let mut parts = built();
        let gap = Pos::new(1, 1, 2);
        parts.remove(&gap);
        assert_eq!(fault(&parts), Some((Fault::NotPart, gap)), "a gap");
        let mut parts = built();
        parts.insert(MIDDLE, Part::Casing);
        assert_eq!(
            fault(&parts),
            Some((Fault::NoChamber, MIDDLE)),
            "no chamber"
        );
        let mut parts = built();
        let second = Pos::new(0, 1, 1);
        parts.insert(second, Part::Chamber);
        assert_eq!(fault(&parts), Some((Fault::Stray, second)), "two chambers");
        let mut parts = built();
        parts.insert(Pos::new(1, 0, 1), Part::Casing);
        assert_eq!(fault(&parts), Some((Fault::NoDrive, MIDDLE)), "no drive");
        parts.insert(Pos::new(-1, 0, 1), Part::Casing);
        assert_eq!(
            fault(&parts),
            Some((Fault::NoIntake, MIDDLE)),
            "no intake either"
        );
    }

    #[test]
    fn a_corner_faces_out_on_three_sides() {
        let cube = Cube::behind(CONTROLLER, Dir::North);
        let corner: Vec<Dir> = cube.outward(Pos::new(1, -1, 2)).collect();
        assert_eq!(
            corner,
            [Dir::East, Dir::Down, Dir::South],
            "east, down and south"
        );
        assert_eq!(cube.outward(MIDDLE).count(), 0, "the middle faces nowhere");
    }
}
