//! The liquid tank: a glass block on a riveted base holding sixteen buckets of one liquid. A tank
//! block placed beside tanks joins them where together they fill a box up to 3 by 3 and 6 high,
//! and is a tank of its own where not (`merge`); a tank's liquid lies from the bottom up, at one
//! level across it. A broken block's share goes to the rest of its tank first, which parts into
//! tanks that fill boxes again, each with its share of the liquid by its size.
//!
//! Each block keeps its own share in its `HELD`, so nothing is kept for the tank as a whole: the
//! blocks of one tank are those its `true` side properties join, and a tank's liquid is theirs
//! together (`blocks`). Its shape follows its joined sides (`shape`).

mod blocks;
pub mod merge;
mod shape;

pub use blocks::Tank;
use blocks::joins;
use merge::{MOST, forms_box, grow, part};
pub use shape::{BASE, GLASS};
use weft_sdk::{Block, BlockEvent, Dir, Registry, Use, Verdict, World, abi, actions, tool_tier};

use crate::fill;
use crate::lib::{Pos, TypedWorld};
use crate::liquid::{HELD, Held, Holders, Vessel, use_bucket};

pub static LIQUID_TANK: Block = Block::named("liquid_tank");

/// Millibuckets one tank block holds.
pub const CELL_CAPACITY: u32 = 16_000;

pub fn register(r: &mut Registry) {
    r.before_event(&actions::BREAK_BLOCK, |ask| {
        before_break(&ask.world(), ask.pos().into());
        Verdict::Abstain
    });
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
            // A whole block to stand on and aim at, whatever its pieces are.
            let state = state
                .texture("liquid_tank_frame")
                .hitbox([0.0; 3], [16.0; 3]);
            if Dir::ALL.into_iter().all(joined) {
                // Inside the tank, never seen: a whole block to stand on, and nothing drawn.
                let unseen = [("liquid_tank_frame", abi::UV_NO_FACE); 6];
                return state.model_box_faces([0.0; 3], [16.0; 3], unseen);
            }
            crate::boxes::draw(state, &shape::pieces(joined))
        })
        // Its block entity, which holds its share, is there from the next tick.
        .on_place(|world, change| world.schedule_tick(change.pos, 1))
        .on_neighbor(neighbor_changed)
        .on_scheduled(|event| regroup(&event.world(), event.pos().into()))
        .on_break(|world, change| broken(world, change.pos.into()))
        .on_use(use_tank)
        .entity(|entity| entity.attach(&HELD));
}

/// The blocks of the tank the block at `pos` is part of: it and every block its sides join,
/// where the other side joins back.
fn joined_cells(world: &World, pos: Pos) -> Vec<Pos> {
    let joined = |cell: Pos, dir: Dir| joins(world.block_typed(cell), dir);
    let both = |cell: Pos, dir: Dir| joined(cell, dir) && joined(cell.toward(dir), dir.opposite());
    flood(pos, MOST, both)
}

/// The blocks reached from `start` over the sides `goes_on` says, `start` first, no more than
/// `most` and one past.
fn flood(start: Pos, most: usize, goes_on: impl Fn(Pos, Dir) -> bool) -> Vec<Pos> {
    let mut found = vec![start];
    let mut next = 0;
    while next < found.len() && found.len() <= most {
        let cell = found[next];
        next += 1;
        for dir in Dir::ALL.into_iter().filter(|&dir| goes_on(cell, dir)) {
            if !found.contains(&cell.toward(dir)) {
                found.push(cell.toward(dir));
            }
        }
    }
    found
}

fn is_tank(world: &World, pos: Pos) -> bool {
    world.block_typed(pos).is(&LIQUID_TANK)
}

/// Puts the tank blocks at `start` in order after a change there. Its tank, if it lost a block
/// or no longer fills a box, parts into tanks that do; then each grows into the tanks beside it
/// it fits with. Every tank keeps the liquid of its blocks, laid out again.
fn regroup(world: &World, start: Pos) {
    if !is_tank(world, start) {
        return;
    }
    let mut holders = Holders::outside(*world);
    let tank = joined_cells(world, start);
    let parts = if is_whole(world, &tank) {
        vec![tank]
    } else {
        part(&tank, |cells| fits(world, &holders, cells))
    };
    for cells in &parts {
        join(world, &mut holders, cells);
    }
    for cells in &parts {
        let tank = joined_cells(world, cells[0]);
        let tank_at = |pos| is_tank(world, pos).then(|| joined_cells(world, pos));
        let grown = grow(tank.clone(), tank_at, |cells| fits(world, &holders, cells));
        if grown.len() > tank.len() {
            join(world, &mut holders, &grown);
        }
    }
}

/// Whether the tank of `cells` is one still: it fills a box, and joins no block outside it.
fn is_whole(world: &World, cells: &[Pos]) -> bool {
    let joins_out = |&cell: &Pos| {
        let block = world.block_typed(cell);
        let joined = |dir: Dir| block.prop(dir.name()) == Some("true");
        Dir::ALL
            .into_iter()
            .any(|dir| joined(dir) && !cells.contains(&cell.toward(dir)))
    };
    forms_box(cells) && !cells.iter().any(joins_out)
}

/// Whether the blocks at `cells` may be one tank: they fill a box and hold one liquid.
fn fits(world: &World, holders: &Holders, cells: &[Pos]) -> bool {
    forms_box(cells) && one_liquid(world, holders, cells)
}

/// Makes the blocks at `cells` one tank, joined to each other and to nothing else, and lays the
/// liquid they hold out in it.
fn join(world: &World, holders: &mut Holders, cells: &[Pos]) {
    for &cell in cells {
        let block = world.block_typed(cell);
        let joins = |dir: Dir| cells.contains(&cell.toward(dir));
        let next = Dir::ALL
            .into_iter()
            .fold(block, |next, dir| next.with_flag(dir.name(), joins(dir)));
        if next != block {
            world.set_block_typed(cell, next);
        }
    }
    let tank = Tank::of(world, cells);
    let held = tank.held(holders);
    tank.settle(holders, held);
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

/// A block beside changed. A tank whose block beside is gone regroups, by a player or any other
/// way (a command, another mod); so does a tank block that came beside, and joins no other yet.
fn neighbor_changed(event: &mut BlockEvent) {
    let world = event.world();
    let (pos, beside) = (Pos::from(event.pos()), Pos::from(event.neighbor()));
    let Some(dir) = Dir::ALL.into_iter().find(|&dir| pos.toward(dir) == beside) else {
        return;
    };
    let (block, other) = (world.block_typed(pos), world.block_typed(beside));
    let joined = block.prop(dir.name()) == Some("true");
    let joins_back = other.is(&LIQUID_TANK) && other.prop(dir.opposite().name()) == Some("true");
    let joins_none = |tank: crate::lib::BlockState| {
        Dir::ALL
            .into_iter()
            .all(|dir| tank.prop(dir.name()) != Some("true"))
    };
    if joined && !joins_back {
        world.schedule_tick(pos, 1);
    } else if !joined && other.is(&LIQUID_TANK) && joins_none(other) {
        world.schedule_tick(beside, 1);
    }
}

/// A tank block is about to be broken: the liquid of its tank goes to the rest, shared among the
/// tanks the rest will part into by how many blocks each has. A block that goes another way (a
/// command) takes its share with it.
fn before_break(world: &World, pos: Pos) {
    if !is_tank(world, pos) {
        return;
    }
    let Some(tank) = Tank::at(world, pos) else {
        return;
    };
    let mut holders = Holders::outside(*world);
    let held = tank.held(&holders);
    let rest: Vec<Pos> = tank.blocks().filter(|&cell| cell != pos).collect();
    let parts = part(&rest, forms_box);
    let sizes: Vec<usize> = parts.iter().map(Vec::len).collect();
    for (cells, amount) in parts.iter().zip(shared(held.amount, &sizes)) {
        let liquid = Held { amount, ..held };
        Tank::of(world, cells).settle(&mut holders, liquid);
    }
    holders.set(world.block_entity(pos), Held::default());
}

/// `total` shared among tanks of `sizes` blocks by their size, none given more than it holds.
fn shared(total: u32, sizes: &[usize]) -> Vec<u32> {
    let blocks: usize = sizes.iter().sum();
    let mut left = total;
    let mut shares = Vec::new();
    for (i, &size) in sizes.iter().enumerate() {
        let fair = if i + 1 == sizes.len() {
            left
        } else {
            (total as u64 * size as u64 / blocks.max(1) as u64) as u32
        };
        let share = fair.min(left).min(CELL_CAPACITY * size as u32);
        left -= share;
        shares.push(share);
    }
    shares
}

/// A tank block is gone: its fill with it. The tanks beside it regroup (`neighbor_changed`).
fn broken(world: &World, pos: Pos) {
    fill::show_in_tank(world, pos, None);
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

    #[test]
    fn a_broken_blocks_liquid_goes_to_the_tanks_left() {
        assert_eq!(shared(9000, &[2, 1]), [6000, 3000], "by their size");
        assert_eq!(shared(10, &[1, 1, 1]), [3, 3, 4], "none lost to rounding");
        let more_than_fits = shared(3 * CELL_CAPACITY, &[1, 1]);
        assert_eq!(more_than_fits, [CELL_CAPACITY; 2], "as much as they hold");
    }
}
