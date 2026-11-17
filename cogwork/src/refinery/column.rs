//! Whether the blocks around a refinery's controller make the column: a box 3 wide, 3 deep and
//! 6 high behind its front, hollow up the middle of layers 1 to 4, its controller in the middle
//! of the front of the bottom layer (`brickwork`). The rest of the shell is casing, with ports in
//! the casing's place: heaters on the bottom layer, condensers on the top, and taps on the four
//! layers between, each layer's taps for one product (`Product::on_layer`). It needs at least a
//! heater, a condenser and a tap for each product but gas, which vents without one.
//!
//! The world comes in as closures saying which part a block is and whether it is air, so tests
//! reach it.

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::Dir;

use super::products::Product;
use crate::brickwork::{self, Shape};
use crate::lib::Pos;

/// What a refinery column is built of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Casing,
    Controller,
    Heater,
    Condenser,
    Tap,
}

/// Its box: 6 high, the controller at the bottom, hollow up the middle of the four layers of
/// taps.
pub const SHAPE: Shape = Shape {
    height: 6,
    controller_layer: 0,
    hollow_layers: 1..5,
};
/// The layer its condensers are on.
const TOP: i32 = SHAPE.height - 1;

/// The blocks of a built column that do something: its shell, which shows it formed, its heaters
/// and condensers, and its taps by product (`Product::index`).
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Layout {
    pub shell: Vec<Pos>,
    pub heaters: Vec<Pos>,
    pub condensers: Vec<Pos>,
    pub taps: [Vec<Pos>; 4],
}

impl Layout {
    /// The taps of `product`.
    pub fn taps_of(&self, product: Product) -> &[Pos] {
        &self.taps[product.index()]
    }
}

/// What keeps a column from being built; its screen says `cogwork.refinery.<name>`. Kept in
/// `Refinery::fault` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Fault {
    /// The block there is no part of a column.
    #[default]
    NotPart,
    /// A block stands where the hollow is.
    NotHollow,
    /// A second controller in the shell.
    Stray,
    /// A heater off the bottom, a condenser off the top or a tap on either.
    Misplaced,
    NoHeater,
    NoCondenser,
    NoTarTap,
    NoLubricantTap,
    NoDieselTap,
}

impl From<brickwork::Fault> for Fault {
    fn from(fault: brickwork::Fault) -> Fault {
        match fault {
            brickwork::Fault::NotPart => Fault::NotPart,
            brickwork::Fault::NotHollow => Fault::NotHollow,
            brickwork::Fault::Stray => Fault::Stray,
        }
    }
}

/// What keeps a column from being built, and the block where: the middle of the layer it
/// concerns when it is none in particular.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Problem {
    pub fault: Fault,
    pub at: Pos,
}

/// The layout of the column whose controller is at `controller` facing `front`, `part_at`
/// telling what each block is and `is_air` whether it is empty; or the first thing wrong with it.
pub fn check(
    controller: Pos,
    front: Dir,
    part_at: impl Fn(Pos) -> Option<Part>,
    is_air: impl Fn(Pos) -> bool,
) -> Result<Layout, Problem> {
    let shell_part = |pos| part_at(pos).map(as_brick);
    let bricks =
        brickwork::check(controller, front, &SHAPE, shell_part, is_air).map_err(|problem| {
            Problem {
                fault: problem.fault.into(),
                at: problem.at,
            }
        })?;
    let middle = controller.toward(front.opposite());
    let mut layout = Layout {
        shell: bricks.shell,
        ..Layout::default()
    };
    for port in bricks.ports {
        let layer = port.y - controller.y;
        match (part_at(port), Product::on_layer(layer)) {
            (Some(Part::Heater), _) if layer == 0 => layout.heaters.push(port),
            (Some(Part::Condenser), _) if layer == TOP => layout.condensers.push(port),
            (Some(Part::Tap), Some(product)) => layout.taps[product.index()].push(port),
            _ => {
                return Err(Problem {
                    fault: Fault::Misplaced,
                    at: port,
                });
            }
        }
    }
    missing(&layout, middle).map_or(Ok(layout), Err)
}

/// The shell part the brick check sees `part` as: its controller, casing, or a port in the
/// casing's place.
fn as_brick(part: Part) -> brickwork::Part {
    match part {
        Part::Casing => brickwork::Part::Brick,
        Part::Controller => brickwork::Part::Controller,
        Part::Heater | Part::Condenser | Part::Tap => brickwork::Part::Port,
    }
}

/// The first port `layout` lacks, said at the middle of its layer above `middle`.
fn missing(layout: &Layout, middle: Pos) -> Option<Problem> {
    let lacks = |fault: Fault, layer: i32| {
        Some(Problem {
            fault,
            at: middle.add_y(layer),
        })
    };
    if layout.heaters.is_empty() {
        return lacks(Fault::NoHeater, 0);
    }
    if layout.condensers.is_empty() {
        return lacks(Fault::NoCondenser, TOP);
    }
    let needed = [
        (Product::Tar, Fault::NoTarTap),
        (Product::Lubricant, Fault::NoLubricantTap),
        (Product::Diesel, Fault::NoDieselTap),
    ];
    let (product, fault) = needed
        .into_iter()
        .find(|&(product, _)| layout.taps_of(product).is_empty())?;
    lacks(fault, product.layer())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// A controller at the origin facing north, so the column lies south of it.
    const CONTROLLER: Pos = Pos::new(0, 0, 0);
    const MIDDLE: Pos = Pos::new(0, 0, 1);

    /// A built column: a heater and taps on its west face, a condenser in the middle of its top.
    fn built() -> HashMap<Pos, Part> {
        let mut parts = HashMap::new();
        for y in 0..SHAPE.height {
            for x in -1..=1 {
                for z in 0..=2 {
                    let pos = Pos::new(x, y, z);
                    let hollow = x == 0 && z == 1 && SHAPE.hollow_layers.contains(&y);
                    if !hollow {
                        parts.insert(pos, Part::Casing);
                    }
                }
            }
        }
        parts.insert(CONTROLLER, Part::Controller);
        parts.insert(Pos::new(-1, 0, 1), Part::Heater);
        parts.insert(MIDDLE.add_y(TOP), Part::Condenser);
        for layer in 1..=3 {
            parts.insert(Pos::new(-1, layer, 1), Part::Tap);
        }
        parts
    }

    fn checked(parts: &HashMap<Pos, Part>) -> Result<Layout, Problem> {
        let part_at = |pos| parts.get(&pos).copied();
        check(CONTROLLER, Dir::North, part_at, |pos| {
            !parts.contains_key(&pos)
        })
    }

    fn fault(parts: &HashMap<Pos, Part>) -> Option<(Fault, Pos)> {
        checked(parts)
            .err()
            .map(|problem| (problem.fault, problem.at))
    }

    #[test]
    fn a_whole_column_is_built() {
        let layout = checked(&built()).expect("built");
        assert_eq!(layout.shell.len(), 54 - 4, "all but the hollow");
        assert_eq!(layout.heaters, [Pos::new(-1, 0, 1)], "its heater");
        assert_eq!(layout.condensers, [Pos::new(0, 5, 1)], "its condenser");
        assert_eq!(
            layout.taps_of(Product::Tar),
            [Pos::new(-1, 1, 1)],
            "tar lowest"
        );
        assert_eq!(
            layout.taps_of(Product::Diesel),
            [Pos::new(-1, 3, 1)],
            "diesel above"
        );
        assert!(
            layout.taps_of(Product::Gas).is_empty(),
            "no gas tap: it vents"
        );
    }

    #[test]
    fn a_tap_on_the_top_layer_of_taps_is_for_gas() {
        let mut parts = built();
        parts.insert(Pos::new(1, 4, 2), Part::Tap);
        let layout = checked(&parts).expect("built");
        assert_eq!(layout.taps_of(Product::Gas), [Pos::new(1, 4, 2)], "gas");
    }

    #[test]
    fn ports_go_only_on_their_own_layers() {
        for (pos, part) in [
            (Pos::new(1, 2, 1), Part::Heater),
            (Pos::new(1, 0, 2), Part::Condenser),
            (Pos::new(1, 0, 2), Part::Tap),
            (Pos::new(1, 5, 2), Part::Tap),
        ] {
            let mut parts = built();
            parts.insert(pos, part);
            assert_eq!(fault(&parts), Some((Fault::Misplaced, pos)), "{part:?}");
        }
    }

    #[test]
    fn what_is_missing_is_said_on_its_layer() {
        let mut parts = built();
        parts.insert(Pos::new(-1, 2, 1), Part::Casing);
        assert_eq!(
            fault(&parts),
            Some((Fault::NoLubricantTap, MIDDLE.add_y(2))),
            "the lubricant's layer"
        );
        parts.insert(MIDDLE.add_y(TOP), Part::Casing);
        assert_eq!(
            fault(&parts),
            Some((Fault::NoCondenser, MIDDLE.add_y(TOP))),
            "on top"
        );
        parts.insert(Pos::new(-1, 0, 1), Part::Casing);
        assert_eq!(
            fault(&parts),
            Some((Fault::NoHeater, MIDDLE)),
            "at the bottom"
        );
    }

    #[test]
    fn a_gap_or_a_filled_hollow_is_said_with_where() {
        let mut parts = built();
        let gap = Pos::new(1, 4, 0);
        parts.remove(&gap);
        assert_eq!(fault(&parts), Some((Fault::NotPart, gap)), "a gap");
        let mut parts = built();
        let filled = MIDDLE.add_y(2);
        parts.insert(filled, Part::Casing);
        assert_eq!(fault(&parts), Some((Fault::NotHollow, filled)), "filled");
    }

    #[test]
    fn every_brick_fault_has_its_own() {
        for &fault in brickwork::Fault::VARIANTS {
            let ours = Fault::from(fault);
            assert_eq!(<&str>::from(ours), <&str>::from(fault), "{fault:?}");
        }
    }
}
