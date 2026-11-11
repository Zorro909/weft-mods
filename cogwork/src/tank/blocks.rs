//! One tank as its blocks: what they hold together, how its liquid lies in them from the bottom
//! up, and the fill each block shows.
//!
//! A `Tank` keeps what it read of the world when it was made: its blocks, the sides each joins,
//! and its liquid once that was asked for. So it is good for as long as its blocks stay as they
//! are and its liquid changes only through its own `settle`: for one event, or one tick of the
//! holders.

use std::cell::Cell as Kept;
use std::sync::OnceLock;

use weft_sdk::{Dir, World};

use super::merge::spread;
use super::{CELL_CAPACITY, LIQUID_TANK, joined_cells};
use crate::fill::{self, TANK_LEVELS};
use crate::lib::{BlockState, Pos, TypedWorld};
use crate::liquid::{DRIBBLE, Held, Holders, NO_LIQUID};

/// One tank: its blocks.
pub struct Tank {
    /// From the bottom layer up.
    cells: Vec<Cell>,
    /// How many blocks each layer has, the bottom one first.
    layers: Vec<usize>,
    /// For each block, which of `cells` lies on it, if one does.
    above: Vec<Option<usize>>,
    /// Its liquid, all its blocks' shares together, once worked out.
    all: Kept<Option<Held>>,
}

#[derive(Clone, Copy)]
struct Cell {
    id: u64,
    pos: Pos,
    /// The sides it joins more of its tank on (`joined_sides`).
    joined: u32,
}

impl Tank {
    /// The tank the block at `pos` is part of: it and every block its sides join, where the
    /// other side joins back. None while a block of it has no block entity yet.
    pub fn at(world: &World, pos: Pos) -> Option<Tank> {
        let tank = Tank::of(world, &joined_cells(world, pos));
        tank.cells.iter().all(|cell| cell.id != 0).then_some(tank)
    }

    /// The tank of the blocks at `cells`.
    pub(super) fn of(world: &World, cells: &[Pos]) -> Tank {
        let cell = |&pos: &Pos| Cell {
            id: world.block_entity(pos),
            pos,
            joined: joined_sides(world.block_typed(pos)),
        };
        let mut cells: Vec<Cell> = cells.iter().map(cell).collect();
        cells.sort_by_key(|cell| (cell.pos.y, cell.pos.x, cell.pos.z));
        let layers = cells.chunk_by(|a, b| a.pos.y == b.pos.y);
        let on = |below: &Cell| {
            let above = below.pos.toward(Dir::Up);
            cells.iter().position(|cell| cell.pos == above)
        };
        Tank {
            layers: layers.map(<[Cell]>::len).collect(),
            above: cells.iter().map(on).collect(),
            cells,
            all: Kept::new(None),
        }
    }

    /// Its liquid, all its blocks' shares together.
    pub fn held(&self, holders: &Holders) -> Held {
        if let Some(all) = self.all.get() {
            return all;
        }
        let all = together(self.cells.iter().map(|cell| holders.get(cell.id)));
        self.all.set(Some(all));
        all
    }

    pub fn capacity(&self) -> u32 {
        CELL_CAPACITY * self.cells.len() as u32
    }

    /// How high its bottom layer stands.
    pub fn floor(&self) -> i32 {
        self.cells.first().map_or(0, |cell| cell.pos.y)
    }

    /// Millibuckets that raise its liquid a block: a layer's worth.
    pub fn per_block(&self) -> u32 {
        CELL_CAPACITY * self.layers.first().map_or(0, |&bottom| bottom as u32)
    }

    /// Its blocks, and their block entities in the same order.
    pub fn blocks(&self) -> impl Iterator<Item = Pos> + '_ {
        self.cells.iter().map(|cell| cell.pos)
    }

    pub fn ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.cells.iter().map(|cell| cell.id)
    }

    /// Every side of its blocks with no more of it there, where it may touch another holder,
    /// by the block it is on.
    pub fn open_sides(&self) -> impl Iterator<Item = (Pos, Dir)> + '_ {
        let open = |cell: &Cell, dir: &Dir| cell.joined & side(*dir) == 0;
        let of_cell = move |cell: &Cell| {
            let cell = *cell;
            let sides = Dir::ALL.into_iter().filter(move |dir| open(&cell, dir));
            sides.map(move |dir| (cell.pos, dir))
        };
        self.cells.iter().flat_map(of_cell)
    }

    /// Lays `held` in the tank from the bottom up and shows each block's fill.
    pub fn settle(&self, holders: &mut Holders, held: Held) {
        let shares = spread(&self.layers, held.amount, CELL_CAPACITY);
        let world = *holders.world();
        for (index, (cell, &amount)) in self.cells.iter().zip(&shares).enumerate() {
            let mut share = holders.get(cell.id);
            share.amount = amount;
            share.liquid = if amount == 0 { NO_LIQUID } else { held.liquid };
            // Its liquid is laid out anew: every block takes a turn again.
            share.asleep = 0;
            let covered = self.above[index].is_some_and(|above| shares[above] >= DRIBBLE);
            show(&world, cell, &mut share, covered);
            holders.set(cell.id, share);
        }
        self.all.set(Some(laid(held, self.capacity())));
    }
}

/// The liquid of a tank whose blocks hold `shares`: the liquid of its biggest share, all of that
/// liquid there is. A block joined while it held a dribble of another liquid loses it.
fn together(shares: impl IntoIterator<Item = Held>) -> Held {
    let shares: Vec<Held> = shares.into_iter().collect();
    let Some(biggest) = shares.iter().max_by_key(|share| share.amount) else {
        return Held::default();
    };
    let liquid = biggest.liquid;
    let of_it = shares.iter().filter(|share| share.liquid == liquid);
    Held {
        liquid,
        amount: of_it.map(|share| share.amount).sum(),
        ..Held::default()
    }
}

/// What the blocks of a tank of `capacity` hold `together` once `held` is laid out in them: as
/// much of it as fits.
fn laid(held: Held, capacity: u32) -> Held {
    let amount = held.amount.min(capacity);
    Held {
        liquid: if amount == 0 { NO_LIQUID } else { held.liquid },
        amount,
        ..Held::default()
    }
}

/// Shows how full the tank block `cell` is, when that or its joined sides changed since
/// `share.shown`. It is `covered` by liquid in the block above.
fn show(world: &World, cell: &Cell, share: &mut Held, covered: bool) {
    let joined = |dir: Dir| cell.joined & side(dir) != 0;
    let mut level = fill::level_of(share.amount, CELL_CAPACITY, TANK_LEVELS);
    if level == TANK_LEVELS && joined(Dir::Up) && !covered {
        // A full block joined above shows no top: liquid goes on up. With none above, a step
        // short of full shows its surface.
        level -= 1;
    }
    let shown = shown_as(level, cell.joined);
    if shown == share.shown {
        return;
    }
    share.shown = shown;
    let filled = (level > 0)
        .then(|| fill::in_tank(share.liquid, level, joined))
        .flatten();
    fill::show_in_tank(world, cell.pos, filled);
}

/// The bit of the side `dir` among a block's sides.
fn side(dir: Dir) -> u32 {
    1 << dir.face()
}

/// Whether the block `block` is a tank block joined to more of its tank toward `dir`.
pub(super) fn joins(block: BlockState, dir: Dir) -> bool {
    joined_sides(block) & side(dir) != 0
}

/// The sides the tank block `block` joins more of its tank on, a bit for each (`side`); none for
/// another block. A tick asks this of every block of every tank, so the properties of each
/// state of the tank block are read once and kept.
fn joined_sides(block: BlockState) -> u32 {
    static OF_STATE: OnceLock<Vec<u32>> = OnceLock::new();
    if !block.is(&LIQUID_TANK) {
        return 0;
    }
    let first = LIQUID_TANK.id();
    let read = |state: BlockState| sides_of(|dir| state.prop(dir.name()) == Some("true"));
    let of_state = OF_STATE.get_or_init(|| {
        let states = (first..).map(BlockState::from);
        let of_tank = states.take_while(|state| state.is(&LIQUID_TANK));
        of_tank.map(read).collect()
    });
    of_state[(block.id() - first) as usize]
}

/// The sides `joined` says, a bit for each (`side`).
fn sides_of(joined: impl Fn(Dir) -> bool) -> u32 {
    let joined = |dir: &Dir| joined(*dir);
    Dir::ALL.into_iter().filter(joined).map(side).sum()
}

/// Bits a fill's level takes in `Held::shown`, below those of its joined sides.
const LEVEL_BITS: u32 = TANK_LEVELS.ilog2() + 1;

/// What a fill of `level` in a block joined on `sides` is kept as in `Held::shown`: 0 for none.
fn shown_as(level: u32, sides: u32) -> u32 {
    if level == 0 {
        return 0;
    }
    level | sides << LEVEL_BITS
}

#[cfg(test)]
mod tests {
    use super::*;

    const WATER: u64 = 7;
    const OIL: u64 = 8;

    fn held(liquid: u64, amount: u32) -> Held {
        Held {
            liquid,
            amount,
            ..Held::default()
        }
    }

    #[test]
    fn a_dribble_of_another_liquid_is_not_the_tanks_liquid() {
        let shares = [held(WATER, 5), held(NO_LIQUID, 0), held(OIL, 8000)];
        let all = together(shares);
        assert_eq!(
            (all.liquid, all.amount),
            (OIL, 8000),
            "oil, the water's dribble gone"
        );
    }

    /// What `together` makes of the shares `held` is laid out as in two layers of two blocks.
    fn together_once_laid(held: Held) -> Held {
        let shares = spread(&[2, 2], held.amount, CELL_CAPACITY);
        let share = |amount: u32| Held {
            liquid: if amount == 0 { NO_LIQUID } else { held.liquid },
            amount,
            ..Held::default()
        };
        together(shares.into_iter().map(share))
    }

    #[test]
    fn a_tank_holds_what_was_laid_out_in_it() {
        let capacity = 4 * CELL_CAPACITY;
        for amount in [0, 1, 5, CELL_CAPACITY, 2 * CELL_CAPACITY + 3, capacity] {
            // Woken and with a head, as a holder's liquid is when liquid came into it.
            let coming = Held {
                head: 3,
                shown: 4,
                ..held(WATER, amount)
            };
            let kept = laid(coming, capacity);
            assert_eq!(kept, together_once_laid(coming), "{amount} mB");
        }
        let too_much = laid(held(OIL, capacity + 9), capacity);
        assert_eq!(too_much, held(OIL, capacity), "no more than fits");
        assert_eq!(too_much, together_once_laid(held(OIL, capacity + 9)));
    }

    #[test]
    fn a_fill_is_shown_anew_when_its_level_or_sides_change() {
        let full = shown_as(TANK_LEVELS, 0);
        assert_ne!(full, shown_as(TANK_LEVELS - 1, 0), "a level less");
        let joined_above = sides_of(|dir| dir == Dir::Up);
        assert_ne!(full, shown_as(TANK_LEVELS, joined_above), "joined above");
        assert_eq!(shown_as(0, sides_of(|_| true)), 0, "nothing shown");
    }

    #[test]
    fn every_set_of_joined_sides_is_kept_apart() {
        let all = sides_of(|_| true);
        assert_eq!(all.count_ones(), 6, "a bit a side");
        for dir in Dir::ALL {
            let but_one = sides_of(|side| side != dir);
            assert_eq!(but_one | side(dir), all, "all but {}", dir.name());
            assert_eq!(but_one & side(dir), 0, "without its bit");
        }
    }
}
