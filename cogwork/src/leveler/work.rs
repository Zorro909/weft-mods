//! A leveler's tick: plan the area, then clear it from the top layer down to its height, then
//! fill it from the deepest layer up to just below.

use weft_sdk::*;

use super::{LEVELER, Level, Stage, Status};
use crate::harvest::{Harvest, cost, harvest};
use crate::lib::{Area, BlockState, Pos};
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

pub fn tick(b: &mut SystemBatch<Level>) {
    let w = b.world();
    for (id, l, _) in b.iter_mut() {
        let Some(st) = w.entity(id) else { continue };
        let pos = Pos::from(st.block_pos());
        if !BlockState::at(&w, pos).is(&LEVELER) {
            continue;
        }
        work(&w, id, pos, l);
        super::keep_loaded(&w, pos, l);
        crate::machine::show_working(&w, pos, l.status().is_working());
    }
}

/// Passes on what the leveler `id` at `pos` cleared, then works on its stage.
fn work(w: &World, id: u64, pos: Pos, l: &mut Level) {
    if l.stage() == Stage::New {
        start(w, pos, l);
    }
    crate::store::eject(w, pos);
    match l.stage() {
        Stage::Clear => clear_next(w, id, l),
        Stage::Fill => fill_next(w, id, l),
        Stage::New | Stage::Done => {}
    }
}

fn start(w: &World, pos: Pos, l: &mut Level) {
    let front = crate::machine::front(w, pos);
    l.area = take_area(w, pos).unwrap_or_else(|| square_behind(pos, front, DEFAULT_SIZE, 0));
    l.set_stage(Stage::Clear);
    l.cursor = l.area.lo.add_y(CLEAR_HEIGHT - 1);
}

/// The block looked at after `c`: along x, then z, then the next layer `dy` away.
fn next(area: &Area, c: Pos, dy: i32) -> Pos {
    if c.x < area.hi.x {
        c.add_x(1)
    } else if c.z < area.hi.z {
        Pos {
            x: area.lo.x,
            ..c.add_z(1)
        }
    } else {
        area.lo.with_y(c.y + dy)
    }
}

/// Breaks the next block standing at or above the leveler's height.
fn clear_next(w: &World, id: u64, l: &mut Level) {
    let clearable = |s: BlockState| s.can_break() && !s.is(&LEVELER);
    let Some((c, block_state)) = find(w, l, clearable) else {
        return;
    };
    l.set_status(Status::Clearing);
    if !pay(w, id, l, cost(block_state, CLEAR_BASE, CLEAR_PER_HARDNESS)) {
        return;
    }
    match harvest(w, id, c, block_state) {
        // Paid for already: it stays paid until there is room.
        Harvest::Full => l.set_status(Status::Full),
        harvested => {
            if matches!(harvested, Harvest::Done) {
                sound(w, &sounds::SCRAPE, c);
            }
            l.spent = 0;
            l.cursor = next(&l.area, c, -1);
        }
    }
}

/// Fills the next hole below the leveler's height with a block from its stock.
fn fill_next(w: &World, id: u64, l: &mut Level) {
    let Some((c, _)) = find(w, l, BlockState::can_build_into) else {
        return;
    };
    let stock = w.inventory(id).group("input");
    let Some((slot, filler)) = first_filler(&stock) else {
        l.set_status(Status::NoBlocks);
        return;
    };
    l.set_status(Status::Filling);
    if !pay(w, id, l, FILL_COST) {
        return;
    }
    filler.place(w, c);
    sound(w, &sounds::FILL, c);
    stock.set(slot, stock.get(slot).shrink(1));
    l.spent = 0;
    l.cursor = next(&l.area, c, 1);
}

/// Plays `sound` at the block `c` worked on, now and then.
fn sound(w: &World, sound: &Sound, c: Pos) {
    if w.game_time().is_multiple_of(SOUND_EVERY) {
        w.play_sound(sound, c.center(), 1.0);
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
    w: &World,
    l: &mut Level,
    wanted: impl Fn(BlockState) -> bool,
) -> Option<(Pos, BlockState)> {
    for _ in 0..SCAN {
        if stage_over(l) {
            return None;
        }
        let c = l.cursor;
        let Some(block_state) = BlockState::loaded_at(w, c) else {
            l.set_status(Status::Blocked);
            return None;
        };
        if wanted(block_state) {
            return Some((c, block_state));
        }
        let dy = if l.stage() == Stage::Clear { -1 } else { 1 };
        l.cursor = next(&l.area, c, dy);
    }
    None
}

/// Moves on to the next stage once the cursor has left this one's layers: clearing ends below
/// the leveler's height, and filling, which starts at the deepest layer, ends at it. True when
/// it moved on.
fn stage_over(l: &mut Level) -> bool {
    let height = l.area.lo.y;
    match l.stage() {
        Stage::Clear if l.cursor.y < height => {
            l.set_stage(Stage::Fill);
            l.cursor = l.area.lo.add_y(-FILL_DEPTH);
            true
        }
        Stage::Fill if l.cursor.y >= height => {
            l.set_stage(Stage::Done);
            l.set_status(Status::Done);
            true
        }
        _ => false,
    }
}

/// Puts stored energy into a job of `cost`; true once it is paid for.
fn pay(w: &World, id: u64, l: &mut Level, cost: i64) -> bool {
    let drawn = crate::machine::draw(w, id, &mut l.spent, cost, DRAW_RATE);
    if l.spent >= cost {
        return true;
    }
    if drawn == 0 {
        l.set_status(Status::NoPower);
    }
    false
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
        // Filling from y 62 to 65 starts a cube below where clearing ends, at 66.
        let area = Area::between(Pos::new(-20, 66, -20), Pos::new(40, 66, 40));
        let mut l = Level {
            stage: Stage::Clear as u32,
            area,
            cursor: area.lo.add_y(CLEAR_HEIGHT - 1),
            ..Level::default()
        };
        let mut kept = Vec::new();
        loop {
            stage_over(&mut l);
            if l.stage() == Stage::Done {
                break;
            }
            kept.push(super::super::work_cubes(&l).expect("working"));
            let dy = if l.stage() == Stage::Clear { -1 } else { 1 };
            l.cursor = next(&area, l.cursor, dy);
        }
        assert_eq!(crate::loading::kept_again(kept), None, "down, then up");
    }

    #[test]
    fn rows_then_layers_in_either_direction() {
        let area = Area::between(Pos::new(0, 5, 0), Pos::new(1, 5, 1));
        let mut c = area.lo;
        let mut seen = vec![c];
        for _ in 0..4 {
            c = next(&area, c, -1);
            seen.push(c);
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
        let mut l = leveling(Stage::Clear, 5);
        assert!(!stage_over(&mut l), "its own layer is cleared too");
        assert_eq!(l.stage(), Stage::Clear, "still clearing");
    }

    #[test]
    fn filling_starts_at_the_deepest_layer_once_cleared() {
        let mut l = leveling(Stage::Clear, 4);
        assert!(stage_over(&mut l), "below the height");
        assert_eq!(l.stage(), Stage::Fill, "filling");
        assert_eq!(l.cursor, Pos::new(0, 5 - FILL_DEPTH, 0), "deepest first");
    }

    #[test]
    fn filling_ends_below_the_leveler_height() {
        let mut l = leveling(Stage::Fill, 4);
        assert!(!stage_over(&mut l), "the layer just below is filled");
        let mut l = leveling(Stage::Fill, 5);
        assert!(stage_over(&mut l), "not its own");
        assert_eq!(l.stage(), Stage::Done, "done");
        assert_eq!(l.status(), Status::Done, "and says so");
    }
}
