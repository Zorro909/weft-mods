//! Whether the blocks around a brick machine's controller make the machine: a box three blocks
//! wide and three deep behind its front, of one kind of brick around a hollow, its controller in
//! the middle of the front face at a layer its `Shape` says. The coke oven and the blast furnace
//! are built this way. A machine may let ports (the coke oven's drains) take the place of
//! bricks anywhere in the shell.
//!
//! The world comes in as closures saying which part a block is and whether it is air, so tests
//! reach it.

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::Dir;

use crate::lib::Pos;

/// What a brick machine is built of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Brick,
    Controller,
    Port,
}

/// A brick machine's box: how many layers high it is, the layer its controller is on (0 the
/// bottom), and its hollow, by layer, in the middle column of the box.
pub struct Shape {
    pub height: i32,
    pub controller_layer: i32,
    pub hollow_layers: std::ops::Range<i32>,
}

/// A machine's box, from its controller and the way that faces.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Box3 {
    /// The middle of its bottom layer.
    floor: Pos,
    front: Dir,
}

impl Box3 {
    fn behind(controller: Pos, front: Dir, shape: &Shape) -> Box3 {
        let middle = controller.toward(front.opposite());
        Box3 {
            floor: middle.add_y(-shape.controller_layer),
            front,
        }
    }

    /// Every block of it, layer by layer from the bottom.
    fn blocks(self, height: i32) -> impl Iterator<Item = Pos> {
        let side = self.front.clockwise();
        (0..height).flat_map(move |layer| {
            let middle = self.floor.add_y(layer);
            [-1, 0, 1].into_iter().flat_map(move |depth| {
                let row = middle.toward_by(self.front, depth);
                [-1, 0, 1].map(|across| row.toward_by(side, across))
            })
        })
    }

    /// Whether `pos` is in its middle column, on a layer of `layers`.
    fn in_middle(self, pos: Pos, layers: &std::ops::Range<i32>) -> bool {
        let layer = pos.y - self.floor.y;
        pos.x == self.floor.x && pos.z == self.floor.z && layers.contains(&layer)
    }
}

/// The blocks of a built machine: its shell (bricks, ports and controller), its ports, and its
/// hollow.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Layout {
    pub shell: Vec<Pos>,
    pub ports: Vec<Pos>,
    pub hollow: Vec<Pos>,
}

impl Layout {
    /// Its shell blocks on its bottom layer.
    pub fn bottom(&self) -> impl Iterator<Item = Pos> + '_ {
        let floor = self.shell.iter().map(|pos| pos.y).min().unwrap_or_default();
        self.shell.iter().copied().filter(move |pos| pos.y == floor)
    }
}

/// What keeps a brick machine from being built; its screen says `cogwork.brickwork.<name>`.
/// Kept in a machine's state as its number, so the order is part of the save format: new ones
/// go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Fault {
    /// The block there is not this machine's brick.
    #[default]
    NotPart,
    /// A block stands where the hollow is.
    NotHollow,
    /// A second controller in the shell.
    Stray,
}

/// What keeps a machine from being built, and the block where.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Problem {
    pub fault: Fault,
    pub at: Pos,
}

/// The layout of the machine of `shape` whose controller is at `controller` facing `front`,
/// `part_at` telling what each block is and `is_air` whether it is empty; or the first thing
/// wrong with it.
pub fn check(
    controller: Pos,
    front: Dir,
    shape: &Shape,
    part_at: impl Fn(Pos) -> Option<Part>,
    is_air: impl Fn(Pos) -> bool,
) -> Result<Layout, Problem> {
    let box3 = Box3::behind(controller, front, shape);
    let mut layout = Layout::default();
    let wrong = |fault: Fault, at: Pos| Err(Problem { fault, at });
    for pos in box3.blocks(shape.height) {
        if box3.in_middle(pos, &shape.hollow_layers) {
            if !is_air(pos) {
                return wrong(Fault::NotHollow, pos);
            }
            layout.hollow.push(pos);
            continue;
        }
        match part_at(pos) {
            None => return wrong(Fault::NotPart, pos),
            Some(Part::Controller) if pos != controller => return wrong(Fault::Stray, pos),
            Some(Part::Port) => layout.ports.push(pos),
            Some(Part::Brick | Part::Controller) => {}
        }
        layout.shell.push(pos);
    }
    Ok(layout)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// A coke oven's: 3 high, controller in the middle layer, the middle block hollow.
    const OVEN: Shape = Shape {
        height: 3,
        controller_layer: 1,
        hollow_layers: 1..2,
    };
    /// A blast furnace's: 4 high, controller at the bottom, hollow above the hearth.
    const FURNACE: Shape = Shape {
        height: 4,
        controller_layer: 0,
        hollow_layers: 1..4,
    };
    /// A controller at the origin facing north, so the box lies south of it.
    const CONTROLLER: Pos = Pos::new(0, 0, 0);

    /// A built machine of `shape`: bricks round the hollow, nothing in it.
    fn built(shape: &Shape) -> HashMap<Pos, Part> {
        let box3 = Box3::behind(CONTROLLER, Dir::North, shape);
        let mut parts: HashMap<Pos, Part> = box3
            .blocks(shape.height)
            .filter(|&pos| !box3.in_middle(pos, &shape.hollow_layers))
            .map(|pos| (pos, Part::Brick))
            .collect();
        parts.insert(CONTROLLER, Part::Controller);
        parts
    }

    fn checked(parts: &HashMap<Pos, Part>, shape: &Shape) -> Result<Layout, Problem> {
        let part_at = |pos| parts.get(&pos).copied();
        check(CONTROLLER, Dir::North, shape, part_at, |pos| {
            !parts.contains_key(&pos)
        })
    }

    #[test]
    fn a_whole_coke_oven_is_built_round_one_hollow_block() {
        let layout = checked(&built(&OVEN), &OVEN).expect("built");
        assert_eq!(layout.hollow, [Pos::new(0, 0, 1)], "behind the controller");
        assert_eq!(layout.shell.len(), 26, "the rest of the 3 by 3 by 3");
        assert!(layout.ports.is_empty(), "no drain");
    }

    #[test]
    fn a_blast_furnace_is_hollow_above_its_hearth() {
        let layout = checked(&built(&FURNACE), &FURNACE).expect("built");
        let hollow: Vec<i32> = layout.hollow.iter().map(|pos| pos.y).collect();
        assert_eq!(hollow, [1, 2, 3], "three blocks up its middle");
        assert_eq!(layout.shell.len(), 33, "36 less the hollow");
        assert_eq!(
            layout.bottom().count(),
            9,
            "the hearth and the ring round it"
        );
    }

    #[test]
    fn a_port_takes_the_place_of_a_brick() {
        let mut parts = built(&OVEN);
        let drain = Pos::new(1, -1, 2);
        parts.insert(drain, Part::Port);
        let layout = checked(&parts, &OVEN).expect("built");
        assert_eq!(layout.ports, [drain], "its drain");
    }

    #[test]
    fn what_is_wrong_is_said_with_where() {
        let mut parts = built(&OVEN);
        let gap = Pos::new(-1, 1, 2);
        parts.remove(&gap);
        let fault = checked(&parts, &OVEN).err();
        // The gap is air, but not part of the shell.
        assert_eq!(
            fault,
            Some(Problem {
                fault: Fault::NotPart,
                at: gap
            }),
            "a gap"
        );
        let mut parts = built(&OVEN);
        let second = Pos::new(1, 0, 1);
        parts.insert(second, Part::Controller);
        let fault = checked(&parts, &OVEN).map_err(|problem| problem.fault);
        assert_eq!(fault, Err(Fault::Stray), "two controllers");
    }

    #[test]
    fn a_filled_hollow_is_not_built() {
        let parts = built(&OVEN);
        let filled = Pos::new(0, 0, 1);
        let fault = check(
            CONTROLLER,
            Dir::North,
            &OVEN,
            |pos| parts.get(&pos).copied(),
            |pos| pos != filled && !parts.contains_key(&pos),
        );
        assert_eq!(
            fault.map_err(|problem| problem.fault),
            Err(Fault::NotHollow),
            "a block inside"
        );
    }

    #[test]
    fn the_box_lies_behind_a_controller_facing_any_way() {
        for front in [Dir::North, Dir::East, Dir::South, Dir::West] {
            let box3 = Box3::behind(CONTROLLER, front, &OVEN);
            let blocks: Vec<Pos> = box3.blocks(OVEN.height).collect();
            assert_eq!(blocks.len(), 27, "facing {front:?}");
            assert!(blocks.contains(&CONTROLLER), "the controller in its face");
            let behind = CONTROLLER.toward_by(front.opposite(), 3);
            assert!(!blocks.contains(&behind), "three behind is outside");
        }
    }
}
