//! Where a water wheel meets the water, and how fast it turns.

use weft_sdk::Dir;

use crate::lib::Pos;

/// How a wheel turns.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Speed {
    /// No flowing water at its foot.
    Still,
    /// The water turns it, but its store is full: it only creeps round.
    Creep,
    /// Running: its energy goes in.
    Turn,
}

impl Speed {
    /// The speed of a wheel with water `flowing` at its foot, whose store `took` what it made.
    pub fn of(flowing: bool, took: bool) -> Speed {
        match (flowing, took) {
            (false, _) => Speed::Still,
            (true, false) => Speed::Creep,
            (true, true) => Speed::Turn,
        }
    }
}

/// The three cells under the wheel on the hub at `pos`, whose axle runs toward `out`: its lower
/// paddles dip into them.
pub fn lower_cells(pos: Pos, out: Dir) -> [Pos; 3] {
    let below = pos.add_y(-1);
    let across = out.clockwise();
    [below.toward(across.opposite()), below, below.toward(across)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wheel_dips_into_the_row_across_its_axle_below_it() {
        let cells = lower_cells(Pos::new(0, 5, 0), Dir::North);
        assert_eq!(
            cells,
            [Pos::new(-1, 4, 0), Pos::new(0, 4, 0), Pos::new(1, 4, 0)],
            "an axle along z: a row along x"
        );
        let cells = lower_cells(Pos::new(0, 5, 0), Dir::East);
        assert!(
            cells.iter().all(|cell| cell.x == 0 && cell.y == 4),
            "along z"
        );
    }

    #[test]
    fn it_turns_fast_only_while_its_energy_goes_in() {
        assert_eq!(Speed::of(false, false), Speed::Still, "no water");
        assert_eq!(Speed::of(true, false), Speed::Creep, "full");
        assert_eq!(Speed::of(true, true), Speed::Turn, "running");
    }
}
