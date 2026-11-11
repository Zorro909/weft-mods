//! The liquid tank: a glass block on a riveted base holding sixteen buckets of one liquid. Tank
//! blocks next to each other become one tank when together they fill a box up to 3 by 3 and 6
//! high (`merge`); its liquid lies from the bottom up, at one level across the tank. A broken
//! block's share is lost, and what is left becomes tanks again.
//!
//! Each block keeps its own share in its `HELD`, so nothing is kept for the tank as a whole: the
//! blocks of one tank are those its `true` side properties join, and a tank's liquid is theirs
//! together. Its shape is the Blockbench model of its joined sides,
//! `assets/models/block/liquid_tank/<sides>.json`, written by `files`.

#[cfg(test)]
mod files;
pub mod merge;

use std::collections::HashSet;

use merge::{MOST, forms_box, spread};
use weft_sdk::*;

use crate::fill::{self, TANK_LEVELS};
use crate::lib::{Pos, TypedWorld};
use crate::liquid::{HELD, Held, Holders, NO_LIQUID, Vessel, use_bucket};

pub static LIQUID_TANK: Block = Block::named("liquid_tank");

/// Millibuckets one tank block holds.
pub const CELL_CAPACITY: u32 = 16_000;

pub fn register(r: &mut Registry) {
    let mut builder = r
        .block(&LIQUID_TANK)
        .hardness(2.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.glass")
        .tab(&crate::TAB);
    for dir in Dir::ALL {
        builder = builder.prop(dir.name(), ["false", "true"]);
    }
    builder
        .states(|props, state| {
            let joined = |dir: Dir| props.is(dir.name(), "true");
            if Dir::ALL.into_iter().all(joined) {
                // Inside the tank, never seen: a whole block to stand on, and nothing drawn.
                let unseen = [("liquid_tank_frame", abi::UV_NO_FACE); 6];
                return state.model_box_faces([0.0; 3], [16.0; 3], unseen);
            }
            state.model_file(&crate::arms::model("liquid_tank", joined))
        })
        // Its block entity, which holds its share, is there from the next tick.
        .on_place(|world, change| world.schedule_tick(change.pos, 1))
        .on_neighbor(neighbor_changed)
        .on_scheduled(|event| regroup(&event.world(), event.pos().into()))
        .on_break(|world, change| broken(world, change.pos.into()))
        .on_use(use_tank)
        .entity(|entity| entity.attach(&HELD));
}

/// One tank: its blocks.
pub struct Tank {
    cells: Vec<Cell>,
}

#[derive(Clone, Copy)]
struct Cell {
    id: u64,
    pos: Pos,
}

impl Tank {
    /// The tank the block at `pos` is part of: it and every block its sides join, where the
    /// other side joins back. None while a block of it has no block entity yet.
    pub fn at(world: &World, pos: Pos) -> Option<Tank> {
        let joined = |cell: Pos, dir: Dir| world.block_typed(cell).prop(dir.name()) == Some("true");
        let both =
            |cell: Pos, dir: Dir| joined(cell, dir) && joined(cell.toward(dir), dir.opposite());
        let mut cells = Vec::new();
        for cell in flood(pos, MOST, both) {
            let id = world.block_entity(cell);
            if id == 0 {
                return None;
            }
            cells.push(Cell { id, pos: cell });
        }
        Some(Tank { cells })
    }

    /// Its liquid, all its blocks' shares together.
    pub fn held(&self, holders: &Holders) -> Held {
        together(self.cells.iter().map(|cell| holders.get(cell.id)))
    }

    pub fn capacity(&self) -> u32 {
        CELL_CAPACITY * self.cells.len() as u32
    }

    /// Lays `held` in the tank from the bottom up and shows each block's fill.
    pub fn settle(&self, holders: &mut Holders, held: Held) {
        let mut cells = self.cells.clone();
        cells.sort_by_key(|cell| (cell.pos.y, cell.pos.x, cell.pos.z));
        let mut layers: Vec<usize> = Vec::new();
        for layer in cells.chunk_by(|a, b| a.pos.y == b.pos.y) {
            layers.push(layer.len());
        }
        let shares = spread(&layers, held.amount, CELL_CAPACITY);
        let world = *holders.world();
        for (cell, amount) in cells.into_iter().zip(shares) {
            let mut share = holders.get(cell.id);
            share.amount = amount;
            share.liquid = if amount == 0 { NO_LIQUID } else { held.liquid };
            show(&world, cell.pos, &mut share);
            holders.set(cell.id, share);
        }
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
        shown: 0,
    }
}

/// The tank blocks touching `start`, it among them: all of them, however many, so that when
/// there are too many for one tank every one of them is parted from the others.
fn touching(start: Pos, is_tank: impl Fn(Pos) -> bool) -> Vec<Pos> {
    flood(start, usize::MAX, |cell, dir| is_tank(cell.toward(dir)))
}

/// The blocks reached from `start` over the sides `goes_on` says, `start` first, no more than
/// `most` and one past.
fn flood(start: Pos, most: usize, goes_on: impl Fn(Pos, Dir) -> bool) -> Vec<Pos> {
    let mut found = vec![start];
    let mut seen = HashSet::from([start]);
    let mut next = 0;
    while next < found.len() && found.len() <= most {
        let cell = found[next];
        next += 1;
        for dir in Dir::ALL.into_iter().filter(|&dir| goes_on(cell, dir)) {
            if seen.insert(cell.toward(dir)) {
                found.push(cell.toward(dir));
            }
        }
    }
    found
}

/// Makes the tank blocks touching `start` one tank where they may be, and each its own tank
/// where they may not, then lays each tank's liquid out again.
fn regroup(world: &World, start: Pos) {
    let is_tank = |cell: Pos| world.block_typed(cell).is(&LIQUID_TANK);
    if !is_tank(start) {
        return;
    }
    let cells = touching(start, is_tank);
    let mut holders = Holders::outside(*world);
    let merged = forms_box(&cells) && one_liquid(world, &holders, &cells);
    for &cell in &cells {
        let joins = |dir: Dir| merged && cells.contains(&cell.toward(dir));
        let block = world.block_typed(cell);
        let next = Dir::ALL
            .into_iter()
            .fold(block, |next, dir| next.with_flag(dir.name(), joins(dir)));
        if next != block {
            world.set_block_typed(cell, next);
        }
    }
    let starts = if merged { &cells[..1] } else { &cells[..] };
    for &cell in starts {
        if let Some(tank) = Tank::at(world, cell) {
            let held = tank.held(&holders);
            tank.settle(&mut holders, held);
        }
    }
}

/// Whether the blocks at `cells` hold no more than one liquid between them.
fn one_liquid(world: &World, holders: &Holders, cells: &[Pos]) -> bool {
    let mut liquids = cells
        .iter()
        .map(|&cell| holders.get(world.block_entity(cell)))
        .filter(|share| !share.is_empty_in(CELL_CAPACITY))
        .map(|share| share.liquid);
    let first = liquids.next();
    liquids.all(|liquid| Some(liquid) == first)
}

/// A block beside changed. When a tank block came there or went, by a player or any other way
/// (a command, another mod), the tanks regroup.
fn neighbor_changed(event: &mut BlockEvent) {
    let world = event.world();
    let (pos, beside) = (Pos::from(event.pos()), Pos::from(event.neighbor()));
    let Some(dir) = Dir::ALL.into_iter().find(|&dir| pos.toward(dir) == beside) else {
        return;
    };
    let joined = world.block_typed(pos).prop(dir.name()) == Some("true");
    if joined != world.block_typed(beside).is(&LIQUID_TANK) {
        world.schedule_tick(pos, 1);
    }
}

/// A tank block is gone: its fill with it, and the tank blocks around it regroup.
fn broken(world: &World, pos: Pos) {
    fill::show_in_tank(world, pos, None);
    for dir in Dir::ALL {
        let next = pos.toward(dir);
        if world.block_typed(next).is(&LIQUID_TANK) {
            world.schedule_tick(next, 1);
        }
    }
}

/// A bucket used on a tank pours into it or fills from it.
fn use_tank(click: &mut Use) -> bool {
    let world = click.world();
    let Some(tank) = Tank::at(&world, click.pos().into()) else {
        return false;
    };
    let mut holders = Holders::outside(world);
    use_bucket(&mut holders, &Vessel::Tank(tank), click.player())
}

/// Shows how full the tank block at `pos` is, when that or its joined sides changed since
/// `share.shown`.
fn show(world: &World, pos: Pos, share: &mut Held) {
    let block = world.block_typed(pos);
    let joined = |dir: Dir| block.prop(dir.name()) == Some("true");
    let level = fill::level_of(share.amount, CELL_CAPACITY, TANK_LEVELS);
    let shown = shown_as(level, joined);
    if shown == share.shown {
        return;
    }
    share.shown = shown;
    let filled = (level > 0)
        .then(|| fill::in_tank(share.liquid, level, joined))
        .flatten();
    fill::show_in_tank(world, pos, filled);
}

/// Bits a fill's level takes in `Held::shown`, below those of its joined sides.
const LEVEL_BITS: u32 = TANK_LEVELS.ilog2() + 1;

/// What a fill of `level` joined toward `joined` is kept as in `Held::shown`: 0 for none.
fn shown_as(level: u32, joined: impl Fn(Dir) -> bool) -> u32 {
    if level == 0 {
        return 0;
    }
    let sides = Dir::ALL
        .into_iter()
        .filter(|&dir| joined(dir))
        .fold(0, |sides, dir| sides | 1 << dir.face());
    level | sides << LEVEL_BITS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flood_stops_one_past_the_most() {
        let line = flood(Pos { x: 0, y: 0, z: 0 }, 4, |_, dir| dir == Dir::East);
        assert_eq!(
            line.len(),
            5,
            "the most and one more, to tell it is too many"
        );
        let alone = flood(Pos { x: 0, y: 0, z: 0 }, 4, |_, _| false);
        assert_eq!(alone.len(), 1, "a block on its own");
    }

    const WATER: u64 = 7;
    const OIL: u64 = 8;

    fn held(liquid: u64, amount: u32) -> Held {
        Held {
            liquid,
            amount,
            shown: 0,
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

    #[test]
    fn a_regroup_finds_every_touching_tank_block() {
        // Two whole tanks of 54 and the block between them: none of them may stay joined.
        let row = |pos: Pos| (0..109).contains(&pos.x) && pos.y == 0 && pos.z == 0;
        let found = touching(Pos { x: 54, y: 0, z: 0 }, row);
        assert_eq!(found.len(), 109, "all of them, more than a tank has");
    }

    #[test]
    fn a_fill_is_shown_anew_when_its_level_or_sides_change() {
        let full = shown_as(TANK_LEVELS, |_| false);
        assert_ne!(full, shown_as(TANK_LEVELS - 1, |_| false), "a level less");
        assert_ne!(
            full,
            shown_as(TANK_LEVELS, |dir| dir == Dir::Up),
            "joined above"
        );
        assert_eq!(shown_as(0, |_| true), 0, "nothing shown");
    }
}
