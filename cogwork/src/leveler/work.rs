//! A leveler's tick: plan the area, then clear it from the top layer down to its height, then
//! fill it from the deepest layer up to just below.

use weft_sdk::*;

use super::{LEVELER, Level, Stage, Status};
use crate::harvest::{Harvest, cost, harvest};
use crate::lib::{Area, BlockState, Pos, TypedWorld};
use crate::machine::{self, Payment};
use crate::markers::{square_behind, take_area};
use crate::sounds;

/// Layers cleared from the leveler's height up, and filled below it.
const CLEAR_HEIGHT: i32 = 16;
const FILL_DEPTH: i32 = 4;
/// The area when no markers say otherwise: 9 by 9 behind it.
const DEFAULT_SIZE: i32 = 9;
/// Energy to clear a block (a base and as much again per point of hardness), and to fill one.
const CLEAR_BASE: f32 = 100.0;
const CLEAR_PER_HARDNESS: f32 = 200.0;
const FILL_COST: i64 = 60;
/// The most energy it puts into its work per tick.
const DRAW_RATE: i64 = 100;
/// The most blocks looked at per tick for the next one to work on.
const SCAN: u32 = 512;
/// Ticks between its sounds at most: it may clear or fill a block every tick.
const SOUND_EVERY: u64 = 4;

pub fn tick(batch: &mut SystemBatch<Level>) {
    let world = batch.world();
    for (id, level, _) in batch.iter_mut() {
        let Some(entity) = world.entity(id) else {
            continue;
        };
        let pos = Pos::from(entity.block_pos());
        if !world.block_typed(pos).is(&LEVELER) {
            continue;
        }
        work(&world, id, pos, level);
        super::keep_loaded(&world, pos, level);
        crate::machine::show_working(&world, pos, level.status().is_working());
    }
}

/// Passes on what the leveler `id` at `pos` cleared, then works on its stage.
fn work(world: &World, id: u64, pos: Pos, level: &mut Level) {
    if level.stage() == Stage::New {
        start(world, pos, level);
    }
    crate::store::eject(world, pos);
    match level.stage() {
        Stage::Clear => clear_next(world, id, level),
        Stage::Fill => fill_next(world, id, level),
        Stage::New | Stage::Done => {}
    }
}

fn start(world: &World, pos: Pos, level: &mut Level) {
    let front = crate::machine::front(world, pos);
    level.area =
        take_area(world, pos).unwrap_or_else(|| square_behind(pos, front, DEFAULT_SIZE, 0));
    level.set_stage(Stage::Clear);
    level.cursor = level.area.lo.add_y(CLEAR_HEIGHT - 1);
}

/// The block looked at after `cell`: along x, then z, then the next layer `dy` away.
fn next(area: &Area, cell: Pos, dy: i32) -> Pos {
    if cell.x < area.hi.x {
        cell.add_x(1)
    } else if cell.z < area.hi.z {
        Pos {
            x: area.lo.x,
            ..cell.add_z(1)
        }
    } else {
        area.lo.with_y(cell.y + dy)
    }
}

/// Moves the cursor on to the next block of the leveler's stage: layer by layer down while it
/// clears, up while it fills.
fn move_on(level: &mut Level) {
    let dy = if level.stage() == Stage::Clear { -1 } else { 1 };
    level.cursor = next(&level.area, level.cursor, dy);
}

/// Breaks the next block standing at or above the leveler's height.
fn clear_next(world: &World, id: u64, level: &mut Level) {
    let clearable = |state: BlockState| state.can_break() && !state.is(&LEVELER);
    let Some((cell, block_state)) = find(world, level, clearable) else {
        return;
    };
    level.set_status(Status::Clearing);
    let cost = cost(block_state, CLEAR_BASE, CLEAR_PER_HARDNESS);
    if !pay(world, id, level, cost) {
        return;
    }
    match harvest(world, id, cell, block_state) {
        Harvest::Full => {
            // Paid for already: it stays paid until there is room.
            level.set_status(Status::Full);
            return;
        }
        Harvest::Done => sound(world, &sounds::SCRAPE, cell),
        Harvest::Refused => {}
    }
    level.spent = 0;
    move_on(level);
}

/// Fills the next hole below the leveler's height with a block from its stock.
fn fill_next(world: &World, id: u64, level: &mut Level) {
    let Some((cell, _)) = find(world, level, BlockState::can_build_into) else {
        return;
    };
    let stock = world.inventory(id).group("input");
    let Some((slot, filler)) = first_filler(&stock) else {
        level.set_status(Status::NoBlocks);
        return;
    };
    level.set_status(Status::Filling);
    if !pay(world, id, level, FILL_COST) {
        return;
    }
    world.set_block_typed(cell, filler);
    sound(world, &sounds::FILL, cell);
    stock.set(slot, stock.get(slot).shrink(1));
    level.spent = 0;
    move_on(level);
}

/// Plays `sound` at the block `cell` worked on, now and then.
fn sound(world: &World, sound: &Sound, cell: Pos) {
    if world.game_time().is_multiple_of(SOUND_EVERY) {
        world.play_sound(sound, cell.center(), 1.0);
    }
}

/// The first stack in `stock` that can fill a hole: its slot, and the block it places.
fn first_filler(stock: &SlotGroup) -> Option<(u32, BlockState)> {
    (0..stock.len()).find_map(|slot| fill_block(stock.get(slot)).map(|block| (slot, block)))
}

/// The block a stack places, if it is a whole one fit to fill with.
fn fill_block(stack: ItemStack) -> Option<BlockState> {
    let block_state = BlockState::placed_by(stack.item).filter(|_| !stack.is_empty())?;
    block_state.is_whole().then_some(block_state)
}

/// From the cursor on, the next block that `wanted` picks and its state. `None` while waiting,
/// when there is nothing this tick, or when the stage is over.
fn find(
    world: &World,
    level: &mut Level,
    wanted: impl Fn(BlockState) -> bool,
) -> Option<(Pos, BlockState)> {
    for _ in 0..SCAN {
        if stage_over(level) {
            return None;
        }
        let cell = level.cursor;
        let Some(block_state) = world.block_loaded_typed(cell) else {
            level.set_status(Status::Blocked);
            return None;
        };
        if wanted(block_state) {
            return Some((cell, block_state));
        }
        move_on(level);
    }
    None
}

/// Moves on to the next stage once the cursor has left this one's layers: clearing ends below
/// the leveler's height, and filling, which starts at the deepest layer, ends at it. True when
/// it moved on.
fn stage_over(level: &mut Level) -> bool {
    let height = level.area.lo.y;
    match level.stage() {
        Stage::Clear if level.cursor.y < height => {
            level.set_stage(Stage::Fill);
            level.cursor = level.area.lo.add_y(-FILL_DEPTH);
            true
        }
        Stage::Fill if level.cursor.y >= height => {
            level.set_stage(Stage::Done);
            level.set_status(Status::Done);
            true
        }
        _ => false,
    }
}

/// Puts stored energy into a job of `cost`; true once it is paid for.
fn pay(world: &World, id: u64, level: &mut Level, cost: i64) -> bool {
    let payment = machine::pay(world, id, &mut level.spent, cost, DRAW_RATE);
    if payment == Payment::NoPower {
        level.set_status(Status::NoPower);
    }
    payment == Payment::Paid
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leveling(stage: Stage, cursor_y: i32) -> Level {
        let area = Area::between(Pos::new(0, 5, 0), Pos::new(8, 5, 8));
        Level {
            stage: stage as u32,
            area,
            cursor: area.lo.with_y(cursor_y),
            ..Level::default()
        }
    }

    #[test]
    fn no_cube_is_let_go_and_kept_again_over_a_job() {
        // At 66, filling from 62 to 65 starts a cube below where clearing ends; at 64, filling
        // ends where a cube begins.
        for height in [66, 64] {
            let area = Area::between(Pos::new(-20, height, -20), Pos::new(40, height, 40));
            let mut level = Level {
                stage: Stage::Clear as u32,
                area,
                cursor: area.lo.add_y(CLEAR_HEIGHT - 1),
                ..Level::default()
            };
            // What it keeps after each block, as a tick does once the cursor has moved on.
            let mut kept = Vec::new();
            loop {
                stage_over(&mut level);
                if level.stage() == Stage::Done {
                    break;
                }
                move_on(&mut level);
                kept.extend(super::super::work_cubes(&level));
            }
            let again = crate::loading::kept_again(kept);
            assert_eq!(again, None, "down, then up, at {height}");
        }
    }

    #[test]
    fn rows_then_layers_in_either_direction() {
        let area = Area::between(Pos::new(0, 5, 0), Pos::new(1, 5, 1));
        let mut cell = area.lo;
        let mut seen = vec![cell];
        for _ in 0..4 {
            cell = next(&area, cell, -1);
            seen.push(cell);
        }
        let expected = [[0, 5, 0], [1, 5, 0], [0, 5, 1], [1, 5, 1], [0, 4, 0]];
        assert_eq!(seen, expected.map(Pos::from), "along x, then z, then down");
        assert_eq!(
            next(&area, Pos::new(1, 5, 1), 1),
            Pos::new(0, 6, 0),
            "or up"
        );
    }

    #[test]
    fn clearing_goes_on_down_to_the_leveler_height() {
        let mut level = leveling(Stage::Clear, 5);
        assert!(!stage_over(&mut level), "its own layer is cleared too");
        assert_eq!(level.stage(), Stage::Clear, "still clearing");
    }

    #[test]
    fn filling_starts_at_the_deepest_layer_once_cleared() {
        let mut level = leveling(Stage::Clear, 4);
        assert!(stage_over(&mut level), "below the height");
        assert_eq!(level.stage(), Stage::Fill, "filling");
        assert_eq!(
            level.cursor,
            Pos::new(0, 5 - FILL_DEPTH, 0),
            "deepest first"
        );
    }

    #[test]
    fn filling_ends_below_the_leveler_height() {
        let mut level = leveling(Stage::Fill, 4);
        assert!(!stage_over(&mut level), "the layer just below is filled");
        let mut level = leveling(Stage::Fill, 5);
        assert!(stage_over(&mut level), "not its own");
        assert_eq!(level.stage(), Stage::Done, "done");
        assert_eq!(level.status(), Status::Done, "and says so");
    }
}
